use crate::digest::{HashState, MakeDigest};
use crate::traits::SaturatedConversion;
use crate::Schedule;
use failure::{bail, err_msg, Error};
use parity_wasm::elements::{self, InitExpr, Instruction, Internal, Serialize};
use pwasm_utils;
use pwasm_utils::rules;

use std::{mem, ptr};
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

#[derive(Default, Debug, Clone)]
pub struct Contract(Vec<u8>);

impl Contract {
    pub fn bytecode(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytecode(self) -> Vec<u8> {
        self.0
    }
}

impl MakeDigest for Contract {
    fn make_digest(&self, state: &mut HashState) {
        state.update(&self.0);
    }
}

pub struct ContractBuilder<'a> {
    module: elements::Module,
    schedule: &'a Schedule,
}
impl<'a> ContractBuilder<'a> {
    pub fn new(
        original_code: &[u8],
        schedule: &'a Schedule,
    ) -> Result<Self, Error> {
        use wasmi_validation::{validate_module, PlainValidator};

        let module = elements::deserialize_buffer(original_code)
            .map_err(|_| err_msg("Can't decode wasm code"))?;

        // Make sure that the module is valid.
        validate_module::<PlainValidator>(&module)
            .map_err(|_| err_msg("Module is not valid"))?;

        let mut contract_module = ContractBuilder { module, schedule };

        contract_module = contract_module
            .inject_gas_metering()?
            .inject_stack_height_metering()?;

        // Return a `ContractModule` instance with
        // __valid__ module.
        Ok(ContractBuilder {
            module: contract_module.module,
            schedule,
        })
    }

    fn inject_gas_metering(self) -> Result<Self, failure::Error> {
        let gas_rules = rules::Set::new(
            self.schedule.regular_op_cost.clone().saturated_into(),
            Default::default(),
        )
        .with_grow_cost(self.schedule.grow_mem_cost.clone().saturated_into())
        .with_forbidden_floats();

        let contract_module =
            pwasm_utils::inject_gas_counter(self.module, &gas_rules)
                .map_err(|_| err_msg("gas instrumentation failed"))?;
        Ok(ContractBuilder {
            module: contract_module,
            schedule: self.schedule,
        })
    }

    fn inject_stack_height_metering(self) -> Result<Self, failure::Error> {
        let contract_module = pwasm_utils::stack_height::inject_limiter(
            self.module,
            self.schedule.max_stack_height,
        )
        .map_err(|_| err_msg("stack height instrumentation failed"))?;
        Ok(ContractBuilder {
            module: contract_module,
            schedule: self.schedule,
        })
    }

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

    pub fn build(self) -> Result<Contract, Error> {
        let mut vec = vec![];
        self.module.serialize(&mut vec)?;
        Ok(Contract(vec))
    }
}
