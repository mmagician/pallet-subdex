// Copyright 2019-2020
//     by  Centrality Investments Ltd.
//     and Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Mocks for the module.

#![cfg(test)]

pub use super::*;
use cumulus_message_broker;
pub use frame_support::traits::Get;
use frame_support::traits::{OnFinalize, OnInitialize};
pub use frame_support::{assert_err, assert_ok};
use frame_support::{impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

pub use frame_support::dispatch::DispatchResult;
pub use pallet_subdex::{Asset, DexTreasury};
pub use polkadot_core_primitives::AccountId;
use std::cell::RefCell;

impl_outer_origin! {
    pub enum Origin for Test where system = frame_system {}
}
use upward_messages;

pub type AssetId = <Test as pallet_subdex::Trait>::AssetId;
pub type Balance = <Test as pallet_balances::Trait>::Balance;

pub const MILLISECS_PER_BLOCK: u64 = 3000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

thread_local! {
    pub static TREASURY_ACCOUNT_ID: RefCell<AccountId> = RefCell::new([255; 32].into());
    pub static FIRST_ACCOUNT_ID: RefCell<AccountId> = RefCell::new([1; 32].into());
    pub static SECOND_ACCOUNT_ID: RefCell<AccountId> = RefCell::new([2; 32].into());
    pub static FIRST_PARA_ID: RefCell<ParaId> = RefCell::new(300.into());
}

pub struct TreasuryAccountId;
impl Get<AccountId> for TreasuryAccountId {
    fn get() -> AccountId {
        TREASURY_ACCOUNT_ID.with(|v| v.borrow().clone())
    }
}

pub struct FirstAccountId;
impl Get<AccountId> for FirstAccountId {
    fn get() -> AccountId {
        FIRST_ACCOUNT_ID.with(|v| v.borrow().clone())
    }
}

pub struct SecondAccountId;
impl Get<AccountId> for SecondAccountId {
    fn get() -> AccountId {
        SECOND_ACCOUNT_ID.with(|v| v.borrow().clone())
    }
}

pub struct FirstParaId;
impl Get<ParaId> for FirstParaId {
    fn get() -> ParaId {
        FIRST_PARA_ID.with(|v| v.borrow().clone())
    }
}

// Used to get min parachain asset amount, based on its type size, set on node runtime level
pub const fn get_min_parachain_asset_amount() -> Balance {
    match core::mem::size_of::<Balance>() {
        size if size <= 64 => 1000,
        // cosider 112 instead
        size if size > 64 && size < 128 => 100_000,
        _ => 1_000_000,
    }
}

// Used to get min main network asset amount, based on its type size, set on node runtime level
pub const fn get_min_main_network_asset_amount() -> Balance {
    match core::mem::size_of::<Balance>() {
        size if size <= 64 => 10_000,
        // cosider 112 instead
        size if size > 64 && size < 128 => 1_000_000,
        _ => 10_000_000,
    }
}

#[derive(Clone, Eq, Debug, PartialEq)]
pub struct Test;
parameter_types! {
    pub const ExistentialDeposit: Balance = 100;
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Call = ();
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type ModuleToIndex = ();
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Trait for Test {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

#[derive(Encode, Decode)]
pub struct TestUpwardMessage {}
impl upward_messages::BalancesMessage<AccountId, Balance> for TestUpwardMessage {
    fn transfer(_a: AccountId, _b: Balance) -> Self {
        TestUpwardMessage {}
    }
}

impl upward_messages::XCMPMessage for TestUpwardMessage {
    fn send_message(_dest: ParaId, _msg: Vec<u8>) -> Self {
        TestUpwardMessage {}
    }
}

pub struct MessageBrokerMock {}
impl UpwardMessageSender<TestUpwardMessage> for MessageBrokerMock {
    fn send_upward_message(
        _msg: &TestUpwardMessage,
        _origin: UpwardMessageOrigin,
    ) -> Result<(), ()> {
        Ok(())
    }
}

impl XCMPMessageSender<XCMPMessage<AccountId, Balance, AssetId>> for MessageBrokerMock {
    fn send_xcmp_message(
        _dest: ParaId,
        _msg: &XCMPMessage<AccountId, Balance, AssetId>,
    ) -> Result<(), ()> {
        Ok(())
    }
}

impl pallet_balances::Trait for Test {
    type Balance = u128;
    type DustRemoval = ();
    type Event = TestEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = system::Module<Test>;
    type WeightInfo = ();
}

impl Trait for Test {
    type UpwardMessageSender = MessageBrokerMock;
    type UpwardMessage = TestUpwardMessage;
    type XCMPMessageSender = MessageBrokerMock;
    type Event = TestEvent;
}

parameter_types! {
    // 3/1000
    pub const FeeRateNominator: Balance = 3;
    pub const FeeRateDenominator: Balance = 1000;
    pub const MinMainNetworkAssetAmount: Balance = get_min_main_network_asset_amount();
    pub const MinParachainAssetAmount: Balance = get_min_parachain_asset_amount();
}

impl pallet_subdex::Trait for Test {
    type Event = TestEvent;
    type Currency = Balances;
    type IMoment = u64;
    type AssetId = u32;
    type FeeRateNominator = FeeRateNominator;
    type FeeRateDenominator = FeeRateDenominator;
    type MinMainNetworkAssetAmount = MinMainNetworkAssetAmount;
    type MinParachainAssetAmount = MinParachainAssetAmount;
}

mod subdex_xcmp {
    pub use crate::Event;
}

use frame_system as system;
impl_outer_event! {
    pub enum TestEvent for Test {
        system<T>,
        subdex_xcmp<T>,
        pallet_subdex<T>,
        cumulus_message_broker<T>,
        pallet_balances<T>,
    }
}

pub type Balances = pallet_balances::Module<Test>;
pub type SubdexXcmp = Module<Test>;
pub type SubDex = pallet_subdex::Module<Test>;
pub type System = frame_system::Module<Test>;

pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build() -> sp_io::TestExternalities {
        let default_subdex_genesis_config = default_pallet_subdex_genesis_config();
        let default_subdex_xcmp_genesis_config = default_pallet_subdex_xcmp_genesis_config();

        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        default_subdex_genesis_config
            .assimilate_storage(&mut t)
            .unwrap();
        default_subdex_xcmp_genesis_config
            .assimilate_storage(&mut t)
            .unwrap();
        t.into()
    }
}

fn default_pallet_subdex_genesis_config() -> pallet_subdex::GenesisConfig<Test> {
    pallet_subdex::GenesisConfig {
        dex_treasury: DexTreasury::new(TreasuryAccountId::get(), 1, 4),
    }
}

fn default_pallet_subdex_xcmp_genesis_config() -> GenesisConfig<Test> {
    GenesisConfig { next_asset_id: 1 }
}

pub fn with_test_externalities<R, F: FnOnce() -> R>(f: F) -> R {
    /*
        Events are not emitted on block 0.
        So any dispatchable calls made during genesis block formation will have no events emitted.
        https://substrate.dev/recipes/2-appetizers/4-events.html
    */
    let func = || {
        run_to_block(1);
        f()
    };

    ExtBuilder::build().execute_with(func)
}

type SubDexXcmpRawTestEvent = RawEvent<AccountId, Balance, Option<AssetId>, AssetId>;

type SubdexRawTestEvent =
    pallet_subdex::RawEvent<AccountId, Asset<AssetId>, Balance, Balance, Option<Balance>>;

pub fn get_subdex_xcmp_test_event(raw_event: SubDexXcmpRawTestEvent) -> TestEvent {
    TestEvent::subdex_xcmp(raw_event)
}

pub fn get_subdex_test_event(raw_event: SubdexRawTestEvent) -> TestEvent {
    TestEvent::pallet_subdex(raw_event)
}

pub fn assert_event_success(tested_event: TestEvent, number_of_events_after_call: usize) {
    // Ensure  runtime events length is equal to expected number of events after call
    assert_eq!(System::events().len(), number_of_events_after_call);

    // Ensure  last emitted event is equal to expected one
    assert!(matches!(
            System::events()
                .iter()
                .last(),
            Some(last_event) if last_event.event == tested_event
    ));
}

pub fn assert_subdex_xcmp_failure(
    call_result: DispatchResult,
    expected_error: Error<Test>,
    number_of_events_before_call: usize,
) {
    // Ensure  call result is equal to expected error
    assert_err!(call_result, expected_error);

    // Ensure  no other events emitted after call
    assert_eq!(System::events().len(), number_of_events_before_call);
}

pub fn assert_subdex_failure(
    call_result: DispatchResult,
    expected_error: pallet_subdex::Error<Test>,
    number_of_events_before_call: usize,
) {
    // Ensure  call result is equal to expected error
    assert_err!(call_result, expected_error);

    // Ensure  no other events emitted after call
    assert_eq!(System::events().len(), number_of_events_before_call);
}

// Recommendation from Parity on testing on_finalize
// https://substrate.dev/docs/en/next/development/module/tests
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        <System as OnFinalize<u64>>::on_finalize(System::block_number());
        <SubdexXcmp as OnFinalize<u64>>::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        <System as OnInitialize<u64>>::on_initialize(System::block_number());
        <SubdexXcmp as OnInitialize<u64>>::on_initialize(System::block_number());
    }
}
