// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use anyhow::Result;
use dusk_plonk::prelude::*;

// Implements a circuit that checks:
// 1) a + b = c where C is a PI
// 2) a <= 2^6
// 3) b <= 2^5
// 4) a * b = d where D is a PI
pub struct TestCircuit {
    pub inputs: Option<[BlsScalar; 4]>,
    pub pi_positions: Vec<PublicInput>,
    pub trim_size: usize,
}

impl Default for TestCircuit {
    fn default() -> Self {
        TestCircuit {
            inputs: Some([BlsScalar::zero(); 4]),
            pi_positions: vec![],
            trim_size: 1 << 9,
        }
    }
}

impl Circuit<'_> for TestCircuit {
    fn gadget(&mut self, composer: &mut StandardComposer) -> Result<()> {
        let inputs = self
            .inputs
            .ok_or_else(|| CircuitErrors::CircuitInputsNotFound)?;
        let pi = self.get_mut_pi_positions();
        let a = composer.add_input(inputs[0]);
        let b = composer.add_input(inputs[1]);
        let zero = composer.add_input(BlsScalar::zero());
        // Make first constraint a + b = c
        pi.push(PublicInput::BlsScalar(-inputs[2], composer.circuit_size()));
        composer.poly_gate(
            a,
            b,
            zero,
            BlsScalar::zero(),
            BlsScalar::one(),
            BlsScalar::one(),
            BlsScalar::zero(),
            BlsScalar::zero(),
            -inputs[2],
        );

        // Check that a and b are in range
        composer.range_gate(a, 1 << 6);
        composer.range_gate(b, 1 << 5);
        // Make second constraint a * b = d
        pi.push(PublicInput::BlsScalar(-inputs[3], composer.circuit_size()));
        composer.poly_gate(
            a,
            b,
            zero,
            BlsScalar::one(),
            BlsScalar::zero(),
            BlsScalar::zero(),
            BlsScalar::one(),
            BlsScalar::zero(),
            -inputs[3],
        );
        Ok(())
    }

    #[inline]
    fn get_trim_size(&self) -> usize {
        self.trim_size
    }

    fn set_trim_size(&mut self, size: usize) {
        self.trim_size = size;
    }

    fn get_mut_pi_positions(&mut self) -> &mut Vec<PublicInput> {
        &mut self.pi_positions
    }

    fn get_pi_positions(&self) -> &Vec<PublicInput> {
        &self.pi_positions
    }
}
