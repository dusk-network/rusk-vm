#![feature(drain_filter)]
mod contract;
mod digest;
mod helpers;
mod host_fns;
mod interfaces;
mod state;
mod wallet;

pub use contract::ContractBuilder;
pub use interfaces::DefaultAccount;
pub use state::NetworkState;
pub use wallet::Wallet;

#[cfg(test)]
mod tests {
    use super::*;

    use digest::Digest;

    #[test]
    fn default_account() {
        let mut wallet = Wallet::new();

        let mut genesis_builder =
            ContractBuilder::new(contract_code!("default_account")).unwrap();

        let pub_key = wallet.default_account().public_key();
        genesis_builder
            .set_parameter("PUBLIC_KEY", pub_key)
            .unwrap();

        let genesis = genesis_builder.build().unwrap();

        // New genesis network with initial value
        let mut network = NetworkState::genesis(genesis, 1_000_000_000);

        let genesis_id = network.genesis_id().clone();

        // setup a secondary account

        wallet.new_account("alice");

        let mut account_builder =
            ContractBuilder::new(contract_code!("default_account")).unwrap();

        let alice_pub_key = wallet.get_account("alice").unwrap().public_key();
        account_builder
            .set_parameter("PUBLIC_KEY", alice_pub_key)
            .unwrap();

        let alice_account = account_builder.build().unwrap();

        // transfer 1000 to alice from genesis account

        let genesis_signer = wallet.default_account().signer();

        let call = DefaultAccount::transfer(
            genesis_signer,
            alice_account.digest(),
            1000,
            0,
        )
        .unwrap();

        println!("{:?}", call);

        network.call_contract(&genesis_id, &call).unwrap();
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
