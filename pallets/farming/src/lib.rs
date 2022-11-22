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
use frame_system::pallet_prelude::*;
use primitives::Balance;
use sp_runtime::{
	traits::{AccountIdConversion, UniqueSaturatedFrom},
	DispatchResult,
};
use sp_std::prelude::*;

pub use pallet::*;
pub mod weights;
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type FungibleTokenIdOf<T> = <T as pallet_token_fungible::Config>::FungibleTokenId;
type FungibleTokenId = u128;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Pool<AccountId, BlockNumber> {
	pub escrow_account: AccountId,
	pub start_at: BlockNumber,
	pub staking_duration: BlockNumber,
	pub locked_duration: BlockNumber,
	pub locked_token_id: FungibleTokenId,
	pub award_token_id: FungibleTokenId,
	pub total_locked: Balance,
	pub total_award: Balance,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct StakingInfo {
	pub staking_balance: Balance,
	pub is_claimed: bool,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum Status {
	StakingNotStart,
	Staking,
	Locked,
	Claim,
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
		StakingNotStart,
		StakingTimeout,
		CurrentStakingTime,
		CurrentLockedTime,
		CurrentClaimTime,
		ClaimNotStart,
		AlreadyClaim,
		NotStaking,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated(u64),
		Staking(T::AccountId, u64, Balance),
		Claim(T::AccountId, u64, Balance, Balance),
	}

	/// The pallet admin key.
	#[pallet::storage]
	#[pallet::getter(fn admin)]
	pub type Admin<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn next_pool_id)]
	pub type NextPoolId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub type Pools<T: Config> = StorageMap<_, Blake2_128, u64, Pool<T::AccountId, T::BlockNumber>>;

	#[pallet::storage]
	#[pallet::getter(fn account_pool_id_locked)]
	pub type AccountPoolIdLocked<T: Config> =
		StorageMap<_, Blake2_128, (T::AccountId, u64), StakingInfo>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_admin())]
		pub fn set_admin(origin: OriginFor<T>, new_admin: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Admin::<T>::mutate(|admin| *admin = Some(new_admin));
			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_pool())]
		pub fn create_pool(
			origin: OriginFor<T>,
			start_at: T::BlockNumber,
			staking_duration: T::BlockNumber,
			locked_duration: T::BlockNumber,
			locked_token_id: FungibleTokenId,
			award_token_id: FungibleTokenId,
			total_award: Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_admin(sender.clone()), Error::<T>::NoPermisson);
			let pool_id = NextPoolId::<T>::try_mutate(|id| -> Result<u64, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(1u64).ok_or(Error::<T>::NoAvailablePoolId)?;
				Ok(current_id)
			})?;

			let escrow_account = Self::escrow_account_id(pool_id);

			pallet_token_fungible::Pallet::<T>::do_transfer(
				FungibleTokenIdOf::<T>::unique_saturated_from(award_token_id),
				&sender,
				&escrow_account,
				total_award,
			)?;

			Pools::<T>::insert(
				pool_id,
				Pool {
					escrow_account,
					start_at,
					staking_duration,
					locked_duration,
					locked_token_id,
					award_token_id,
					total_locked: 0,
					total_award,
				},
			);

			Self::deposit_event(Event::PoolCreated(pool_id));

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::staking())]
		pub fn staking(origin: OriginFor<T>, pool_id: u64, amount: Balance) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			match Self::pool_status(Self::now(), &pool) {
				Status::StakingNotStart => ensure!(false, Error::<T>::StakingNotStart),
				Status::Staking => {},
				Status::Locked => ensure!(false, Error::<T>::CurrentLockedTime),
				Status::Claim => ensure!(false, Error::<T>::CurrentClaimTime),
			};

			pallet_token_fungible::Pallet::<T>::do_transfer(
				FungibleTokenIdOf::<T>::unique_saturated_from(pool.locked_token_id),
				&sender,
				&pool.escrow_account,
				amount,
			)?;

			if AccountPoolIdLocked::<T>::get((sender.clone(), pool_id)).is_none() {
				AccountPoolIdLocked::<T>::insert(
					(sender.clone(), pool_id),
					StakingInfo { staking_balance: amount, is_claimed: false },
				);
			} else {
				AccountPoolIdLocked::<T>::mutate((sender.clone(), pool_id), |old_staking_info| {
					if let Some(staking_info) = old_staking_info {
						staking_info.staking_balance =
							staking_info.staking_balance.saturating_add(amount);
					}
				});
			}

			Pools::<T>::mutate(pool_id, |old_pool| {
				if let Some(op) = old_pool {
					op.total_locked = op.total_locked.saturating_add(amount);
				}
			});

			Self::deposit_event(Event::Staking(sender, pool_id, amount));

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::claim())]
		pub fn claim(origin: OriginFor<T>, pool_id: u64) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			match Self::pool_status(Self::now(), &pool) {
				Status::StakingNotStart => ensure!(false, Error::<T>::StakingNotStart),
				Status::Staking => ensure!(false, Error::<T>::CurrentStakingTime),
				Status::Locked => ensure!(false, Error::<T>::CurrentLockedTime),
				Status::Claim => {},
			};

			let pool_id_locked = AccountPoolIdLocked::<T>::get((sender.clone(), pool_id))
				.ok_or(Error::<T>::NotStaking)?;

			ensure!(pool_id_locked.is_claimed == false, Error::<T>::AlreadyClaim);

			pallet_token_fungible::Pallet::<T>::do_transfer(
				FungibleTokenIdOf::<T>::unique_saturated_from(pool.locked_token_id),
				&pool.escrow_account,
				&sender,
				pool_id_locked.staking_balance,
			)?;

			AccountPoolIdLocked::<T>::mutate((sender.clone(), pool_id), |old_staking_info| {
				if let Some(staking_info) = old_staking_info {
					staking_info.is_claimed = true;
				}
			});

			let award = pool_id_locked.staking_balance * pool.total_award / pool.total_locked;

			pallet_token_fungible::Pallet::<T>::do_transfer(
				FungibleTokenIdOf::<T>::unique_saturated_from(pool.award_token_id),
				&pool.escrow_account,
				&sender,
				award,
			)?;

			Self::deposit_event(Event::Claim(
				sender,
				pool_id,
				pool_id_locked.staking_balance,
				award,
			));

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::force_claim())]
		pub fn force_claim(
			origin: OriginFor<T>,
			traget: T::AccountId,
			pool_id: u64,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_admin(sender.clone()), Error::<T>::NoPermisson);

			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			let pool_id_locked = AccountPoolIdLocked::<T>::get((traget.clone(), pool_id))
				.ok_or(Error::<T>::NotStaking)?;

			ensure!(pool_id_locked.is_claimed == false, Error::<T>::AlreadyClaim);

			pallet_token_fungible::Pallet::<T>::do_transfer(
				FungibleTokenIdOf::<T>::unique_saturated_from(pool.locked_token_id),
				&pool.escrow_account,
				&traget,
				pool_id_locked.staking_balance,
			)?;

			AccountPoolIdLocked::<T>::remove((traget, pool_id));

			Pools::<T>::mutate(pool_id, |old_pool| {
				if let Some(op) = old_pool {
					op.total_locked =
						op.total_locked.saturating_sub(pool_id_locked.staking_balance);
				}
			});

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn is_admin(sender: T::AccountId) -> bool {
		matches!(Admin::<T>::get(), Some(admin) if admin == sender)
	}

	fn now() -> T::BlockNumber {
		frame_system::Pallet::<T>::block_number()
	}

	pub fn escrow_account_id(pool_id: u64) -> T::AccountId {
		<T as pallet::Config>::PalletId::get().into_sub_account_truncating(pool_id)
	}

	pub fn pool_status(now: T::BlockNumber, pool: &Pool<T::AccountId, T::BlockNumber>) -> Status {
		let start_at = pool.start_at;
		let staking_end_time = pool.start_at + pool.staking_duration;
		let locked_end_time = pool.start_at + pool.staking_duration + pool.locked_duration;
		match now {
			now if now < start_at => Status::StakingNotStart,
			now if start_at <= now && now < staking_end_time => Status::Staking,
			now if staking_end_time <= now && now < locked_end_time => Status::Locked,
			_ => Status::Claim,
		}
	}
}
