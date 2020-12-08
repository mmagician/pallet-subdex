use super::*;
use sp_runtime::traits::IntegerSquareRoot;

/// Structure, used to represent exchange pool
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct Exchange<T: Trait> {
    // first asset pool
    first_asset_pool: BalanceOf<T>,
    // second asset pool
    second_asset_pool: BalanceOf<T>,
    // first and second asset pool invariant
    pub invariant: BalanceOf<T>,
    // total pool shares
    pub total_shares: BalanceOf<T>,
    // last timestamp, after pool update performed, needed for time_elapsed calculation
    pub last_timestamp: T::IMoment,
    // first_asset_pool / second_asset_pool * time_elapsed
    pub price1_cumulative_last: BalanceOf<T>,
    // second_asset_pool / first_asset_pool * time_elapsed
    pub price2_cumulative_last: BalanceOf<T>,
    // individual shares
    shares: BTreeMap<T::AccountId, BalanceOf<T>>,
}

impl<T: Trait> Default for Exchange<T> {
    fn default() -> Self {
        Self {
            first_asset_pool: BalanceOf::<T>::default(),
            second_asset_pool: BalanceOf::<T>::default(),
            invariant: BalanceOf::<T>::default(),
            total_shares: BalanceOf::<T>::default(),
            last_timestamp: T::IMoment::default(),
            price1_cumulative_last: BalanceOf::<T>::default(),
            price2_cumulative_last: BalanceOf::<T>::default(),
            shares: BTreeMap::new(),
        }
    }
}

/// Structure, used to represent first and second asset pools after exchange performed and asset amount, based on swap direction
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct SwapDelta<T: Trait> {
    // new first asset pool amount
    pub first_asset_pool: BalanceOf<T>,
    // new second asset pool amount
    pub second_asset_pool: BalanceOf<T>,
    // Either first or second asset amount (depends on swap direction)
    pub amount: BalanceOf<T>,
}

impl<T: Trait> SwapDelta<T> {
    pub fn new(
        first_asset_pool: BalanceOf<T>,
        second_asset_pool: BalanceOf<T>,
        amount: BalanceOf<T>,
    ) -> Self {
        Self {
            first_asset_pool,
            second_asset_pool,
            amount,
        }
    }
}

impl<T: Trait> Exchange<T> {
    // Calculate min fee, used to substract from initial shares amount, based on balances type size set
    fn get_min_fee() -> BalanceOf<T> {
        match core::mem::size_of::<BalanceOf<T>>() {
            size if size <= 64 => 1.into(),
            // cosider 112 instead
            size if size > 64 && size < 128 => 10.into(),
            _ => (10 * 10 * 10).into(),
        }
    }

    /// Initialize new exchange
    pub fn initialize_new(
        first_asset_amount: BalanceOf<T>,
        second_asset_amount: BalanceOf<T>,
        sender: T::AccountId,
    ) -> Result<(Self, BalanceOf<T>), Error<T>> {
        let mut shares_map = BTreeMap::new();
        let min_fee = Self::get_min_fee();

        // Calculate initial shares amount, based on formula
        let initial_shares = first_asset_amount
            .checked_mul(&second_asset_amount)
            .map(|result| result.integer_sqrt_checked())
            .flatten()
            // substract min fee amount
            .map(|sqrt_result| sqrt_result.checked_sub(&min_fee))
            .flatten()
            .ok_or(Error::<T>::UnderflowOccured)?;

        shares_map.insert(sender, initial_shares);
        let exchange = Self {
            first_asset_pool: first_asset_amount,
            second_asset_pool: second_asset_amount,
            invariant: first_asset_amount
                .checked_mul(&second_asset_amount)
                .ok_or(Error::<T>::UnderflowOrOverflowOccured)?,
            total_shares: initial_shares,
            shares: shares_map,
            last_timestamp: <pallet_timestamp::Module<T>>::get().into(),
            price1_cumulative_last: BalanceOf::<T>::default(),
            price2_cumulative_last: BalanceOf::<T>::default(),
        };
        Ok((exchange, initial_shares))
    }

    // Calculate first to second asset swap delta
    fn perform_first_to_second_asset_swap_calculation(
        &self,
        exchange_fee: BalanceOf<T>,
        first_asset_amount: BalanceOf<T>,
    ) -> Result<SwapDelta<T>, Error<T>> {
        let new_first_asset_pool = self
            .first_asset_pool
            .checked_add(&first_asset_amount)
            .ok_or(Error::<T>::OverflowOccured)?;

        let temp_first_asset_pool = new_first_asset_pool
            .checked_sub(&exchange_fee)
            .ok_or(Error::<T>::UnderflowOccured)?;

        let new_second_asset_pool = self
            .invariant
            .checked_div(&temp_first_asset_pool)
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

        let second_asset_amount = self
            .second_asset_pool
            .checked_sub(&new_second_asset_pool)
            .ok_or(Error::<T>::UnderflowOccured)?;

        Ok(SwapDelta::new(
            new_first_asset_pool,
            new_second_asset_pool,
            second_asset_amount,
        ))
    }

    /// Calculate first to second asset swap delta and treasury fee (if enabled)
    pub fn calculate_first_to_second_asset_swap(
        &self,
        first_asset_amount: BalanceOf<T>,
    ) -> Result<(SwapDelta<T>, Option<(BalanceOf<T>, T::AccountId)>), Error<T>> {
        let fee = T::FeeRateNominator::get()
            .checked_mul(&first_asset_amount)
            .map(|result| result.checked_div(&T::FeeRateDenominator::get()))
            .flatten()
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

        if let Ok(dex_treasury) = <DEXTreasury<T>>::try_get() {
            let treasury_fee = dex_treasury
                .treasury_fee_rate_nominator
                .checked_mul(&fee)
                .map(|result| result.checked_div(&dex_treasury.treasury_fee_rate_denominator))
                .flatten()
                .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

            let exchange_fee = fee - treasury_fee;

            let swap_delta = self
                .perform_first_to_second_asset_swap_calculation(exchange_fee, first_asset_amount)?;

            Ok((swap_delta, Some((treasury_fee, dex_treasury.dex_account))))
        } else {
            let swap_delta =
                self.perform_first_to_second_asset_swap_calculation(fee, first_asset_amount)?;
            Ok((swap_delta, None))
        }
    }

    // Calculate second to first asset swap delta
    fn perform_second_to_first_asset_swap_calculation(
        &self,
        exchange_fee: BalanceOf<T>,
        second_asset_amount: BalanceOf<T>,
    ) -> Result<SwapDelta<T>, Error<T>> {
        let new_second_asset_pool = self
            .second_asset_pool
            .checked_add(&second_asset_amount)
            .ok_or(Error::<T>::OverflowOccured)?;

        let temp_second_asset_pool = new_second_asset_pool
            .checked_sub(&exchange_fee)
            .ok_or(Error::<T>::UnderflowOccured)?;

        let new_first_asset_pool = self
            .invariant
            .checked_div(&temp_second_asset_pool)
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

        let first_asset_amount = self
            .first_asset_pool
            .checked_sub(&new_first_asset_pool)
            .ok_or(Error::<T>::UnderflowOccured)?;

        Ok(SwapDelta::new(
            new_first_asset_pool,
            new_second_asset_pool,
            first_asset_amount,
        ))
    }

    /// Calculate second to first asset swap delta and treasury fee (if enabled)
    pub fn calculate_second_to_first_asset_swap(
        &self,
        second_asset_amount: BalanceOf<T>,
    ) -> Result<(SwapDelta<T>, Option<(BalanceOf<T>, T::AccountId)>), Error<T>> {
        let fee = T::FeeRateNominator::get()
            .checked_mul(&second_asset_amount)
            .map(|result| result.checked_div(&T::FeeRateDenominator::get()))
            .flatten()
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

        if let Ok(dex_treasury) = <DEXTreasury<T>>::try_get() {
            let treasury_fee = dex_treasury
                .treasury_fee_rate_nominator
                .checked_mul(&fee)
                .map(|result| result.checked_div(&dex_treasury.treasury_fee_rate_denominator))
                .flatten()
                .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

            let exchange_fee = fee - treasury_fee;

            let swap_delta = self.perform_second_to_first_asset_swap_calculation(
                exchange_fee,
                second_asset_amount,
            )?;

            Ok((swap_delta, Some((treasury_fee, dex_treasury.dex_account))))
        } else {
            let swap_delta =
                self.perform_second_to_first_asset_swap_calculation(fee, second_asset_amount)?;

            Ok((swap_delta, None))
        }
    }

    /// Calculate costs for both first and second currencies, needed to get a given amount of shares
    pub fn calculate_costs(
        &self,
        shares: BalanceOf<T>,
    ) -> Result<(BalanceOf<T>, BalanceOf<T>), Error<T>> {
        let first_asset_cost = shares
            .checked_mul(&self.first_asset_pool)
            .map(|result| result.checked_div(&self.total_shares))
            .flatten()
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

        let second_asset_cost = shares
            .checked_mul(&self.second_asset_pool)
            .map(|result| result.checked_div(&self.total_shares))
            .flatten()
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

        Ok((first_asset_cost, second_asset_cost))
    }

    /// Perform invest operation
    pub fn invest(
        &mut self,
        first_asset_amount: BalanceOf<T>,
        second_asset_amount: BalanceOf<T>,
        shares: BalanceOf<T>,
        sender: &T::AccountId,
    ) -> Result<(), Error<T>> {
        let updated_shares = if let Some(prev_shares) = self.shares.get(sender) {
            prev_shares
                .checked_add(&shares)
                .ok_or(Error::<T>::OverflowOccured)?
        } else {
            shares
        };

        self.shares.insert(sender.clone(), updated_shares);

        self.total_shares = self
            .total_shares
            .checked_add(&shares)
            .ok_or(Error::<T>::OverflowOccured)?;

        self.first_asset_pool = self
            .first_asset_pool
            .checked_add(&first_asset_amount)
            .ok_or(Error::<T>::OverflowOccured)?;

        self.second_asset_pool = self
            .second_asset_pool
            .checked_add(&second_asset_amount)
            .ok_or(Error::<T>::OverflowOccured)?;

        self.invariant = self
            .first_asset_pool
            .checked_mul(&self.second_asset_pool)
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;
        Ok(())
    }

    /// Perform divest operation
    pub fn divest(
        &mut self,
        first_asset_amount: BalanceOf<T>,
        second_asset_amount: BalanceOf<T>,
        shares: BalanceOf<T>,
        sender: &T::AccountId,
    ) -> Result<(), Error<T>> {
        if let Some(share) = self.shares.get_mut(sender) {
            *share = share
                .checked_sub(&shares)
                .ok_or(Error::<T>::UnderflowOccured)?;
        }

        self.total_shares = self
            .total_shares
            .checked_sub(&shares)
            .ok_or(Error::<T>::UnderflowOccured)?;

        self.first_asset_pool = self
            .first_asset_pool
            .checked_sub(&first_asset_amount)
            .ok_or(Error::<T>::UnderflowOccured)?;

        self.second_asset_pool = self
            .second_asset_pool
            .checked_sub(&second_asset_amount)
            .ok_or(Error::<T>::UnderflowOccured)?;

        if self.total_shares == BalanceOf::<T>::zero() {
            self.invariant = BalanceOf::<T>::zero();
        } else {
            self.invariant = self
                .first_asset_pool
                .checked_mul(&self.second_asset_pool)
                .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;
        }
        Ok(())
    }

    /// Update exchange liquidity pools with amounts provided, update cumulative price data
    pub fn update_pools(
        &mut self,
        first_asset_pool: BalanceOf<T>,
        second_asset_pool: BalanceOf<T>,
    ) -> Result<(), Error<T>> {
        self.first_asset_pool = first_asset_pool;
        self.second_asset_pool = second_asset_pool;

        let now: T::IMoment = <pallet_timestamp::Module<T>>::get().into();
        let time_elapsed: T::IMoment = now
            .checked_sub(&self.last_timestamp)
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

        let price1_cumulative = first_asset_pool
            .checked_div(&second_asset_pool)
            .map(|result| result.checked_mul(&time_elapsed.into()))
            .flatten()
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

        self.price1_cumulative_last = self
            .price1_cumulative_last
            .checked_add(&price1_cumulative)
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

        let price2_cumulative = second_asset_pool
            .checked_div(&first_asset_pool)
            .map(|result| result.checked_mul(&time_elapsed.into()))
            .flatten()
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

        self.price2_cumulative_last = self
            .price2_cumulative_last
            .checked_add(&price2_cumulative)
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;

        self.last_timestamp = now;

        self.invariant = self
            .first_asset_pool
            .checked_mul(&self.second_asset_pool)
            .ok_or(Error::<T>::UnderflowOrOverflowOccured)?;
        Ok(())
    }

    /// Ensure new liquidity pool can be launched successfully
    pub fn ensure_launch(&self) -> dispatch::DispatchResult {
        ensure!(
            self.invariant == BalanceOf::<T>::zero(),
            Error::<T>::InvariantNotNull
        );
        ensure!(
            self.total_shares == BalanceOf::<T>::zero(),
            Error::<T>::TotalSharesNotNull
        );
        Ok(())
    }

    /// Ensure second asset amount is available for withdraw
    pub fn ensure_second_asset_amount(
        &self,
        second_asset_out_amount: BalanceOf<T>,
        min_asset_out_amount: BalanceOf<T>,
    ) -> dispatch::DispatchResult {
        // Ensure second asset out amount is greater than min expected asset out amount
        ensure!(
            second_asset_out_amount >= min_asset_out_amount,
            Error::<T>::SecondAssetAmountBelowExpectation
        );
        // Ensure second  asset out amount is less than total available in the pool
        ensure!(
            second_asset_out_amount <= self.second_asset_pool,
            Error::<T>::InsufficientPool
        );
        Ok(())
    }

    /// Perform all necessary cheks to ensure that given amount of shares can be burned succesfully
    pub fn ensure_burned_shares(
        &self,
        sender: &T::AccountId,
        shares_burned: BalanceOf<T>,
    ) -> dispatch::DispatchResult {
        ensure!(
            shares_burned > BalanceOf::<T>::zero(),
            Error::<T>::InvalidShares
        );
        if let Some(shares) = self.shares.get(sender) {
            ensure!(*shares >= shares_burned, Error::<T>::InsufficientShares);
            Ok(())
        } else {
            Err(Error::<T>::DoesNotOwnShare.into())
        }
    }

    /// Ensure first asset amount is available for withdraw
    pub fn ensure_first_asset_amount(
        &self,
        first_asset_out_amount: BalanceOf<T>,
        min_first_asset_out_amount: BalanceOf<T>,
    ) -> dispatch::DispatchResult {
        // Ensure first asset out amount is greater than min expected asset out amount
        ensure!(
            first_asset_out_amount >= min_first_asset_out_amount,
            Error::<T>::SecondAssetAmountBelowExpectation
        );
        // Ensure first asset out amount is less than total available in the pool
        ensure!(
            first_asset_out_amount <= self.first_asset_pool,
            Error::<T>::InsufficientPool
        );
        Ok(())
    }
}
