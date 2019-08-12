mod digest;
mod host_fns;
mod state;
mod transaction;
mod wallet;

pub use state::NetworkState;
pub use wallet::Wallet;

#[cfg(test)]
mod tests {

    use crate::state::NetworkState;
    use crate::wallet::Wallet;

    #[test]
    fn simple_transactions() {
        let mut wallet = Wallet::new();

        let primary = wallet.default_account();
        wallet.new_account("secondary").unwrap();

        let mut network = NetworkState::genesis(wallet.default_account());

        assert_eq!(
            network
                .get_account(&wallet.default_account().id())
                .unwrap()
                .balance(),
            1_000_000
        );

        // our local wallet has not been synced yet
        assert_eq!(wallet.default_account().balance(), 0);

        // sync local wallet
        wallet.sync(&network);

        // now the genesis mint should be readable
        assert_eq!(wallet.default_account().balance(), 1_000_000);

        // get the id of our secondary account
        let secondary_id = wallet.get_account("secondary").unwrap().id();

        // cannot send more than we have
        assert!(wallet
            .default_account_mut()
            .send_value(secondary_id.clone(), 2_000_000)
            .is_err());

        // send 1000 Dusk to secondary account!
        let transaction = wallet
            .default_account_mut()
            .send_value(secondary_id, 1000)
            .unwrap();

        // put the transaction in the queue
        network.queue_transaction(transaction);

        network.mint_block();

        // sync local wallet again
        wallet.sync(&network);

        // Transaction should have taken place
        assert_eq!(wallet.default_account().balance(), 999_000);
        assert_eq!(wallet.get_account("secondary").unwrap().balance(), 1000);
    }
}
