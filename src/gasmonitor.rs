// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_abi::ContractId;
use std::io::Write;
use wabt::wasm2wat;

pub fn export_wat<S: AsRef<[u8]>>(
    bytecode: S,
    _id: ContractId,
    is_instr: bool,
) {
    //unimplemented!();
    let wat = wasm2wat(bytecode).expect("failed to parse wasm to wat.");
    let filename = {
        if is_instr {
            concat!("tests/", "_deployed.wat")
        } else {
            concat!("tests/", "_undeployed.wat")
        }
    };
    let mut file =
        std::fs::File::create(filename).expect("Couldn't create file");
    file.write_all(wat.as_bytes())
        .expect("Couldn't write into file");
}
