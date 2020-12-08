use super::*;

#[test]
fn transfer_balance_to_parachain_chain() {
    with_test_externalities(|| {
        let next_asset_id = get_next_asset_id();

        let transfer_amount = 10_000;

        let para_asset_id = Some(5);

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            transfer_amount,
            para_asset_id,
        );

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Successfully transfer balance backwards to parachain chain
        assert_ok!(emulate_transfer_asset_balance_to_parachain_chain(
            FirstAccountId::get(),
            FirstParaId::get(),
            FirstAccountId::get(),
            para_asset_id,
            transfer_amount
        ));

        // Runtime tested state after call

        // Ensure parachain asset balance transferred backwards successfully
        assert_eq!(asset_balances(FirstAccountId::get(), next_asset_id), 0);

        let transferred_balance_to_parachain_chain_event =
            get_subdex_xcmp_test_event(RawEvent::WithdrawAssetViaXCMP(
                FirstParaId::get(),
                para_asset_id,
                FirstAccountId::get(),
                next_asset_id,
                transfer_amount,
            ));

        // Last event checked
        assert_event_success(
            transferred_balance_to_parachain_chain_event,
            number_of_events_before_call + 1,
        );
    })
}

#[test]
fn transfer_zero_balance_to_parachain_chain() {
    with_test_externalities(|| {
        let transfer_amount = 10_000;

        let para_asset_id = Some(5);

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            transfer_amount,
            para_asset_id,
        );

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Make an attempt to transfer zero balance backwards to parachain chain
        let transfer_balance_to_parachain_chain_result =
            emulate_transfer_asset_balance_to_parachain_chain(
                FirstAccountId::get(),
                FirstParaId::get(),
                FirstAccountId::get(),
                para_asset_id,
                0,
            );

        // Failure checked
        assert_subdex_xcmp_failure(
            transfer_balance_to_parachain_chain_result,
            Error::<Test>::AmountShouldBeGreaterThanZero,
            number_of_events_before_call,
        )
    })
}

#[test]
fn transfer_balance_to_parachain_chain_not_sufficient_amount() {
    with_test_externalities(|| {
        let transfer_amount = 10_000;

        let para_asset_id = Some(5);

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            transfer_amount,
            para_asset_id,
        );

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Make an attempt to transfer parachain currency from account, which does not have sufficient balance to parachain chain
        let transfer_balance_to_parachain_chain_result =
            emulate_transfer_asset_balance_to_parachain_chain(
                FirstAccountId::get(),
                FirstParaId::get(),
                FirstAccountId::get(),
                para_asset_id,
                2 * transfer_amount,
            );

        // Failure checked
        assert_subdex_failure(
            transfer_balance_to_parachain_chain_result,
            pallet_subdex::Error::<Test>::InsufficientParachainAssetAmount,
            number_of_events_before_call,
        )
    })
}

#[test]
fn transfer_balance_to_parachain_chain_asset_does_not_exist() {
    with_test_externalities(|| {
        let transfer_amount = 10_000;

        let para_asset_id = Some(5);

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Make an attempt to transfer asset, which respective entry does not exist in AssetIdByParaAssetId mapping yet
        let transfer_balance_to_parachain_chain_result =
            emulate_transfer_asset_balance_to_parachain_chain(
                FirstAccountId::get(),
                FirstParaId::get(),
                FirstAccountId::get(),
                para_asset_id,
                transfer_amount,
            );

        // Failure checked
        assert_subdex_xcmp_failure(
            transfer_balance_to_parachain_chain_result,
            Error::<Test>::AssetIdDoesNotExist,
            number_of_events_before_call,
        )
    })
}
