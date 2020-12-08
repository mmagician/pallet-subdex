use super::*;

#[test]
fn transfer_balance_to_relay_chain() {
    with_test_externalities(|| {
        // Transfer some amount from relay chain here to test if we can withdraw backwards
        let transfer_amount = 15_000;

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), transfer_amount);

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Successfully transfer balance backwards to relay chain
        assert_ok!(emulate_transfer_balance_to_relay_chain(
            FirstAccountId::get(),
            FirstAccountId::get(),
            transfer_amount
        ));

        // Runtime tested state after call
        assert_eq!(Balances::free_balance(FirstAccountId::get()), 0);

        let transferred_balance_to_relay_chain_event = get_subdex_xcmp_test_event(
            RawEvent::TransferredTokensToRelayChain(FirstAccountId::get(), transfer_amount),
        );

        // Last event checked

        // slash() method of Currency trait emits 1 additional event
        assert_event_success(
            transferred_balance_to_relay_chain_event,
            number_of_events_before_call + 2,
        );
    })
}

#[test]
fn transfer_zero_balance_to_relay_chain() {
    with_test_externalities(|| {
        // Transfer some amount from relay chain here to test if we can withdraw backwards
        let transfer_amount = 10000;

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), transfer_amount);

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Make an attempt to transfer zero balance to relay chain
        let transfer_balance_to_relay_chain_result = emulate_transfer_balance_to_relay_chain(
            FirstAccountId::get(),
            FirstAccountId::get(),
            0,
        );

        // Failure checked
        assert_subdex_xcmp_failure(
            transfer_balance_to_relay_chain_result,
            Error::<Test>::AmountShouldBeGreaterThanZero,
            number_of_events_before_call,
        )
    })
}

#[test]
fn transfer_balance_to_relay_chain_not_sufficient_amount() {
    with_test_externalities(|| {
        let transfer_amount = 10000;

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), transfer_amount);

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Make an attempt to transfer main currency from account, which does not have sufficient balance to relay chain
        let transfer_balance_to_relay_chain_result = emulate_transfer_balance_to_relay_chain(
            FirstAccountId::get(),
            FirstAccountId::get(),
            2 * transfer_amount,
        );

        // Failure checked
        assert_subdex_failure(
            transfer_balance_to_relay_chain_result,
            pallet_subdex::Error::<Test>::InsufficientMainNetworkAssetAmount,
            number_of_events_before_call,
        )
    })
}
