// This file is part of Web3Games.

// Copyright (C) 2021-2022 Web3Games https://web3games.org
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, PalletId};
use frame_system::{pallet_prelude::*, WeightInfo};
use primitives::Balance;
use sp_runtime::{
	traits::{AccountIdConversion, UniqueSaturatedFrom},
	DispatchResult,
};
use sp_std::prelude::*;
use pallet_support::FungibleMetadata;

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type FungibleTokenIdOf<T> = <T as pallet_token_fungible::Config>::FungibleTokenId;
type FungibleTokenId = u128;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Pool<AccountId, BlockNumber> {
	pub escrow_account: AccountId,
	pub sale_start: BlockNumber,
	pub sale_end: BlockNumber,
	pub sale_token_id: FungibleTokenId,
	pub buy_token_id: FungibleTokenId,
	pub token_price: Balance,
	pub total_sale_amount: Balance,
	pub raise_amount: Balance,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ClaimInfo {
	pub balance: Balance,
	pub is_claimed: bool,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_token_fungible::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;

		/// This pallet id.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		NoAvailablePoolId,
		NoPermisson,
		ArithmeticOverflow,
		PoolNotFound,
		OutOfSaleTime,
		ClaimNotStart,
		AlreadyClaim,
		NotBuy,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated(u64),
		BuyToken(T::AccountId, u64, Balance),
		Claim(T::AccountId, u64, Balance),
	}

	#[pallet::storage]
	#[pallet::getter(fn next_pool_id)]
	pub type NextPoolId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub type Pools<T: Config> = StorageMap<_, Blake2_128, u64, Pool<T::AccountId, T::BlockNumber>>;

	#[pallet::storage]
	#[pallet::getter(fn account_pool_id_locked)]
	pub type AccountPoolIdLocked<T: Config> =
	StorageMap<_, Blake2_128, (T::AccountId, u64), ClaimInfo>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(2000)]
		pub fn create_pool(
			origin: OriginFor<T>,
			sale_start: T::BlockNumber,
			sale_duration: T::BlockNumber,
			sale_token_id: FungibleTokenId,
			buy_token_id: FungibleTokenId,
			total_sale_amount: Balance,
			token_price: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let pool_id = NextPoolId::<T>::try_mutate(|id| -> Result<u64, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(1u64).ok_or(Error::<T>::NoAvailablePoolId)?;
				Ok(current_id)
			})?;

			let escrow_account = Self::escrow_account_id(pool_id);

			pallet_token_fungible::Pallet::<T>::do_transfer(
				FungibleTokenIdOf::<T>::unique_saturated_from(sale_token_id),
				&sender,
				&escrow_account,
				total_sale_amount,
			)?;

			Pools::<T>::insert(
				pool_id,
				Pool {
					escrow_account,
					sale_start,
					sale_end: sale_start + sale_duration,
					sale_token_id,
					buy_token_id,
					token_price,
					total_sale_amount,
					raise_amount:total_sale_amount
				},
			);

			Self::deposit_event(Event::PoolCreated(pool_id));

			Ok(())
		}

		#[pallet::weight(2000)]
		pub fn buy_token(origin: OriginFor<T>, pool_id: u64, amount: Balance) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			ensure!(Self::now() >= pool.sale_start && Self::now() < pool.sale_end, Error::<T>::OutOfSaleTime);

			let pay_amount = amount.saturating_mul(pool.token_price);

			//check balance
			pallet_token_fungible::Pallet::<T>::do_transfer(
				FungibleTokenIdOf::<T>::unique_saturated_from(pool.buy_token_id),
				&sender,
				&pool.escrow_account,
				pay_amount,
			)?;

			let decimals = pallet_token_fungible::Pallet::<T>::token_decimals(
				FungibleTokenIdOf::<T>::unique_saturated_from(pool.sale_token_id)
			);

			let claim_amount = amount.saturating_mul(10u128.pow(decimals as u32));

			if AccountPoolIdLocked::<T>::get((sender.clone(), pool_id)).is_none() {
				AccountPoolIdLocked::<T>::insert(
					(sender.clone(), pool_id),
					ClaimInfo { balance: claim_amount, is_claimed: false },
				);
			} else {
				AccountPoolIdLocked::<T>::mutate((sender.clone(), pool_id), |old_claim_info| {
					if let Some(claim_info) = old_claim_info {
						claim_info.balance =
							claim_info.balance.saturating_add(claim_amount);
					}
				});
			}

			Pools::<T>::mutate(pool_id, |old_pool| {
				if let Some(op) = old_pool {
					op.raise_amount = op.raise_amount.saturating_sub(claim_amount);
				}
			});

			Self::deposit_event(Event::BuyToken(sender, pool_id, claim_amount));

			Ok(())
		}

		#[pallet::weight(2000)]
		pub fn claim(origin: OriginFor<T>, pool_id: u64) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			ensure!(Self::now() > pool.sale_end, Error::<T>::ClaimNotStart);

			let claim_info = AccountPoolIdLocked::<T>::get((sender.clone(), pool_id))
				.ok_or(Error::<T>::NotBuy)?;

			ensure!(claim_info.is_claimed == false, Error::<T>::AlreadyClaim);

			pallet_token_fungible::Pallet::<T>::do_transfer(
				FungibleTokenIdOf::<T>::unique_saturated_from(pool.sale_token_id),
				&pool.escrow_account,
				&sender,
				claim_info.balance,
			)?;

			AccountPoolIdLocked::<T>::mutate((sender.clone(), pool_id), |old_claim_info| {
				if let Some(claim_info) = old_claim_info {
					claim_info.is_claimed = true;
				}
			});

			Self::deposit_event(Event::Claim(
				sender,
				pool_id,
				claim_info.balance,
			));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn now() -> T::BlockNumber {
		frame_system::Pallet::<T>::block_number()
	}

	pub fn escrow_account_id(pool_id: u64) -> T::AccountId {
		<T as pallet::Config>::PalletId::get().into_sub_account_truncating(pool_id)
	}
}
