use failure::Error;
use parity_wasm::elements::{Internal, Module, Section, Serialize};
use wasmi;

trait ModuleExt {
    fn remove_fn_recursive(&mut self, index: u32);
    fn remove_named_fn_recursive(&mut self, name: &str);
}

impl ModuleExt for Module {
    fn remove_named_fn_recursive(&mut self, name: &str) {
        let exports =
            self.export_section_mut().take().expect("no export section");
        let removed = exports
            .entries_mut()
            .drain_filter(|e| e.field() == name)
            .next()
            .expect("no function with this name");

        match removed.internal() {
            Internal::Function(function_idx) => {
                self.remove_fn_recursive(*function_idx)
            }
            _ => panic!("named entry is not a function"),
        }
    }

    fn remove_fn_recursive(&mut self, _index: u32) {
        // make a graph of which functions are calling each other
        // for example we have these functions (from the tests)

        // fn from_both(a: u32) -> u32 { ... }
        //
        // fn from_call(b: u32) -> u32 {
        //     from_call_deep(b)
        // }
        //
        // fn from_call_deep(b: u32) -> u32 { ... }
        // fn from_deploy(c: u32) -> u32 { .. }

        // pub fn deploy() -> u32 {
        //     from_both(0) + from_deploy(0)
        // }

        // pub fn call() -> u32 {
        //     from_both(0) + from_call(0)
        // }

        // resulting call-graph

        // from_both : call, deploy
        // from_call: call
        // from_call_deep: from_call
        // from_deploy: deploy,
        // deploy: ()
        // call: ()
    }

    // fn called_by(&mut self, accumulator: &mut Vec<u32>) {
    //     unimplemented!()
    // }
}

pub fn prepare_module(
    bytecode: &[u8],
) -> Result<(wasmi::Module, Vec<u8>), Error> {
    let mut full_module: Module = parity_wasm::deserialize_buffer(bytecode)?;

    // strip custom sections
    full_module.sections_mut().retain(|section| match section {
        Section::Function(_)
        | Section::Import(_)
        | Section::Table(_)
        | Section::Type(_)
        | Section::Code(_)
        | Section::Data(_)
        | Section::Export(_)
        | Section::Global(_)
        | Section::Memory(_) => true,
        _ => false,
    });

    let ctor = full_module.clone();
    let contract = full_module;

    // ctor.remove_named_fn_recursive("call");
    // contract.remove_named_fn_recursive("deploy");

    let mut contract_bytecode = vec![];
    contract.serialize(&mut contract_bytecode)?;

    // // create a data section if none is available
    // if ctor.data_section().is_none() {
    //     ctor.sections_mut()
    //         .push(Section::Data(DataSection::with_entries(vec![])))
    // }

    // // inject the contract code into the data section
    // let data_section = ctor
    //     .data_section_mut()
    //     .expect("Above none_check guarantees success here");
    // let entries = data_section.entries_mut();
    // let len = entries.len();
    // let segment =
    //     DataSegment::new(core::u32::MAX, InitExpr::empty(), contract_bytecode);
    // data_section.entries_mut().push(segment);

    let ctor = wasmi::Module::from_parity_wasm_module(ctor)?;
    ctor.deny_floating_point()?;

    Ok((ctor, contract_bytecode))
}
