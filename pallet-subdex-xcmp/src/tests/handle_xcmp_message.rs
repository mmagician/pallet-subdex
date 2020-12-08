use super::*;

#[test]
fn handle_xcmp_message() {
    with_test_externalities(|| {
        let next_asset_id = get_next_asset_id();

        let transfer_amount = 10_000;

        let para_asset_id = None;

        // Runtime tested state before call

        // Ensure given asset id does not exist yet.
        assert!(!asset_id_exists(FirstParaId::get(), para_asset_id));

        assert_eq!(asset_balances(FirstAccountId::get(), next_asset_id), 0);

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Emulate first xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            transfer_amount,
            para_asset_id,
        );

        // Emulate second xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            transfer_amount,
            para_asset_id,
        );

        // Runtime tested state after call

        // Ensure main parachain asset currency balance transferred successfully
        assert_eq!(
            asset_balances(FirstAccountId::get(), next_asset_id),
            2 * transfer_amount
        );

        // Ensure new entry in asset_id_by_para_asset_id mapping created successfully.
        assert_eq!(
            asset_id_by_para_asset_id(FirstParaId::get(), para_asset_id),
            next_asset_id
        );

        // Ensure next_asset_id value incremented
        assert_eq!(next_asset_id + 1, get_next_asset_id());

        let transferred_tokens_from_relay_chain_event =
            get_subdex_xcmp_test_event(RawEvent::DepositAssetViaXCMP(
                FirstParaId::get(),
                para_asset_id,
                FirstAccountId::get(),
                next_asset_id,
                transfer_amount,
            ));

        // Last event checked (number_of_events_before_call is incremented by two, because we received 2 xcmp messages)
        assert_event_success(
            transferred_tokens_from_relay_chain_event,
            number_of_events_before_call + 2,
        );
    })
}
