use super::*;

#[test]
fn handle_downward_message() {
    with_test_externalities(|| {
        // Runtime tested state before call

        assert_eq!(Balances::free_balance(FirstAccountId::get()), 0);

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        let transfer_amount = 15_000;

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), transfer_amount);

        // Runtime tested state after call

        // Ensure main network currency balance transferred successfully
        assert_eq!(
            Balances::free_balance(FirstAccountId::get()),
            transfer_amount
        );

        let transferred_tokens_from_relay_chain_event = get_subdex_xcmp_test_event(
            RawEvent::TransferredTokensFromRelayChain(FirstAccountId::get(), transfer_amount),
        );

        // Last event checked

        // deposit_creating() method of Currency trait emits 2 additional events
        assert_event_success(
            transferred_tokens_from_relay_chain_event,
            number_of_events_before_call + 3,
        );
    })
}
