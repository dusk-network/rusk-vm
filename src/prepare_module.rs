use failure::Error;
use parity_wasm::elements::{Module, Section};
use wasmi;

pub fn prepare_module(
    bytecode: &[u8],
) -> Result<(wasmi::Module, wasmi::Module), Error> {
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
        Section::Start(_) => {
            panic!("Contract module not allowed to have a start section")
        }
        _ => false,
    });

    let ctor = wasmi::Module::from_parity_wasm_module(full_module.clone())?;
    let contract = wasmi::Module::from_parity_wasm_module(full_module)?;

    ctor.deny_floating_point()?;
    contract.deny_floating_point()?;

    Ok((ctor, contract))
}
