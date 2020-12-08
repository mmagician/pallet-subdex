use super::*;

#[test]
fn invest_liquidity() {
    with_test_externalities(|| {
        // Transfer both main network currency and custom parachain assets to dex parachain.

        let main_network_currency_transfer_amount = 10_0000;

        let para_asset_transfer_amount = 6_0000;

        let para_asset_id = Some(5);

        // Receive provided amounts for both main network curency and parachain assets through xcmp and use them to initialize exchange
        initialize_simple_exchange(
            FirstAccountId::get(),
            main_network_currency_transfer_amount,
            para_asset_id,
            para_asset_transfer_amount,
        );

        // previosuly mapped parachain asset representation
        let dex_para_asset_id = get_next_asset_id() - 1;

        // An amount of shares to be own by specific actor
        let shares_to_be_own = 100000;

        let exchange = dex_exchanges(
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(get_next_asset_id() - 1),
        );

        // Calculate an amount of both assets, needed to be invested, to own an exact amount of shares.
        let (first_asset_cost, second_asset_cost) =
            exchange.calculate_costs(shares_to_be_own).unwrap();

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), first_asset_cost);

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            second_asset_cost,
            para_asset_id,
        );

        // Runtime tested state before call

        // Events number before tested call
        let number_of_events_before_call = System::events().len();

        // Invest liquidity
        assert_ok!(emulate_invest_liquidity(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(dex_para_asset_id),
            shares_to_be_own
        ));

        // Runtime tested state after call

        // Ensure both main network and parachain asset balances were successfully invested
        assert_eq!(asset_balances(FirstAccountId::get(), dex_para_asset_id), 0);

        assert_eq!(Balances::free_balance(FirstAccountId::get()), 0);

        // Ensure exchanges storage updated successfully
        let (mut newly_created_exchange, _) = Exchange::<Test>::initialize_new(
            main_network_currency_transfer_amount,
            para_asset_transfer_amount,
            FirstAccountId::get(),
        )
        .unwrap();

        let _ = newly_created_exchange.invest(
            first_asset_cost,
            second_asset_cost,
            shares_to_be_own,
            &FirstAccountId::get(),
        );

        assert_eq!(
            newly_created_exchange,
            dex_exchanges(
                Asset::MainNetworkCurrency,
                Asset::ParachainAsset(get_next_asset_id() - 1)
            )
        );

        let exchange_invested_event = get_subdex_test_event(pallet_subdex::RawEvent::Invested(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            Asset::ParachainAsset(dex_para_asset_id),
            shares_to_be_own,
        ));

        // Last event checked
        assert_event_success(
            exchange_invested_event,
            // additional event emitted when Currency slash() performed
            number_of_events_before_call + 2,
        );
    })
}

#[test]
fn invest_liquidity_exchange_does_not_exist() {
    with_test_externalities(|| {
        // Transfer both main network currency and custom parachain assets to dex parachain.

        let main_network_currency_transfer_amount = 10_0000;

        let para_asset_transfer_amount = 6_0000;

        let para_asset_id = Some(5);

        // An amount of shares to be own by specific actor
        let shares_to_be_own = 1000;

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), main_network_currency_transfer_amount);

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            para_asset_transfer_amount,
            para_asset_id,
        );

        // previosuly mapped parachain asset representation
        let dex_para_asset_id = get_next_asset_id() - 1;

        // Runtime tested state before call

        // Events number before tested call
        let number_of_events_before_call = System::events().len();

        // Making attempt o invest liquidity to non existent exchange
        let invest_liquidity_result = emulate_invest_liquidity(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(dex_para_asset_id),
            shares_to_be_own,
        );

        // Failure checked
        assert_subdex_failure(
            invest_liquidity_result,
            pallet_subdex::Error::<Test>::ExchangeNotExists,
            number_of_events_before_call,
        )
    })
}

#[test]
fn invest_liquidity_invalid_exchange() {
    with_test_externalities(|| {
        // Transfer both main network currency and custom parachain assets to dex parachain.

        let main_network_currency_transfer_amount = 10_0000;

        let para_asset_transfer_amount = 6_0000;

        let para_asset_id = Some(5);

        // Receive provided amounts for both main network curency and parachain assets through xcmp and use them to initialize exchange
        initialize_simple_exchange(
            FirstAccountId::get(),
            main_network_currency_transfer_amount,
            para_asset_id,
            para_asset_transfer_amount,
        );

        // An amount of shares to be own by specific actor
        let shares_to_be_own = 100000;

        let exchange = dex_exchanges(
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(get_next_asset_id() - 1),
        );

        // Calculate an amount of both assets, needed to be invested, to own an exact amount of shares.
        let (first_asset_cost, second_asset_cost) =
            exchange.calculate_costs(shares_to_be_own).unwrap();

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), first_asset_cost);

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            second_asset_cost,
            para_asset_id,
        );

        // Runtime tested state before call

        // Events number before tested call
        let number_of_events_before_call = System::events().len();

        // Make an attempt to invest liqudity, providing the same first and second assets.
        let invest_liquidity_result = emulate_invest_liquidity(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            Asset::MainNetworkCurrency,
            shares_to_be_own,
        );

        // Failure checked
        assert_subdex_failure(
            invest_liquidity_result,
            pallet_subdex::Error::<Test>::InvalidExchange,
            number_of_events_before_call,
        )
    })
}

#[test]
fn invest_liquidity_insufficient_parachain_currency_amount() {
    with_test_externalities(|| {
        // Transfer both main network currency and custom parachain assets to dex parachain.

        let main_network_currency_transfer_amount = 10_0000;

        let para_asset_transfer_amount = 6_0000;

        let para_asset_id = Some(5);

        // Receive provided amounts for both main network curency and parachain assets through xcmp and use them to initialize exchange
        initialize_simple_exchange(
            FirstAccountId::get(),
            main_network_currency_transfer_amount,
            para_asset_id,
            para_asset_transfer_amount,
        );

        // previosuly mapped parachain asset representation
        let dex_para_asset_id = get_next_asset_id() - 1;

        // An amount of shares to be own by specific actor
        let shares_to_be_own = 100000;

        let exchange = dex_exchanges(
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(dex_para_asset_id),
        );

        // Calculate an amount of both assets, needed to be invested, to own an exact amount of shares.
        let (first_asset_cost, _) = exchange.calculate_costs(shares_to_be_own).unwrap();

        // Emulate downward message
        emulate_downward_message(FirstAccountId::get(), first_asset_cost);

        // Runtime tested state before call

        // Events number before tested call
        let number_of_events_before_call = System::events().len();

        // Make an attempt to invest liqudity, when account does not have sufficient parachain asset amount
        let invest_liquidity_result = emulate_invest_liquidity(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            Asset::ParachainAsset(dex_para_asset_id),
            shares_to_be_own,
        );

        // Failure checked
        assert_subdex_failure(
            invest_liquidity_result,
            pallet_subdex::Error::<Test>::InsufficientParachainAssetAmount,
            number_of_events_before_call,
        )
    })
}

#[test]
fn invest_liquidity_insufficient_main_network_currency_amount() {
    with_test_externalities(|| {
        // Transfer both main network currency and custom parachain assets to dex parachain.

        let main_network_currency_transfer_amount = 10_0000;

        let para_asset_transfer_amount = 6_0000;

        let para_asset_id = Some(5);

        // Receive provided amounts for both main network curency and parachain assets through xcmp and use them to initialize exchange
        initialize_simple_exchange(
            FirstAccountId::get(),
            main_network_currency_transfer_amount,
            para_asset_id,
            para_asset_transfer_amount,
        );

        // previosuly mapped parachain asset representation
        let dex_para_asset_id = get_next_asset_id() - 1;

        // An amount of shares to be own by specific actor
        let shares_to_be_own = 100000;

        let exchange = dex_exchanges(
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(dex_para_asset_id),
        );

        // Calculate an amount of both assets, needed to be invested, to own an exact amount of shares.
        let (_, second_asset_cost) = exchange.calculate_costs(shares_to_be_own).unwrap();

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            FirstAccountId::get(),
            second_asset_cost,
            para_asset_id,
        );

        // Runtime tested state before call

        // Events number before tested call
        let number_of_events_before_call = System::events().len();

        // Make an attempt to invest liqudity, when account does not have sufficient main network currency amount
        let invest_liquidity_result = emulate_invest_liquidity(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            Asset::ParachainAsset(dex_para_asset_id),
            shares_to_be_own,
        );

        // Failure checked
        assert_subdex_failure(
            invest_liquidity_result,
            pallet_subdex::Error::<Test>::InsufficientMainNetworkAssetAmount,
            number_of_events_before_call,
        )
    })
}
