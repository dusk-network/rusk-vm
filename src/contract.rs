use std::io;
use std::rc::Rc;

use dusk_abi::H256;
use failure::{bail, err_msg, Error};
use kelvin::{ByteHash, Content, Sink, Source};
use parity_wasm::elements::{
    self, InitExpr, Instruction, Internal, Serialize, Type, ValueType,
};
use pwasm_utils::rules;
use wasmi::Module as WasmiModule;

use crate::{Schedule, VMError};

use std::{mem, ptr};

/// read out the i32 const in a WASM `InitExpr`
fn get_i32_const(init_expr: &InitExpr) -> Option<i32> {
    let code = init_expr.code();

    assert!(code.len() == 2);
    assert!(code[1] == Instruction::End);

    if let Instruction::I32Const(ofs) = code[0] {
        Some(ofs)
    } else {
        None
    }
}

#[derive(Clone)]
pub enum MeteredContract {
    Code(Vec<u8>),
    Module {
        module: Rc<WasmiModule>,
        code: Vec<u8>,
    },
}

impl MeteredContract {
    pub fn bytecode(&self) -> &[u8] {
        match self {
            MeteredContract::Code(code)
            | MeteredContract::Module { code, .. } => &code,
        }
    }

    pub fn empty() -> Self {
        MeteredContract::Code(vec![])
    }

    pub fn ensure_compiled(&mut self) -> Result<(), VMError> {
        if let MeteredContract::Code(_) = self {
            if let MeteredContract::Code(code) =
                mem::replace(self, MeteredContract::Code(vec![]))
            {
                if code.len() == 0 {
                    return Err(VMError::UnknownContract);
                }
                *self = MeteredContract::Module {
                    module: Rc::new(wasmi::Module::from_buffer(&code)?),
                    code,
                };
            }
        }
        Ok(())
    }
}

/// A parsed contract module, not metered, can still be parameterized with the
/// `set_parameter` call
pub struct Contract<'a> {
    hash: H256,
    module: elements::Module,
    schedule: &'a Schedule,
}

impl<'a> Contract<'a> {
    /// Creates a new Contract from provided code and Schedule.
    pub fn new<H: ByteHash>(
        original_code: &[u8],
        schedule: &'a Schedule,
    ) -> Result<Self, VMError> {
        use wasmi_validation::PlainValidator;

        let hash = H256::from_bytes(H::hash(original_code).as_ref());

        let module = elements::deserialize_buffer(original_code)
            .map_err(|_| VMError::InvalidWASMModule)?;

        // Make sure that the module is valid.
        wasmi_validation::validate_module::<PlainValidator>(&module)
            .map_err(|_| VMError::InvalidWASMModule)?;

        let mut contract_module = Contract {
            hash,
            module,
            schedule,
        };

        contract_module
            .ensure_no_floating_types()
            .map_err(|_| VMError::InvalidWASMModule)?;
        contract_module
            .ensure_table_size_limit(&schedule)
            .map_err(|_| VMError::InvalidWASMModule)?;

        contract_module = contract_module
            .inject_gas_metering()
            .map_err(|_| VMError::InvalidWASMModule)?
            .inject_stack_height_metering()
            .map_err(|_| VMError::InvalidWASMModule)?;

        // Return a `Contract` instance with
        // __valid__ module.
        Ok(Contract {
            hash,
            module: contract_module.module,
            schedule,
        })
    }

    /// Returns the Contract's hash
    pub fn hash(&self) -> H256 {
        self.hash
    }
    /// Injects gas metering into the contract
    fn inject_gas_metering(self) -> Result<Self, failure::Error> {
        let gas_rules = rules::Set::new(
            self.schedule.regular_op_cost as u32,
            Default::default(),
        )
        .with_grow_cost(self.schedule.grow_mem_cost as u32)
        .with_forbidden_floats();

        let contract_module =
            pwasm_utils::inject_gas_counter(self.module, &gas_rules)
                .map_err(|_| err_msg("gas instrumentation failed"))?;
        Ok(Contract {
            hash: self.hash,
            module: contract_module,
            schedule: self.schedule,
        })
    }

    /// Injects stack height metering
    fn inject_stack_height_metering(self) -> Result<Self, failure::Error> {
        let contract_module = pwasm_utils::stack_height::inject_limiter(
            self.module,
            self.schedule.max_stack_height,
        )
        .map_err(|_| err_msg("stack height instrumentation failed"))?;
        Ok(Contract {
            hash: self.hash,
            module: contract_module,
            schedule: self.schedule,
        })
    }

    /// Ensures that tables declared in the module are not too big.
    fn ensure_table_size_limit(
        &self,
        schedule: &Schedule,
    ) -> Result<(), failure::Error> {
        if let Some(table_section) = self.module.table_section() {
            // In Wasm MVP spec, there may be at most one table declared. Double check this
            // explicitly just in case the Wasm version changes.
            if table_section.entries().len() > 1 {
                return Err(err_msg("multiple tables declared"));
            }
            if let Some(table_type) = table_section.entries().first() {
                // Check the table's initial size as there is no instruction or environment function
                // capable of growing the table.
                if table_type.limits().initial() > schedule.max_table_size {
                    return Err(err_msg("table exceeds maximum size allowed"));
                }
            }
        }
        Ok(())
    }

    /// Ensures that no floating point types are in use.
    fn ensure_no_floating_types(&self) -> Result<(), failure::Error> {
        if let Some(global_section) = self.module.global_section() {
            for global in global_section.entries() {
                match global.global_type().content_type() {
                    ValueType::F32 | ValueType::F64 => return Err(err_msg(
                        "use of floating point type in globals is forbidden",
                    )),
                    _ => {}
                }
            }
        }

        if let Some(code_section) = self.module.code_section() {
            for func_body in code_section.bodies() {
                for local in func_body.locals() {
                    match local.value_type() {
                        ValueType::F32 | ValueType::F64 => return Err(
                            err_msg("use of floating point type in locals is forbidden"),
                        ),
                        _ => {}
                    }
                }
            }
        }

        if let Some(type_section) = self.module.type_section() {
            for wasm_type in type_section.types() {
                match wasm_type {
                    Type::Function(func_type) => {
                        let return_type = func_type.return_type();
                        for value_type in
                            func_type.params().iter().chain(return_type.iter())
                        {
                            match value_type {
								ValueType::F32 | ValueType::F64 => {
									return Err(
										err_msg("use of floating point type in function types is forbidden"),
									)
								}
								_ => {}
							}
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Modifies a parameter in the contract body
    pub fn set_parameter<V: Copy + std::fmt::Debug + Sized>(
        &mut self,
        name: &str,
        value: V,
    ) -> Result<(), Error> {
        // Find the global index of the Parameter
        let mut global_index = None;
        if let Some(export) = self.module.export_section() {
            for e in export.entries() {
                if e.field() == name {
                    if let Internal::Global(index) = e.internal() {
                        global_index = Some(index)
                    }
                }
            }
        }

        // Find the offset of the Parameter
        let mut offset = None;
        if let Some(index) = global_index {
            if let Some(global_section) = self.module.global_section() {
                let init_expr =
                    global_section.entries()[*index as usize].init_expr();

                if let Some(ofs) = get_i32_const(init_expr) {
                    offset = Some(ofs);
                } else {
                    bail!("Invalid global init expression")
                }
            }
        }

        // Update the pointed-to value in the data section
        if let Some(mut data_offset) = offset {
            if let Some(data) = self.module.data_section_mut() {
                let entries = data.entries_mut();

                // Find the correct data section by offset.
                // i.e, the largest one that is smaller than data_offset.
                let mut best_ofs = None;
                let mut best_idx = 0;

                for (i, entry) in entries.iter().enumerate() {
                    if let Some(section_init_expr) = entry.offset() {
                        // the offset init expr
                        if let Some(section_offset) =
                            get_i32_const(section_init_expr)
                        {
                            // the actual offset
                            match best_ofs {
                                None => best_ofs = Some(section_offset),
                                Some(current_best) => {
                                    if section_offset > current_best
                                        && section_offset <= data_offset
                                    {
                                        best_ofs = Some(section_offset);
                                        best_idx = i;
                                    }
                                }
                            }
                        }
                    }
                }

                // Subtract the offset
                if let Some(best_ofs) = best_ofs {
                    data_offset -= best_ofs;
                } else {
                    bail!("Could not find correct data segment")
                }

                let entry = &mut entries[best_idx];
                let segment = entry.value_mut();

                // make sure there's enough room in the buffer,
                // and that data_offset is positive
                assert!(data_offset >= 0);
                assert!(
                    segment.len() - data_offset as usize >= mem::size_of::<V>()
                );

                let offset_segment = &mut segment[data_offset as usize];

                // overwrite the value
                // TODO, consider endianness etc here, using serde?
                unsafe {
                    let pointer: &mut V = mem::transmute(offset_segment);
                    ptr::write_unaligned(pointer, value);
                }
            }

            Ok(())
        } else {
            bail!("No such parameter")
        }
    }

    /// Builds a `MeteredContract` from the `Contract`
    pub fn build(self) -> Result<MeteredContract, VMError> {
        let mut code = vec![];
        self.module
            .serialize(&mut code)
            .map_err(|_| VMError::InvalidWASMModule)?;
        Ok(MeteredContract::Code(code))
    }
}

impl<H: ByteHash> Content<H> for MeteredContract {
    fn persist(&mut self, sink: &mut Sink<H>) -> io::Result<()> {
        match self {
            MeteredContract::Code(code)
            | MeteredContract::Module { code, .. } => code.persist(sink),
        }
    }

    fn restore(source: &mut Source<H>) -> io::Result<Self> {
        Ok(MeteredContract::Code(Vec::restore(source)?))
    }
}
