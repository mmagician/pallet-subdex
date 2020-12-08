use super::*;

#[test]
fn divest_liquidity() {
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

        let exchange = dex_exchanges(
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(get_next_asset_id() - 1),
        );

        let shares = exchange.total_shares;

        // Calculate an amount of both assets, needed to be divested, to extract an exact amount of shares.
        let (first_asset_cost, second_asset_cost) = exchange.calculate_costs(shares).unwrap();

        // Runtime tested state before call

        // Events number before tested call
        let number_of_events_before_call = System::events().len();

        // Divest all available liquidity
        assert_ok!(emulate_divest_liquidity(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(dex_para_asset_id),
            shares,
            first_asset_cost,
            second_asset_cost
        ));

        // Runtime tested state after call

        // Ensure both main network and parachain asset balances were successfully divested
        assert_eq!(
            asset_balances(FirstAccountId::get(), dex_para_asset_id),
            second_asset_cost
        );

        assert_eq!(
            Balances::free_balance(FirstAccountId::get()),
            first_asset_cost
        );

        // Ensure exchanges storage updated successfully
        let (mut newly_created_exchange, _) = Exchange::<Test>::initialize_new(
            main_network_currency_transfer_amount,
            para_asset_transfer_amount,
            FirstAccountId::get(),
        )
        .unwrap();

        let _ = newly_created_exchange.divest(
            first_asset_cost,
            second_asset_cost,
            shares,
            &FirstAccountId::get(),
        );

        assert_eq!(
            newly_created_exchange,
            dex_exchanges(
                Asset::MainNetworkCurrency,
                Asset::ParachainAsset(get_next_asset_id() - 1)
            )
        );

        let exchange_invested_event = get_subdex_test_event(pallet_subdex::RawEvent::Divested(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            Asset::ParachainAsset(dex_para_asset_id),
            shares,
        ));

        // Last event checked
        assert_event_success(
            exchange_invested_event,
            // additional events emitted when Currency deposit_creating() method performed
            number_of_events_before_call + 3,
        );
    })
}

#[test]
fn divest_liquidity_invalid_exchange() {
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

        let exchange = dex_exchanges(
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(get_next_asset_id() - 1),
        );

        let shares = exchange.total_shares;

        // Calculate an amount of both assets, needed to be divested, to extract an exact amount of shares.
        let (first_asset_cost, second_asset_cost) = exchange.calculate_costs(shares).unwrap();

        // Runtime tested state before call

        // Events number before tested call
        let number_of_events_before_call = System::events().len();

        // Make an attempt to divest liqudity, providing the same first and second assets.
        let divest_liquidity_result = emulate_divest_liquidity(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            Asset::MainNetworkCurrency,
            shares,
            first_asset_cost,
            second_asset_cost,
        );

        // Failure checked
        assert_subdex_failure(
            divest_liquidity_result,
            pallet_subdex::Error::<Test>::InvalidExchange,
            number_of_events_before_call,
        )
    })
}

#[test]
fn divest_liquidity_exchange_does_not_exist() {
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

        // Make an attempt to divest liqudity from non existent exchange
        let divest_liquidity_result = emulate_divest_liquidity(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            Asset::ParachainAsset(dex_para_asset_id),
            shares_to_be_own,
            main_network_currency_transfer_amount,
            para_asset_transfer_amount,
        );

        // Failure checked
        assert_subdex_failure(
            divest_liquidity_result,
            pallet_subdex::Error::<Test>::ExchangeNotExists,
            number_of_events_before_call,
        )
    })
}

#[test]
fn divest_liquidity_does_not_own_share() {
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

        let exchange = dex_exchanges(
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(dex_para_asset_id),
        );

        let shares = exchange.total_shares;

        // Calculate an amount of both assets, needed to be divested, to extract an exact amount of shares.
        let (first_asset_cost, second_asset_cost) = exchange.calculate_costs(shares).unwrap();

        // Runtime tested state before call

        // Events number before tested call
        let number_of_events_before_call = System::events().len();

        // Make an attempt to divest liqudity, using an account, which does not own any share.
        let divest_liquidity_result = emulate_divest_liquidity(
            SecondAccountId::get(),
            Asset::MainNetworkCurrency,
            Asset::ParachainAsset(dex_para_asset_id),
            shares,
            first_asset_cost,
            second_asset_cost,
        );

        // Failure checked
        assert_subdex_failure(
            divest_liquidity_result,
            pallet_subdex::Error::<Test>::DoesNotOwnShare,
            number_of_events_before_call,
        )
    })
}

#[test]
fn divest_liquidity_insufficient_shares() {
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
        let (first_asset_cost, second_asset_cost) =
            exchange.calculate_costs(shares_to_be_own).unwrap();

        // Emulate downward message
        emulate_downward_message(SecondAccountId::get(), first_asset_cost);

        // Emulate xcmp message
        emulate_xcmp_message(
            FirstParaId::get(),
            SecondAccountId::get(),
            second_asset_cost,
            para_asset_id,
        );

        // Use second account to invest liquidity
        assert_ok!(emulate_invest_liquidity(
            SecondAccountId::get(),
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(dex_para_asset_id),
            shares_to_be_own
        ));

        let exchange_after_invest_performed = dex_exchanges(
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(dex_para_asset_id),
        );

        let total_shares = exchange_after_invest_performed.total_shares;

        // Calculate an amount of both assets, needed to be divested, to extract an exact amount of shares.
        let (first_asset_cost, second_asset_cost) = exchange_after_invest_performed
            .calculate_costs(total_shares)
            .unwrap();

        // Runtime tested state before call

        // Events number before tested call
        let number_of_events_before_call = System::events().len();

        // Make an attempt to divest liqudity, using an account, which does not own sufficient amount of share.
        let divest_liquidity_result = emulate_divest_liquidity(
            SecondAccountId::get(),
            Asset::MainNetworkCurrency,
            Asset::ParachainAsset(dex_para_asset_id),
            total_shares,
            first_asset_cost,
            second_asset_cost,
        );

        // Failure checked
        assert_subdex_failure(
            divest_liquidity_result,
            pallet_subdex::Error::<Test>::InsufficientShares,
            number_of_events_before_call,
        )
    })
}

#[test]
fn divest_liquidity_first_asset_amount_below_expectation() {
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

        let exchange = dex_exchanges(
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(dex_para_asset_id),
        );

        let shares = exchange.total_shares;

        // Calculate an amount of both assets, needed to be divested, to extract an exact amount of shares.
        let (first_asset_cost, second_asset_cost) = exchange.calculate_costs(shares).unwrap();

        // Runtime tested state before call

        // Events number before tested call
        let number_of_events_before_call = System::events().len();

        // Make an attempt to divest liqudity, providing min first asset amount, which does not satify divest expectations.
        let divest_liquidity_result = emulate_divest_liquidity(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            Asset::ParachainAsset(dex_para_asset_id),
            shares,
            2 * first_asset_cost,
            second_asset_cost,
        );

        // Failure checked
        assert_subdex_failure(
            divest_liquidity_result,
            pallet_subdex::Error::<Test>::FirstAssetAmountBelowExpectation,
            number_of_events_before_call,
        )
    })
}

#[test]
fn divest_liquidity_second_asset_amount_below_expectation() {
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

        let exchange = dex_exchanges(
            Asset::MainNetworkCurrency,
            // previosuly mapped parachain asset representation
            Asset::ParachainAsset(dex_para_asset_id),
        );

        let shares = exchange.total_shares;

        // Calculate an amount of both assets, needed to be divested, to extract an exact amount of shares.
        let (first_asset_cost, second_asset_cost) = exchange.calculate_costs(shares).unwrap();

        // Runtime tested state before call

        // Events number before tested call
        let number_of_events_before_call = System::events().len();

        // Make an attempt to divest liqudity, providing min second asset amount, which does not satify divest expectations.
        let divest_liquidity_result = emulate_divest_liquidity(
            FirstAccountId::get(),
            Asset::MainNetworkCurrency,
            Asset::ParachainAsset(dex_para_asset_id),
            shares,
            first_asset_cost,
            2 * second_asset_cost,
        );

        // Failure checked
        assert_subdex_failure(
            divest_liquidity_result,
            pallet_subdex::Error::<Test>::SecondAssetAmountBelowExpectation,
            number_of_events_before_call,
        )
    })
}
