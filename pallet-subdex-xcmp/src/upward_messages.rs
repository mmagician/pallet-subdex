use sp_std::vec::Vec;

use polkadot_core_primitives::{AccountId, Balance};
use polkadot_parachain::primitives::Id as ParaId;
use rococo_runtime::{BalancesCall, ParachainsCall};
// mod kusama;
// mod polkadot;
// mod westend;

/// A `Balances` related upward message.
pub trait BalancesMessage<AccountId, Balance>: Sized {
    /// Transfer the given `amount` from the parachain account to the given
    /// `dest` account.
    fn transfer(dest: AccountId, amount: Balance) -> Self;
}

/// A `XCMP` related upward message.
pub trait XCMPMessage: Sized {
    /// Send the given XCMP message to given parachain.
    fn send_message(dest: ParaId, msg: Vec<u8>) -> Self;
}

/// The Rococo upward message.
pub type RococoUpwardMessage = rococo_runtime::Call;

impl BalancesMessage<AccountId, Balance> for RococoUpwardMessage {
    fn transfer(dest: AccountId, amount: Balance) -> Self {
        BalancesCall::transfer(dest, amount).into()
    }
}

impl XCMPMessage for RococoUpwardMessage {
    fn send_message(dest: ParaId, msg: Vec<u8>) -> Self {
        ParachainsCall::send_xcmp_message(dest, msg).into()
    }
}
