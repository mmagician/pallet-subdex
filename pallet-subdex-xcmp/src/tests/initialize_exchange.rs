use super::*;

#[test]
fn initialize_exchange() {
    with_test_externalities(|| {
        // Transfer both main network currency and custom parachain assets to dex parachain.

        let main_network_currency_transfer_amount = 10_000;

        let para_asset_transfer_amount = 6_000;

        let para_asset_id = Some(5);

        let asset_id = get_next_asset_id();

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            para_asset_transfer_amount,
            para_asset_id,
        );

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), main_network_currency_transfer_amount);

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Initialize new exchange
        assert_ok!(initialize_new_exchange(
            FirstAccountId::get(),
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(asset_id),
            para_asset_transfer_amount,
            Asset::MainNetworkCurrency,
            main_network_currency_transfer_amount
        ));

        // Runtime tested state after call

        // Ensure both main network and parachain asset balances were successfully invested
        assert_eq!(asset_balances(FirstAccountId::get(), asset_id), 0);

        assert_eq!(Balances::free_balance(FirstAccountId::get()), 0);

        // Ensure exchanges storage updated successfully
        let (newly_created_exchange, initial_shares) = Exchange::<Test>::initialize_new(
            main_network_currency_transfer_amount,
            para_asset_transfer_amount,
            FirstAccountId::get(),
        )
        .unwrap();

        assert_eq!(
            newly_created_exchange,
            dex_exchanges(Asset::MainNetworkCurrency, Asset::ParachainAsset(asset_id))
        );

        let exchange_initialized_event =
            get_subdex_test_event(pallet_subdex::RawEvent::Initialized(
                FirstAccountId::get(),
                Asset::MainNetworkCurrency,
                Asset::ParachainAsset(asset_id),
                initial_shares,
            ));

        // Last event checked
        assert_event_success(
            exchange_initialized_event,
            // additional event emitted when Currency slash() performed
            number_of_events_before_call + 2,
        );
    })
}

#[test]
fn initialize_invalid_exchange() {
    with_test_externalities(|| {
        // Transfer both main network currency and custom parachain assets to dex parachain.

        let transfer_amount = 10_000;

        let para_asset_id = Some(5);

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            transfer_amount,
            para_asset_id,
        );

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), transfer_amount);

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Make an attempt to initialize exchange, providing the same first and second assets
        let initialize_new_exchange_result = initialize_new_exchange(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            transfer_amount / 2,
            Asset::MainNetworkCurrency,
            transfer_amount / 2,
        );

        // Failure checked
        assert_subdex_failure(
            initialize_new_exchange_result,
            pallet_subdex::Error::<Test>::InvalidExchange,
            number_of_events_before_call,
        )
    })
}

#[test]
fn initialize_exchange_main_network_asset_amount_below_min() {
    with_test_externalities(|| {
        let main_network_currency_transfer_amount = get_min_main_network_asset_amount() - 1;

        let para_asset_transfer_amount = get_min_parachain_asset_amount();

        let para_asset_id = Some(5);

        let asset_id = get_next_asset_id();

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            para_asset_transfer_amount,
            para_asset_id,
        );

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), main_network_currency_transfer_amount);

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Make an attempt to initialize exchange, providing main network asset amount, which is below minimual for this operation
        let initialize_new_exchange_result = initialize_new_exchange(
            FirstAccountId::get(),
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(asset_id),
            para_asset_transfer_amount,
            Asset::MainNetworkCurrency,
            main_network_currency_transfer_amount,
        );

        // Failure checked
        assert_subdex_failure(
            initialize_new_exchange_result,
            pallet_subdex::Error::<Test>::MainNetworkAssetAmountBelowMin,
            number_of_events_before_call,
        )
    })
}

#[test]
fn initialize_exchange_parachain_asset_amount_below_min() {
    with_test_externalities(|| {
        let main_network_currency_transfer_amount = get_min_main_network_asset_amount();

        let para_asset_transfer_amount = get_min_parachain_asset_amount() - 1;

        let para_asset_id = Some(5);

        let asset_id = get_next_asset_id();

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            para_asset_transfer_amount,
            para_asset_id,
        );

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), main_network_currency_transfer_amount);

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Make an attempt to initialize exchange, providing parachain asset amount, which is below minimual for this operation
        let initialize_new_exchange_result = initialize_new_exchange(
            FirstAccountId::get(),
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(asset_id),
            para_asset_transfer_amount,
            Asset::MainNetworkCurrency,
            main_network_currency_transfer_amount,
        );

        // Failure checked
        assert_subdex_failure(
            initialize_new_exchange_result,
            pallet_subdex::Error::<Test>::ParachainAssetAmountBelowMin,
            number_of_events_before_call,
        )
    })
}

#[test]
fn initialize_exchange_already_exists() {
    with_test_externalities(|| {
        let main_network_currency_transfer_amount = get_min_main_network_asset_amount();

        let para_asset_transfer_amount = get_min_parachain_asset_amount();

        let para_asset_id = Some(5);

        let asset_id = get_next_asset_id();

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            para_asset_transfer_amount,
            para_asset_id,
        );

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), main_network_currency_transfer_amount);

        // Initialize new exchange
        assert_ok!(initialize_new_exchange(
            FirstAccountId::get(),
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(asset_id),
            para_asset_transfer_amount,
            Asset::MainNetworkCurrency,
            main_network_currency_transfer_amount
        ));

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Make an attempt to reinitialize already existing exchange
        let initialize_new_exchange_result = initialize_new_exchange(
            FirstAccountId::get(),
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(asset_id),
            para_asset_transfer_amount,
            Asset::MainNetworkCurrency,
            main_network_currency_transfer_amount,
        );

        // Failure checked
        assert_subdex_failure(
            initialize_new_exchange_result,
            pallet_subdex::Error::<Test>::ExchangeAlreadyExists,
            number_of_events_before_call,
        )
    })
}

#[test]
fn initialize_exchange_insufficient_main_network_asset_amount() {
    with_test_externalities(|| {
        let main_network_currency_transfer_amount = get_min_main_network_asset_amount();

        let para_asset_transfer_amount = get_min_parachain_asset_amount();

        let para_asset_id = Some(5);

        let asset_id = get_next_asset_id();

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            para_asset_transfer_amount,
            para_asset_id,
        );

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), main_network_currency_transfer_amount);

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Make an attempt to initialize exchange, when origin does not have a sufficient main network asset amount
        let initialize_new_exchange_result = initialize_new_exchange(
            FirstAccountId::get(),
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(asset_id),
            para_asset_transfer_amount,
            Asset::MainNetworkCurrency,
            2 * main_network_currency_transfer_amount,
        );

        // Failure checked
        assert_subdex_failure(
            initialize_new_exchange_result,
            pallet_subdex::Error::<Test>::InsufficientMainNetworkAssetAmount,
            number_of_events_before_call,
        )
    })
}

#[test]
fn initialize_exchange_insufficient_parachain_asset_amount() {
    with_test_externalities(|| {
        let main_network_currency_transfer_amount = get_min_main_network_asset_amount();

        let para_asset_transfer_amount = get_min_parachain_asset_amount();

        let para_asset_id = Some(5);

        let asset_id = get_next_asset_id();

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            para_asset_transfer_amount,
            para_asset_id,
        );

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), main_network_currency_transfer_amount);

        // Runtime tested state before call

        // Events number before tested calls
        let number_of_events_before_call = System::events().len();

        // Make an attempt to initialize exchange, when origin does not have a sufficient parachain asset amount
        let initialize_new_exchange_result = initialize_new_exchange(
            FirstAccountId::get(),
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(asset_id),
            2 * para_asset_transfer_amount,
            Asset::MainNetworkCurrency,
            main_network_currency_transfer_amount,
        );

        // Failure checked
        assert_subdex_failure(
            initialize_new_exchange_result,
            pallet_subdex::Error::<Test>::InsufficientParachainAssetAmount,
            number_of_events_before_call,
        )
    })
}
