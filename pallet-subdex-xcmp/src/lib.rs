#![cfg_attr(not(feature = "std"), no_std)]

pub mod upward_messages;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, traits::Currency};
use frame_system::ensure_signed;

use codec::{Codec, Decode, Encode};
pub use cumulus_primitives::{
    relay_chain::DownwardMessage,
    xcmp::{XCMPMessageHandler, XCMPMessageSender},
    DownwardMessageHandler, ParaId, UpwardMessageOrigin, UpwardMessageSender,
};

pub use crate::upward_messages::BalancesMessage;
pub use pallet_subdex::Asset;
pub use sp_arithmetic::traits::{One, Zero};

/// Used to represent xcmp message type
#[derive(Encode, Decode)]
pub enum XCMPMessage<XAccountId, XBalance, XAssetIdOf> {
    /// Transfer tokens to the given account from the Parachain account.
    /// When XAssetIdOf is None, treat message as main currency transfer.
    TransferToken(XAccountId, XBalance, Option<XAssetIdOf>),
}

/// Type, used for dex assets balances representation
pub type BalanceOf<T> = <<T as pallet_subdex::Trait>::Currency as Currency<
    <T as frame_system::Trait>::AccountId,
>>::Balance;

/// Type, used for representation of assets, located on other parachains (both internal and remote).
pub type AssetIdOf<T> = <T as pallet_subdex::Trait>::AssetId;

/// Configuration trait of this pallet.
pub trait Trait: frame_system::Trait + pallet_subdex::Trait {
    /// Event type used by the runtime.
    type Event: From<Event<Self>>
        + Into<<Self as frame_system::Trait>::Event>
        + Into<<Self as pallet_subdex::Trait>::Event>;

    /// The sender of upward messages.
    type UpwardMessageSender: UpwardMessageSender<Self::UpwardMessage>;

    /// The upward message type used by the Parachain runtime.
    type UpwardMessage: Codec + BalancesMessage<Self::AccountId, BalanceOf<Self>>;

    /// The sender of XCMP messages.
    type XCMPMessageSender: XCMPMessageSender<
        XCMPMessage<Self::AccountId, BalanceOf<Self>, AssetIdOf<Self>>,
    >;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ParachainUpgrade {

        // Maps parachain asset id to our internal respresentation
        pub AssetIdByParaAssetId get(fn asset_id_by_para_asset_id):
            double_map hasher(blake2_128_concat) ParaId, hasher(blake2_128_concat) Option<AssetIdOf<T>> => AssetIdOf<T>;

        // Next dex parachain asset id
        pub NextAssetId get(fn next_asset_id) config(): AssetIdOf<T>;
    }
}

decl_event! {
    pub enum Event<T> where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>,
        // None if main currency
        ParaChainAssetId = Option<AssetIdOf<T>>,
        // Our internal para asset id representation
        DexAssetId = AssetIdOf<T>

    {
        /// Transferred main currency amount to the account on the relay chain.
        TransferredTokensToRelayChain(AccountId, Balance),

        /// Transferred main currency amount  to the account on request from the relay chain.
        TransferredTokensFromRelayChain(AccountId, Balance),

        /// Transferred custom asset to the account from the given parachain account.
        DepositAssetViaXCMP(ParaId, ParaChainAssetId, AccountId, DexAssetId, Balance),

        /// Transferred custom asset to the account from the given parachain account.
        WithdrawAssetViaXCMP(ParaId, ParaChainAssetId, AccountId, DexAssetId, Balance),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin, system = frame_system {

        fn deposit_event() = default;

        /// Transfer `amount` of main currency on the relay chain to
        /// the given `dest` account.
        #[weight = 10]
        fn transfer_balance_to_relay_chain(origin, dest: T::AccountId, amount: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;

            // Ensure provided balance amount is greater than zero
            Self::ensure_non_zero_balance(amount)?;

            // Ensure account has sufficient balance to withdraw
            <pallet_subdex::Module<T>>::ensure_sufficient_balance(&sender, Asset::MainNetworkCurrency, amount)?;

            //
            // == MUTATION SAFE ==
            //

            // Slash account asset balance to perform withdraw
            <pallet_subdex::Module<T>>::slash_asset(&sender, Asset::MainNetworkCurrency, amount);


            // Send upward transfer message
            let msg = <T as Trait>::UpwardMessage::transfer(dest.clone(), amount);
            <T as Trait>::UpwardMessageSender::send_upward_message(&msg, UpwardMessageOrigin::Signed)
                .expect("Should not fail; qed");

            Self::deposit_event(Event::<T>::TransferredTokensToRelayChain(dest, amount));
        }

        /// Transfer a given `amount` of another parachain asset to another parachain.
        #[weight = 10]
        fn transfer_asset_balance_to_parachain_chain(
            origin,
            para_id: u32,
            dest: T::AccountId,
            para_asset_id: Option<AssetIdOf<T>>,
            amount: BalanceOf<T>,
        ) {

            //TODO make sure that the parachain has some tokens on the other parachain.
            let who = ensure_signed(origin)?;

            // Ensure provided balance amount is greater than zero
            Self::ensure_non_zero_balance(amount)?;

            let para_id: ParaId = para_id.into();

            // Retreive our internal para asset id representation
            let asset_id = Self::ensure_asset_id_exists(para_id, para_asset_id)?;

            // Ensure account has sufficient balance perform withdraw to another parachain
            <pallet_subdex::Module<T>>::ensure_sufficient_balance(&who, Asset::ParachainAsset(asset_id), amount)?;

            //
            // == MUTATION SAFE ==
            //

            // Slash balance of account inernal parachain asset representation to perform withdraw
            <pallet_subdex::Module<T>>::slash_asset(&who, Asset::ParachainAsset(asset_id), amount);

            // Send xcmp transfer message
            T::XCMPMessageSender::send_xcmp_message(
                para_id,
                &XCMPMessage::TransferToken(dest.clone(), amount, para_asset_id),
            ).expect("Should not fail; qed");

            Self::deposit_event(Event::<T>::WithdrawAssetViaXCMP(para_id, para_asset_id, dest, asset_id, amount));
        }

    }
}

/// This is a hack to convert from one generic type to another where we are sure that both are the
/// same type/use the same encoding.
fn convert_hack<O: Decode>(input: &impl Encode) -> O {
    input.using_encoded(|e| Decode::decode(&mut &e[..]).expect("Must be compatible; qed"))
}

impl<T: Trait> DownwardMessageHandler for Module<T> {
    /// Transfer main network asset into dex parachain from the relay chain (natively supported via Currency trait)
    fn handle_downward_message(msg: &DownwardMessage) {
        if let DownwardMessage::TransferInto(dest, amount, _) = msg {
            let dest = convert_hack(&dest);
            let amount: BalanceOf<T> = convert_hack(amount);

            // Ensure account can hold given balance after deposit from relay chain performed
            <pallet_subdex::Module<T>>::ensure_can_hold_balance(
                &dest,
                Asset::MainNetworkCurrency,
                amount,
            )
            .expect("Should not fail!");

            //
            // == MUTATION SAFE ==
            //

            // Mint respective asset amount to given account to complete funds deposit
            <pallet_subdex::Module<T>>::mint_asset(&dest, Asset::MainNetworkCurrency, amount);

            Self::deposit_event(Event::<T>::TransferredTokensFromRelayChain(dest, amount));
        }
    }
}

impl<T: Trait> XCMPMessageHandler<XCMPMessage<T::AccountId, BalanceOf<T>, AssetIdOf<T>>>
    for Module<T>
{
    /// Transfer main currency or custom asset from other parachain to our chain
    fn handle_xcmp_message(
        src: ParaId,
        msg: &XCMPMessage<T::AccountId, BalanceOf<T>, AssetIdOf<T>>,
    ) {
        let asset_id = match msg {
            XCMPMessage::TransferToken(dest, amount, para_asset_id)
                if <AssetIdByParaAssetId<T>>::contains_key(src, para_asset_id) =>
            {
                // Retreive internal dex asset id representation
                let asset_id = Self::asset_id_by_para_asset_id(src, para_asset_id);

                // Ensure account can hold given balance after deposit from another parachain performed
                <pallet_subdex::Module<T>>::ensure_can_hold_balance(
                    &dest,
                    Asset::ParachainAsset(asset_id),
                    *amount,
                )
                .expect("Should not fail!");
                Some(asset_id)
            }
            _ => None,
        };

        //
        // == MUTATION SAFE ==
        //

        match msg {
            XCMPMessage::TransferToken(dest, amount, para_asset_id) => {
                if let Some(asset_id) = asset_id {
                    // Mint respective asset amount to given account to complete deposit from another parachain
                    <pallet_subdex::Module<T>>::mint_asset(
                        &dest,
                        Asset::ParachainAsset(asset_id),
                        *amount,
                    );
                    Self::deposit_event(Event::<T>::DepositAssetViaXCMP(
                        src,
                        // para asset_id
                        *para_asset_id,
                        dest.clone(),
                        // internal asset id representation
                        asset_id,
                        *amount,
                    ));
                } else {
                    // Create new (para id, para asset id) -> internal asset id maping entry to keep track of new assets, coming from another parachains
                    let next_asset_id = Self::next_asset_id();
                    <AssetIdByParaAssetId<T>>::insert(src, *para_asset_id, next_asset_id);

                    // Mint respective asset amount to given account to complete deposit from another parachain
                    <pallet_subdex::Module<T>>::mint_asset(
                        &dest,
                        Asset::ParachainAsset(next_asset_id),
                        *amount,
                    );

                    <NextAssetId<T>>::mutate(|asset_id| *asset_id += AssetIdOf::<T>::one());

                    Self::deposit_event(Event::<T>::DepositAssetViaXCMP(
                        src,
                        // para asset_id
                        *para_asset_id,
                        dest.clone(),
                        // internal asset id representation
                        next_asset_id,
                        *amount,
                    ));
                }
            }
        }
    }
}

impl<T: Trait> Module<T> {
    /// Ensure asset under given id exists
    pub fn ensure_asset_id_exists(
        para_id: ParaId,
        para_asset_id: Option<AssetIdOf<T>>,
    ) -> Result<AssetIdOf<T>, Error<T>> {
        ensure!(
            <AssetIdByParaAssetId<T>>::contains_key(para_id, para_asset_id),
            Error::<T>::AssetIdDoesNotExist
        );
        Ok(Self::asset_id_by_para_asset_id(para_id, para_asset_id))
    }

    // Ensure provided balance amount is greater than zero
    pub fn ensure_non_zero_balance(amount: BalanceOf<T>) -> Result<(), Error<T>> {
        ensure!(
            amount > BalanceOf::<T>::zero(),
            Error::<T>::AmountShouldBeGreaterThanZero
        );
        Ok(())
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Transferred amount should be greater than 0
        AmountShouldBeGreaterThanZero,
        /// Given parachain asset id entry does not exist
        AssetIdDoesNotExist,
    }
}
