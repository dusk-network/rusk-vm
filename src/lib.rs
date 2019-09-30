#![feature(drain_filter)]
mod digest;
mod host_fns;
// mod prepare_module;
mod state;
// mod transaction;
mod contract_builder;
mod helpers;
mod wallet;

pub use state::NetworkState;
pub use wallet::Wallet;

#[cfg(test)]
mod tests {
    use crate::contract_builder::ContractBuilder;
    use crate::contract_code;
    use crate::digest::Digest;
    use crate::state::NetworkState;
    use crate::wallet::Wallet;
    //use ethereum_types::U256;

    #[test]
    fn default_account() {
        use default_account::AccountCall;
        use dusk_abi::{
            encoding,
            types::{Signature, H256},
        };

        let mut network = NetworkState::genesis(
            contract_code!("default_account"),
            1_000_000_000,
        );
        let mut wallet = Wallet::genesis();

        let pub_key = wallet.default_account().public_key();

        // borrow checker cannot prove that genesis_id() is constant.
        // so we clone.
        let genesis_id = network.genesis_id().clone();
        // make a dummy call
        network.call_contract(&genesis_id, 0, &[]).unwrap();

        // let signer = wallet.default_account().signer();

        // Burn 123 tokens, by sending them to an unretrievable address.
        // let call = AccountCall::new(signer, H256::zero(), 123, &[0, 11, 2]);

        // let mut buffer = [0u8; 120];

        // encoding::encode(&call, &mut buffer);
    }

    // #[test]
    // fn simple_transactions() {
    //     let mut wallet = Wallet::new();

    //     wallet.new_account("secondary").unwrap();

    //     let mut network = NetworkState::genesis(wallet.default_account());

    //     assert_eq!(
    //         network
    //             .get_account(&wallet.default_account().id())
    //             .unwrap()
    //             .balance(),
    //         1_000_000
    //     );

    //     // our local wallet has not been synced yet
    //     assert_eq!(wallet.default_account().balance(), 0);

    //     // sync local wallet
    //     wallet.sync(&network);

    //     // now the genesis mint should be readable
    //     assert_eq!(wallet.default_account().balance(), 1_000_000);

    //     // get the id of our secondary account
    //     let secondary_id = wallet.get_account("secondary").unwrap().id();

    //     // cannot send more than we have
    //     assert!(wallet
    //         .default_account_mut()
    //         .send_value(&secondary_id, 2_000_000)
    //         .is_err());

    //     // send 1000 Dusk to secondary account!
    //     let transaction = wallet
    //         .default_account_mut()
    //         .send_value(&secondary_id, 1000)
    //         .unwrap();

    //     // put the transaction in the queue
    //     network.queue_transaction(transaction);
    //     network.mint_block().unwrap();

    //     // sync local wallet again
    //     wallet.sync(&network);

    //     // Transaction should have taken place
    //     assert_eq!(wallet.default_account().balance(), 999_000);
    //     assert_eq!(wallet.get_account("secondary").unwrap().balance(), 1000);
    // }

    // #[test]
    // fn store_in_deploy() {
    //     const CONTRACT: &'static [u8] = include_bytes!("../test_contracts/store_in_deploy/target/wasm32-unknown-unknown/release/store_in_deploy.wasm");

    //     let mut wallet = Wallet::new();
    //     let mut network = NetworkState::genesis(wallet.default_account());

    //     wallet.sync(&network);

    //     // deploy contract with 1000 dusk
    //     let (transaction, contract_id) = wallet
    //         .default_account_mut()
    //         .deploy_contract(CONTRACT, 1000)
    //         .unwrap();

    //     network.queue_transaction(transaction);
    //     network.mint_block().unwrap();

    //     // should have written value H256::max_value() to key H256::max_value()
    //     assert_eq!(
    //         network
    //             .get_contract(&contract_id)
    //             .expect("a")
    //             .storage()
    //             .get(&H256::max_value())
    //             .expect("b"),
    //         &H256::max_value()
    //     );

    //     let max_minus_one = H256::max_value() - 1;

    //     assert!(network
    //         .get_contract(&contract_id)
    //         .unwrap()
    //         .storage()
    //         .get(&max_minus_one)
    //         .is_none());

    //     let transaction = wallet
    //         .default_account_mut()
    //         .call_contract(&contract_id, 0, b"hello world")
    //         .unwrap();

    //     network.queue_transaction(transaction);
    //     network.mint_block().unwrap();

    //     assert_eq!(
    //         network
    //             .get_contract(&contract_id)
    //             .expect("a")
    //             .storage()
    //             .get(&max_minus_one)
    //             .expect("b"),
    //         &max_minus_one
    //     );
    // }

    // #[test]
    // fn sub_token() {
    //     use sub_token;

    //     const CONTRACT: &'static [u8] = test_source!("sub_token");
    //     const OWNER: [u8; 32] = [0u8; 32];

    //     let mut wallet = Wallet::new();
    //     let mut network = NetworkState::genesis(wallet.default_account());
    //     wallet.new_account("secondary").unwrap();

    //     wallet.sync(&network);

    //     // deploy contract with 0 dusk
    //     let (transaction, contract_id) = wallet
    //         .default_account_mut()
    //         .deploy_contract(CONTRACT, 0)
    //         .unwrap();

    //     network.queue_transaction(transaction);
    //     network.mint_block().unwrap();

    //     // default sub_token account contains 1_000_000_000 dusk;
    //     assert_eq!(
    //         network
    //             .get_contract(&contract_id)
    //             .expect("a")
    //             .storage()
    //             .get(&wallet.default_account().id())
    //             .expect("b"),
    //         &1_000_000_000.into()
    //     );

    //     let client = sub_token::ClientApi::debug();

    //     //let transaction = client.transact(wallet.get_account("secondary").unwrap().id());
    // }
}
