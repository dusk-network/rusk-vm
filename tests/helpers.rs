// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

#[macro_export]
macro_rules! contract_code {
    ($name:expr) => {
        include_bytes!(concat!(
            "contracts/",
            $name,
            "/target/wasm32-unknown-unknown/release/",
            $name,
            ".wasm"
        ))
    };
}
