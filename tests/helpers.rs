#[macro_export]
macro_rules! contract_code {
    ($name:expr) => {
        include_bytes!(concat!(
            "contracts/",
            $name,
            "/wasm/target/wasm32-unknown-unknown/release/",
            $name,
            ".wasm"
        ))
    };
}
