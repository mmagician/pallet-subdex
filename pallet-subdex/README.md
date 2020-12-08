# pallet-subdex


## Purpose

This pallet implements core subdex exchange funtionalities 

___ 
## Dependencies

### Traits

Pallet depends on `pallet_timestamp::Trait` to perform cumulative prices calcuation.

`Currency` trait used to provide an interface over fungible asset system implementation, specified on node runtime level.
Currently we use it to handle main network currency native support.


## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following line to your runtime's `Cargo.toml` file:

```TOML
pallet-subdex = { git = "https://github.com/subdarkdex/pallet-subdex", default-features = false }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-subdex/std',
]
```

### Runtime `lib.rs`

You should implement related traits like so, please check up [lib.rs](https://github.com/subdarkdex/subdex-parachain/blob/subdex/runtime/src/lib.rs#L294) for details:

```rust
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
DexPallet: pallet_subdex::{Module, Config<T>, Call, Storage, Event<T>},
```

### Genesis Configuration example
```rust
 pallet_subdex: Some(DexPalletConfig {
      dex_treasury: DexTreasury::new(root_key, 1, 4),
 })
```
## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```

