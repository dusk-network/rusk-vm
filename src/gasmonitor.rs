// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_abi::ContractId;
use wabt::{wasm2wat, Error as WabtError};

pub fn export_wat<S: AsRef<[u8]>>(
    bytecode: S,
    id: ContractId,
    is_instr: bool,
) -> Result<(), WabtError> {
    unimplemented!();
}
