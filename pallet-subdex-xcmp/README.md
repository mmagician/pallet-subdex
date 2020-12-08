# FRAME pallet - token dealer for fungible assets 


## Purpose

This pallet aims to provide functionalities for sending and handling transfer of parachain assets and main network currency between subdex parachain and other parachains/relay chain. 

___ 
## Dependencies

### Traits

This pallet depends on `pallet_subdex::Trait` to perform assets minting and slashing.

### Pallets

[pallet-subdex](https://github.com/subdarkdex/pallet-subdex/tree/master/pallet-subdex)


## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following lines to your runtime's `Cargo.toml` file:

```TOML
pallet-subdex = { git = "https://github.com/subdarkdex/pallet-subdex", default-features = false }
pallet-subdex-xcmp = { git = "https://github.com/subdarkdex/pallet-subdex", default-features = false }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-subdex/std',
    'pallet-subdex-xcmp/std',
]
```
*P.S.* You need to include both pallets, because `pallet-subdex-xcmp` depends on `pallet-subdex`, changing its state, when XCMP triggered.

### Runtime `lib.rs`

You should implement related traits like so, please check up [lib.rs](https://github.com/subdarkdex/subdex-parachain/blob/subdex/runtime/src/lib.rs#L259) for details:

```rust
impl pallet_subdex_xcmp::Trait for Runtime {
    type Event = Event;
    type UpwardMessageSender = MessageBroker;
    type UpwardMessage = cumulus_upward_message::RococoUpwardMessage;
    type XCMPMessageSender = MessageBroker;
}

impl pallet_subdex::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
    type IMoment = u64;
    type AssetId = AssetId;
    type FeeRateNominator = FeeRateNominator;
    type FeeRateDenominator = FeeRateDenominator;
    type MinMainNetworkAssetAmount = MinMainNetworkAssetAmount;
    type MinParachainAssetAmount = MinParachainAssetAmount;
}

```

and include it in your `construct_runtime!` macro:

```rust
DexXCMP: pallet_subdex_xcmp::{Module, Call, Event<T>, Storage, Config<T>},
DexPallet: pallet_subdex::{Module, Config<T>, Call, Storage, Event<T>},
```

### Genesis Configuration example
```rust
 pallet_subdex: Some(DexPalletConfig {
            dex_treasury: DexTreasury::new(root_key, 1, 4),
 }),
 pallet_subdex_xcmp: Some(DexXCMPConfig { next_asset_id: 1 }),
```
## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```

