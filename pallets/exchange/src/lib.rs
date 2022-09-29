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

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{Currency, ExistenceRequirement::AllowDeath, Get, Randomness, ReservableCurrency},
	PalletId,
};
use integer_sqrt::IntegerSquareRoot;
use primitives::Balance;
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::{
	traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, One, Zero},
	RuntimeDebug,
};
use sp_std::{cmp, prelude::*};

pub use pallet::*;
pub mod weights;
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type FungibleTokenIdOf<T> = <T as pallet_token_fungible::Config>::FungibleTokenId;

pub const MINIMUM_LIQUIDITY: u128 = 1000; // 10**3;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Pool<AccountId, FungibleTokenId> {
	/// The id of first token
	pub token_0: FungibleTokenId,
	/// The id of second token
	pub token_1: FungibleTokenId,
	/// The id of liquidity pool token
	pub lp_token: FungibleTokenId,
	/// The id of liquidity pool token
	pub lp_token_account_id: AccountId,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, transactional};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_token_fungible::Config + pallet_wrap_currency::Config
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type PalletId: Get<PalletId>;

		type PoolId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// The minimum balance to create pool
		#[pallet::constant]
		type CreatePoolDeposit: Get<BalanceOf<Self>>;

		/// The minimum balance to create pool
		#[pallet::constant]
		type WW3G: Get<FungibleTokenIdOf<Self>>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// runtime weights.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub(super) type Pools<T: Config> = StorageMap<
		_,
		Blake2_128,
		(T::FungibleTokenId, T::FungibleTokenId),
		Pool<T::AccountId, T::FungibleTokenId>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_pool_id)]
	pub(super) type NextPoolId<T: Config> = StorageValue<_, T::PoolId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn lp_token_to_token)]
	pub(super) type LpTokenToToken<T: Config> =
		StorageMap<_, Blake2_128, T::FungibleTokenId, (T::FungibleTokenId, T::FungibleTokenId)>;

	#[pallet::storage]
	#[pallet::getter(fn reserves)]
	pub(super) type Reserves<T: Config> = StorageMap<
		_,
		Blake2_128,
		(T::FungibleTokenId, T::FungibleTokenId),
		(Balance, Balance),
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type KLast<T: Config> =
		StorageMap<_, Blake2_128, (T::FungibleTokenId, T::FungibleTokenId), Balance, ValueQuery>;

	#[pallet::storage]
	pub(super) type FeeTo<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated(T::PoolId, T::FungibleTokenId, T::FungibleTokenId, T::AccountId),
		LiquidityAdded(T::FungibleTokenId, Balance, Balance, Balance),
		LiquidityRemoved(T::FungibleTokenId, Balance, Balance, Balance),
		Swap(T::AccountId, Balance, Balance, Balance, Balance, T::AccountId),
		SetFeeTo(T::AccountId),
		Burn(T::AccountId, Balance, Balance, T::AccountId),
		Mint(T::AccountId, Balance, Balance, T::AccountId),
		Sync(Balance, Balance),
	}

	#[pallet::error]
	pub enum Error<T> {
		Overflow,
		PoolNotFound,
		NoAvailablePoolId,
		InsufficientAmount,
		InsufficientOutAmount,
		InsufficientInputAmount,
		InsufficientOutputAmount,
		InsufficientLiquidity,
		AdjustedError,
		InsufficientAAmount,
		InsufficientBAmount,
		InsufficientLiquidityMinted,
		InsufficientLiquidityBurned,
		InvalidPath,
		TokenAccountNotFound,
		PoolAlreadyCreated,
		TokenRepeat,
		Deadline,
		VecToU128Failed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			token_a: T::FungibleTokenId,
			token_b: T::FungibleTokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(token_a != token_b, Error::<T>::TokenRepeat);
			ensure!(
				pallet_token_fungible::Pallet::<T>::exists(token_a) &&
					pallet_token_fungible::Pallet::<T>::exists(token_b),
				Error::<T>::TokenAccountNotFound,
			);

			ensure!(!Self::exists(token_a, token_b), Error::<T>::PoolAlreadyCreated);

			Self::do_create_pool(who, token_a, token_b)?;

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			token_a: T::FungibleTokenId,
			token_b: T::FungibleTokenId,
			#[pallet::compact] amount_a_desired: Balance,
			#[pallet::compact] amount_b_desired: Balance,
			#[pallet::compact] amount_a_min: Balance,
			#[pallet::compact] amount_b_min: Balance,
			to: T::AccountId,
			#[pallet::compact] deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(deadline > Self::now(), Error::<T>::Deadline);

			let (token_0, token_1) = Self::sort_tokens(token_a, token_b);
			let amount_0;
			let amount_1;
			let amount_0_min;
			let amount_1_min;
			if token_a == token_0 {
				amount_0 = amount_a_desired;
				amount_1 = amount_b_desired;
				amount_0_min = amount_a_min;
				amount_1_min = amount_b_min;
			} else {
				amount_0 = amount_b_desired;
				amount_1 = amount_a_desired;
				amount_0_min = amount_b_min;
				amount_1_min = amount_a_min;
			}

			let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;

			let (amount_a, amount_b) = Self::do_add_liquidity(
				token_0,
				token_1,
				amount_0,
				amount_1,
				amount_0_min,
				amount_1_min,
			)?;

			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.token_0,
				&who,
				&pool.lp_token_account_id,
				amount_a,
			)?;
			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.token_1,
				&who,
				&pool.lp_token_account_id,
				amount_b,
			)?;
			let liquidity = Self::mint(who, token_0, token_1, to)?;
			//
			Self::deposit_event(Event::LiquidityAdded(
				pool.lp_token,
				amount_a,
				amount_b,
				liquidity,
			));

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn add_liquidity_w3g(
			origin: OriginFor<T>,
			token: T::FungibleTokenId,
			#[pallet::compact] amount_w3g_desired: Balance,
			#[pallet::compact] amount_desired: Balance,
			#[pallet::compact] amount_w3g_min: Balance,
			#[pallet::compact] amount_min: Balance,
			to: T::AccountId,
			#[pallet::compact] deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(deadline > Self::now(), Error::<T>::Deadline);

			pallet_wrap_currency::Pallet::<T>::do_deposit(who.clone(), amount_w3g_desired)?;

			let (token_0, token_1) = Self::sort_tokens(T::WW3G::get(), token);

			let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;

			let (amount_a, amount_b) = Self::do_add_liquidity(
				T::WW3G::get(),
				token,
				amount_w3g_desired,
				amount_desired,
				amount_w3g_min,
				amount_min,
			)?;

			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.token_0,
				&who,
				&pool.lp_token_account_id,
				amount_a,
			)?;
			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.token_1,
				&who,
				&pool.lp_token_account_id,
				amount_b,
			)?;
			let liquidity = Self::mint(who, token_0, token_1, to)?;
			//
			Self::deposit_event(Event::LiquidityAdded(
				pool.lp_token,
				amount_a,
				amount_b,
				liquidity,
			));

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::remove_liquidity())]
		#[transactional]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			token_a: T::FungibleTokenId,
			token_b: T::FungibleTokenId,
			#[pallet::compact] liquidity: Balance,
			#[pallet::compact] amount_a_min: Balance,
			#[pallet::compact] amount_b_min: Balance,
			to: T::AccountId,
			#[pallet::compact] deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(deadline > Self::now(), Error::<T>::Deadline);

			let (token_0, token_1) = Self::sort_tokens(token_a, token_b);
			let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;

			// return Lp to Pallet
			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.lp_token,
				&who,
				&pool.lp_token_account_id,
				liquidity,
			)?;

			let (amount_0, amount_1) = Self::burn(who, token_0, token_1, to)?;

			ensure!(amount_0 >= amount_a_min, Error::<T>::InsufficientAAmount);
			ensure!(amount_1 >= amount_b_min, Error::<T>::InsufficientBAmount);
			Self::deposit_event(Event::LiquidityRemoved(
				pool.lp_token,
				amount_0,
				amount_1,
				liquidity,
			));

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::remove_liquidity())]
		#[transactional]
		pub fn remove_liquidity_w3g(
			origin: OriginFor<T>,
			token: T::FungibleTokenId,
			#[pallet::compact] liquidity: Balance,
			#[pallet::compact] amount_w3g_min: Balance,
			#[pallet::compact] amount_min: Balance,
			to: T::AccountId,
			#[pallet::compact] deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(deadline > Self::now(), Error::<T>::Deadline);

			let (token_0, token_1) = Self::sort_tokens(T::WW3G::get(), token);
			let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;

			// return Lp to Pallet
			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.lp_token,
				&who,
				&pool.lp_token_account_id,
				liquidity,
			)?;

			let (amount_0, amount_1) = Self::burn(who.clone(), token_0, token_1, to)?;
			ensure!(amount_0 >= amount_w3g_min, Error::<T>::InsufficientAAmount);
			ensure!(amount_1 >= amount_min, Error::<T>::InsufficientBAmount);

			pallet_wrap_currency::Pallet::<T>::do_withdraw(who.clone(), amount_0)?;

			Self::deposit_event(Event::LiquidityRemoved(
				pool.lp_token,
				amount_0,
				amount_1,
				liquidity,
			));

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::swap_exact_tokens_for_tokens())]
		#[transactional]
		pub fn swap_exact_tokens_for_tokens(
			origin: OriginFor<T>,
			#[pallet::compact] amount_in: Balance,
			#[pallet::compact] amount_out_min: Balance,
			path: Vec<T::FungibleTokenId>,
			to: T::AccountId,
			#[pallet::compact] deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(deadline > Self::now(), Error::<T>::Deadline);

			let (token_0, token_1) = Self::sort_tokens(path[0], path[1]);
			let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;

			let amounts = Self::get_amounts_out(amount_in, path.clone())?;

			ensure!(
				amounts[amounts.len() - 1] >= amount_out_min,
				Error::<T>::InsufficientOutAmount
			);

			pallet_token_fungible::Pallet::<T>::do_transfer(
				path[0],
				&who,
				&pool.lp_token_account_id,
				amounts[0],
			)?;
			Self::do_swap(who, amounts, path, to)?;

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::swap_exact_tokens_for_tokens())]
		#[transactional]
		pub fn swap_exact_w3g_for_tokens(
			origin: OriginFor<T>,
			#[pallet::compact] amount_in_w3g: Balance,
			#[pallet::compact] amount_out_min: Balance,
			path: Vec<T::FungibleTokenId>,
			to: T::AccountId,
			#[pallet::compact] deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(path[0] == T::WW3G::get(), Error::<T>::InvalidPath);
			ensure!(deadline > Self::now(), Error::<T>::Deadline);

			let (token_0, token_1) = Self::sort_tokens(path[0], path[1]);
			let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;

			let amounts = Self::get_amounts_out(amount_in_w3g, path.clone())?;

			ensure!(
				amounts[amounts.len() - 1] >= amount_out_min,
				Error::<T>::InsufficientOutAmount
			);

			pallet_wrap_currency::Pallet::<T>::do_deposit(who.clone(), amounts[0])?;

			pallet_token_fungible::Pallet::<T>::do_transfer(
				path[0],
				&who,
				&pool.lp_token_account_id,
				amounts[0],
			)?;
			Self::do_swap(who, amounts, path, to)?;

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::swap_tokens_for_exact_tokens())]
		#[transactional]
		pub fn swap_tokens_for_exact_tokens(
			origin: OriginFor<T>,
			#[pallet::compact] amount_out: Balance,
			#[pallet::compact] amount_in_max: Balance,
			path: Vec<T::FungibleTokenId>,
			to: T::AccountId,
			#[pallet::compact] deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(deadline > Self::now(), Error::<T>::Deadline);

			let (token_0, token_1) = Self::sort_tokens(path[0], path[1]);
			let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;

			let amounts = Self::get_amounts_in(amount_out, path.clone())?;
			ensure!(amounts[0] <= amount_in_max, Error::<T>::InsufficientInputAmount);

			pallet_token_fungible::Pallet::<T>::do_transfer(
				path[0],
				&who,
				&pool.lp_token_account_id,
				amounts[0],
			)?;
			Self::do_swap(who, amounts, path, to)?;

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::swap_tokens_for_exact_tokens())]
		#[transactional]
		pub fn swap_tokens_for_exact_w3g(
			origin: OriginFor<T>,
			#[pallet::compact] amount_out_w3g: Balance,
			#[pallet::compact] amount_in_max: Balance,
			path: Vec<T::FungibleTokenId>,
			to: T::AccountId,
			#[pallet::compact] deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(path[path.len() - 1] == T::WW3G::get(), Error::<T>::InvalidPath);
			ensure!(deadline > Self::now(), Error::<T>::Deadline);

			let (token_0, token_1) = Self::sort_tokens(path[0], path[1]);
			let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;

			let amounts = Self::get_amounts_in(amount_out_w3g, path.clone())?;
			ensure!(amounts[0] <= amount_in_max, Error::<T>::InsufficientInputAmount);

			pallet_token_fungible::Pallet::<T>::do_transfer(
				path[0],
				&who,
				&pool.lp_token_account_id,
				amounts[0],
			)?;
			Self::do_swap(who, amounts.clone(), path, to.clone())?;

			pallet_wrap_currency::Pallet::<T>::do_withdraw(to, amounts[amounts.len() - 1])?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn set_fee_to(origin: OriginFor<T>, to: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			FeeTo::<T>::try_mutate(|fee_to| {
				*fee_to = Some(to.clone());
				Self::deposit_event(Event::SetFeeTo(to));
				Ok(())
			})
		}
	}
}

impl<T: Config> Pallet<T> {
	// The account ID of the vault
	fn account_id() -> T::AccountId {
		<T as Config>::PalletId::get().into_account_truncating()
	}

	pub fn token_id_to_account_id(token: T::FungibleTokenId) -> T::AccountId {
		<T as Config>::PalletId::get().into_sub_account_truncating(token)
	}

	fn now() -> T::BlockNumber {
		frame_system::Pallet::<T>::block_number()
	}

	pub fn exists(token_a: T::FungibleTokenId, token_b: T::FungibleTokenId) -> bool {
		let (token_0, token_1) = Self::sort_tokens(token_a, token_b);
		Pools::<T>::get((token_0, token_1)).is_some()
	}

	pub fn do_create_pool(
		who: T::AccountId,
		token_a: T::FungibleTokenId,
		token_b: T::FungibleTokenId,
	) -> Result<T::PoolId, DispatchError> {
		let id = NextPoolId::<T>::try_mutate(|id| -> Result<T::PoolId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailablePoolId)?;
			Ok(current_id)
		})?;
		// Creating a pool requires payment
		let deposit = T::CreatePoolDeposit::get();
		<T as Config>::Currency::transfer(&who, &Self::account_id(), deposit, AllowDeath)?;

		let (token_0, token_1) = Self::sort_tokens(token_a, token_b);

		let lp_token = Self::generate_lp_token_id(token_0, token_1);
		let lp_token_account_id = Self::token_id_to_account_id(lp_token);
		let name: Vec<u8> = "LP Token".as_bytes().to_vec();
		let symbol: Vec<u8> = "LPV1".as_bytes().to_vec();

		pallet_token_fungible::Pallet::<T>::do_create_token(
			&Self::account_id(),
			lp_token,
			name,
			symbol,
			18,
		)?;

		let pool = Pool { token_0, token_1, lp_token, lp_token_account_id };

		Pools::<T>::insert((token_0, token_1), pool);
		LpTokenToToken::<T>::insert(lp_token, (token_0, token_1));

		Self::deposit_event(Event::PoolCreated(id, token_0, token_1, who));

		Ok(id)
	}

	fn do_add_liquidity(
		token_0: T::FungibleTokenId,
		token_1: T::FungibleTokenId,
		amount_a_desired: Balance,
		amount_b_desired: Balance,
		amount_a_min: Balance,
		amount_b_min: Balance,
	) -> Result<(Balance, Balance), DispatchError> {
		let (reserve_a, reserve_b) = Self::get_reserves(token_0, token_1)?;

		let amount_a;
		let amount_b;
		// amount_a_desired amount_b_desired = amount_a_desired amount_b_desired
		if reserve_a == Zero::zero() && reserve_b == Zero::zero() {
			amount_a = amount_a_desired;
			amount_b = amount_b_desired;
		} else {
			let amount_b_optimal = Self::quote(amount_a_desired, reserve_a, reserve_b)?;
			if amount_b_optimal <= amount_b_desired {
				ensure!(amount_b_optimal >= amount_b_min, Error::<T>::InsufficientBAmount);
				amount_a = amount_a_desired;
				amount_b = amount_b_optimal;
			} else {
				let amount_a_optimal = Self::quote(amount_b_desired, reserve_b, reserve_a)?;
				ensure!(amount_a_optimal <= amount_a_desired, Error::<T>::InsufficientAmount);
				ensure!(amount_a_optimal >= amount_a_min, Error::<T>::InsufficientAAmount);
				amount_a = amount_a_optimal;
				amount_b = amount_b_desired;
			}
		}

		Ok((amount_a, amount_b))
	}

	// requires the initial amount to have already been sent to the first pair
	fn do_swap(
		who: T::AccountId,
		amounts: Vec<Balance>,
		path: Vec<T::FungibleTokenId>,
		to: T::AccountId,
	) -> DispatchResult {
		for i in 0..(path.len() - 1) {
			let (input, output) = (path[i], path[i + 1]);
			let (token_0, token_1) = Self::sort_tokens(input, output);

			let amount_out = amounts[i + 1];
			let (amount_0_out, amount_1_out) = if input == token_0 {
				(Balance::from(0u128), amount_out)
			} else {
				(amount_out, Balance::from(0u128))
			};

			let receiver = if i < path.len() - 2 {
				let (lp_token_0, lp_token_1) = Self::sort_tokens(output, path[i + 2]);
				let pool =
					Pools::<T>::get((lp_token_0, lp_token_1)).ok_or(Error::<T>::PoolNotFound)?;
				pool.lp_token_account_id
			} else {
				to.clone()
			};

			Self::swap(who.clone(), token_0, token_1, amount_0_out, amount_1_out, receiver)?;
		}

		Ok(())
	}

	pub fn swap(
		who: T::AccountId,
		token_0: T::FungibleTokenId,
		token_1: T::FungibleTokenId,
		amount_0_out: Balance,
		amount_1_out: Balance,
		to: T::AccountId,
	) -> DispatchResult {
		ensure!(
			amount_0_out > Zero::zero() || amount_1_out > Zero::zero(),
			Error::<T>::InsufficientOutAmount
		);

		let (reserve_0, reserve_1) = Self::get_reserves(token_0, token_1)?;
		let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;

		ensure!(
			amount_0_out < reserve_0 && amount_1_out < reserve_1,
			Error::<T>::InsufficientLiquidity
		);

		if amount_0_out > Zero::zero() {
			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.token_0,
				&pool.lp_token_account_id,
				&to,
				amount_0_out,
			)?;
		}
		if amount_1_out > Zero::zero() {
			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.token_1,
				&pool.lp_token_account_id,
				&to,
				amount_1_out,
			)?;
		}

		let balance_0 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_0, &pool.lp_token_account_id);
		let balance_1 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_1, &pool.lp_token_account_id);

		let amount_0_in = Self::init_amount_in(balance_0, reserve_0, amount_0_out)?;
		let amount_1_in = Self::init_amount_in(balance_1, reserve_1, amount_1_out)?;

		ensure!(
			amount_0_in > Zero::zero() || amount_1_in > Zero::zero(),
			Error::<T>::InsufficientInputAmount
		);

		let balance_0_adjusted = Self::init_balance_adjusted(balance_0, amount_0_in)?;
		let balance_1_adjusted = Self::init_balance_adjusted(balance_1, amount_1_in)?;

		ensure!(
			balance_0_adjusted * balance_1_adjusted >=
				reserve_0 * reserve_1 * Balance::from(1000u128) * Balance::from(1000u128),
			Error::<T>::AdjustedError
		);

		Self::do_update(token_0, token_1, balance_0, balance_1)?;
		Self::deposit_event(Event::Swap(
			who,
			amount_0_in,
			amount_1_in,
			amount_0_out,
			amount_1_out,
			to,
		));
		Ok(())
	}

	pub fn mint_fee(
		token_0: T::FungibleTokenId,
		token_1: T::FungibleTokenId,
		reserve_0: Balance,
		reserve_1: Balance,
	) -> Result<bool, DispatchError> {
		let k_last = KLast::<T>::get((token_0, token_1));
		if let Some(fee_on) = FeeTo::<T>::get() {
			let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;
			let total_supply = pallet_token_fungible::Pallet::<T>::total_supply(pool.lp_token);
			if k_last != 0 {
				let root_k =
					(U256::from(reserve_0).saturating_mul(U256::from(reserve_1))).integer_sqrt();
				let root_k_last = U256::from(k_last).integer_sqrt();
				if root_k > root_k_last {
					let numerator = U256::from(total_supply).saturating_mul(
						root_k.checked_sub(root_k_last).ok_or(Error::<T>::Overflow)?,
					);
					let denominator = root_k
						.saturating_mul(U256::from(5u128))
						.checked_add(root_k_last)
						.ok_or(Error::<T>::Overflow)?;
					let liquidity = numerator
						.checked_div(denominator)
						.and_then(|l| TryInto::<Balance>::try_into(l).ok())
						.ok_or(Error::<T>::Overflow)?;
					if liquidity > 0 {
						pallet_token_fungible::Pallet::<T>::do_mint(
							pool.lp_token,
							&Self::account_id(),
							fee_on,
							liquidity,
						)?;
					}
				}
			}
			Ok(true)
		} else {
			if k_last != 0 {
				KLast::<T>::mutate((token_0, token_1), |k| *k = 0);
			}
			Ok(false)
		}
	}

	pub fn mint(
		who: T::AccountId,
		token_0: T::FungibleTokenId,
		token_1: T::FungibleTokenId,
		to: T::AccountId,
	) -> Result<Balance, DispatchError> {
		let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;

		let (reserve_0, reserve_1) = Reserves::<T>::get((token_0, token_1));

		let balance_0 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_0, &pool.lp_token_account_id);
		let balance_1 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_1, &pool.lp_token_account_id);

		let amount_0 = balance_0.checked_sub(reserve_0).ok_or(Error::<T>::Overflow)?;
		let amount_1 = balance_1.checked_sub(reserve_1).ok_or(Error::<T>::Overflow)?;

		let fee_on = Self::mint_fee(token_0, token_1, reserve_0, reserve_1)?;

		let liquidity: Balance;

		let total_supply = pallet_token_fungible::Pallet::<T>::total_supply(pool.lp_token);
		if total_supply == Zero::zero() {
			liquidity = (amount_0 * amount_1)
				.integer_sqrt()
				.checked_sub(Balance::from(MINIMUM_LIQUIDITY))
				.ok_or(Error::<T>::Overflow)?;
			// permanently lock the first MINIMUM_LIQUIDITY tokens
			pallet_token_fungible::Pallet::<T>::do_mint(
				pool.lp_token,
				&Self::account_id(),
				Self::account_id(),
				Balance::from(MINIMUM_LIQUIDITY),
			)?;
		} else {
			liquidity = cmp::min(
				U256::from(amount_0)
					.checked_mul(U256::from(total_supply))
					.and_then(|l| l.checked_div(U256::from(reserve_0)))
					.and_then(|l| TryInto::<Balance>::try_into(l).ok())
					.ok_or(Error::<T>::Overflow)?,
				U256::from(amount_1)
					.checked_mul(U256::from(total_supply))
					.and_then(|l| l.checked_div(U256::from(reserve_1)))
					.and_then(|l| TryInto::<Balance>::try_into(l).ok())
					.ok_or(Error::<T>::Overflow)?,
			);
		}
		ensure!(liquidity >= Zero::zero(), Error::<T>::InsufficientLiquidityMinted);
		pallet_token_fungible::Pallet::<T>::do_mint(
			pool.lp_token,
			&Self::account_id(),
			to.clone(),
			liquidity,
		)?;

		Self::do_update(token_0, token_1, balance_0, balance_1)?;

		if fee_on {
			let (_reserve_0, _reserve_1) = Reserves::<T>::get((token_0, token_1));
			KLast::<T>::mutate((token_0, token_1), |k| *k = _reserve_0 * _reserve_1);
		}
		Self::deposit_event(Event::Mint(who, amount_0, amount_1, to));
		Ok(liquidity)
	}

	pub fn get_liquidity(
		token_a: T::FungibleTokenId,
		amount_a: Balance,
		token_b: T::FungibleTokenId,
		amount_b: Balance,
	) -> Result<Balance, DispatchError> {
		let (token_0, token_1) = Self::sort_tokens(token_a, token_b);
		let amount_0;
		let amount_1;
		if token_a == token_0 {
			amount_0 = amount_a;
			amount_1 = amount_b;
		} else {
			amount_0 = amount_b;
			amount_1 = amount_a;
		}
		let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;
		let (reserve_0, reserve_1) = Reserves::<T>::get((token_0, token_1));
		let liquidity: Balance;
		let total_supply = pallet_token_fungible::Pallet::<T>::total_supply(pool.lp_token);
		if total_supply == Zero::zero() {
			liquidity = (amount_0 * amount_1)
				.integer_sqrt()
				.checked_sub(MINIMUM_LIQUIDITY)
				.ok_or(Error::<T>::Overflow)?;
		} else {
			liquidity = cmp::min(
				U256::from(amount_0)
					.checked_mul(U256::from(total_supply))
					.and_then(|l| l.checked_div(U256::from(reserve_0)))
					.and_then(|l| TryInto::<Balance>::try_into(l).ok())
					.ok_or(Error::<T>::Overflow)?,
				U256::from(amount_1)
					.checked_mul(U256::from(total_supply))
					.and_then(|l| l.checked_div(U256::from(reserve_1)))
					.and_then(|l| TryInto::<Balance>::try_into(l).ok())
					.ok_or(Error::<T>::Overflow)?,
			);
		}
		ensure!(liquidity >= Zero::zero(), Error::<T>::InsufficientLiquidityMinted);

		Ok(liquidity)
	}

	pub fn liquidity_to_token(
		lp_token: T::FungibleTokenId,
		lp_balance: Balance,
	) -> Result<(Balance, Balance), DispatchError> {
		let (token_0, token_1) =
			LpTokenToToken::<T>::get(lp_token).ok_or(Error::<T>::PoolNotFound)?;
		ensure!(token_0 != token_1, Error::<T>::PoolNotFound);
		let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;

		let balance_0 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_0, &pool.lp_token_account_id);
		let balance_1 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_1, &pool.lp_token_account_id);

		let liquidity_balance = pallet_token_fungible::Pallet::<T>::balance_of(
			pool.lp_token,
			&pool.lp_token_account_id,
		);

		let liquidity = liquidity_balance + lp_balance;

		let total_supply = pallet_token_fungible::Pallet::<T>::total_supply(pool.lp_token);

		let amount_0 = U256::from(liquidity)
			.checked_mul(U256::from(balance_0))
			.and_then(|l| l.checked_div(U256::from(total_supply)))
			.and_then(|l| TryInto::<Balance>::try_into(l).ok())
			.ok_or(Error::<T>::Overflow)?;
		let amount_1 = U256::from(liquidity)
			.checked_mul(U256::from(balance_1))
			.and_then(|l| l.checked_div(U256::from(total_supply)))
			.and_then(|l| TryInto::<Balance>::try_into(l).ok())
			.ok_or(Error::<T>::Overflow)?;
		ensure!(
			amount_0 > Zero::zero() && amount_1 > Zero::zero(),
			Error::<T>::InsufficientLiquidityBurned
		);
		Ok((amount_0, amount_1))
	}

	pub fn burn(
		who: T::AccountId,
		token_0: T::FungibleTokenId,
		token_1: T::FungibleTokenId,
		to: T::AccountId,
	) -> Result<(Balance, Balance), DispatchError> {
		let pool = Pools::<T>::get((token_0, token_1)).ok_or(Error::<T>::PoolNotFound)?;
		let (reserve_0, reserve_1) = Reserves::<T>::get((token_0, token_1));

		let mut balance_0 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_0, &pool.lp_token_account_id);
		let mut balance_1 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_1, &pool.lp_token_account_id);

		let liquidity = pallet_token_fungible::Pallet::<T>::balance_of(
			pool.lp_token,
			&pool.lp_token_account_id,
		);

		let fee_on = Self::mint_fee(token_0, token_1, reserve_0, reserve_1)?;

		let total_supply = pallet_token_fungible::Pallet::<T>::total_supply(pool.lp_token);

		let amount_0 = U256::from(liquidity)
			.checked_mul(U256::from(balance_0))
			.and_then(|l| l.checked_div(U256::from(total_supply)))
			.and_then(|l| TryInto::<Balance>::try_into(l).ok())
			.ok_or(Error::<T>::Overflow)?;
		let amount_1 = U256::from(liquidity)
			.checked_mul(U256::from(balance_1))
			.and_then(|l| l.checked_div(U256::from(total_supply)))
			.and_then(|l| TryInto::<Balance>::try_into(l).ok())
			.ok_or(Error::<T>::Overflow)?;
		ensure!(
			amount_0 > Zero::zero() && amount_1 > Zero::zero(),
			Error::<T>::InsufficientLiquidityBurned
		);

		pallet_token_fungible::Pallet::<T>::do_burn(
			pool.lp_token,
			&pool.lp_token_account_id,
			liquidity,
		)?;
		pallet_token_fungible::Pallet::<T>::do_transfer(
			pool.token_0,
			&pool.lp_token_account_id,
			&to,
			amount_0,
		)?;
		pallet_token_fungible::Pallet::<T>::do_transfer(
			pool.token_1,
			&pool.lp_token_account_id,
			&to,
			amount_1,
		)?;

		balance_0 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_0, &pool.lp_token_account_id);
		balance_1 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_1, &pool.lp_token_account_id);

		Self::do_update(token_0, token_1, balance_0, balance_1)?;

		if fee_on {
			let (_reserve_0, _reserve_1) = Reserves::<T>::get((token_0, token_1));
			KLast::<T>::mutate((token_0, token_1), |k| *k = _reserve_0 * _reserve_1);
		}
		Self::deposit_event(Event::Burn(who, amount_0, amount_1, to));
		Ok((amount_0, amount_1))
	}

	fn do_update(
		token_0: T::FungibleTokenId,
		token_1: T::FungibleTokenId,
		balance_0: Balance,
		balance_1: Balance,
	) -> DispatchResult {
		Reserves::<T>::mutate((token_0, token_1), |reserve| *reserve = (balance_0, balance_1));
		Self::deposit_event(Event::Sync(balance_0, balance_1));
		Ok(())
	}

	fn init_amount_in(
		balance: Balance,
		reserve: Balance,
		amount_out: Balance,
	) -> Result<Balance, DispatchError> {
		if balance > reserve {
			let temp = U256::from(reserve)
				.checked_sub(U256::from(amount_out))
				.ok_or(Error::<T>::Overflow)?;
			Ok(U256::from(balance)
				.checked_sub(temp)
				.and_then(|l| TryInto::<Balance>::try_into(l).ok())
				.ok_or(Error::<T>::Overflow)?)
		} else {
			Ok(Zero::zero())
		}
	}

	fn init_balance_adjusted(
		balance: Balance,
		amount_in: Balance,
	) -> Result<Balance, DispatchError> {
		let temp_0 = U256::from(balance)
			.checked_mul(U256::from(1000u128))
			.ok_or(Error::<T>::Overflow)?;
		let temp_1 = U256::from(amount_in)
			.checked_mul(U256::from(3u128))
			.ok_or(Error::<T>::Overflow)?;
		Ok(temp_0
			.checked_sub(temp_1)
			.and_then(|l| TryInto::<Balance>::try_into(l).ok())
			.ok_or(Error::<T>::Overflow)?)
	}

	fn sort_tokens(
		token_a: T::FungibleTokenId,
		token_b: T::FungibleTokenId,
	) -> (T::FungibleTokenId, T::FungibleTokenId) {
		if token_a < token_b {
			(token_a, token_b)
		} else {
			(token_b, token_a)
		}
	}

	pub fn get_reserves(
		token_a: T::FungibleTokenId,
		token_b: T::FungibleTokenId,
	) -> Result<(Balance, Balance), DispatchError> {
		let (token_0, token_1) = Self::sort_tokens(token_a, token_b);

		let (reserve_0, reserve_1) = Reserves::<T>::get((token_0, token_1));
		let (reserve_a, reserve_b) =
			if token_a == token_0 { (reserve_0, reserve_1) } else { (reserve_1, reserve_0) };

		Ok((reserve_a, reserve_b))
	}

	pub fn quote(
		amount_a: Balance,
		reserve_a: Balance,
		reserve_b: Balance,
	) -> Result<Balance, DispatchError> {
		ensure!(amount_a > Zero::zero(), Error::<T>::InsufficientAmount);
		ensure!(
			reserve_a > Zero::zero() && reserve_b > Zero::zero(),
			Error::<T>::InsufficientLiquidity
		);
		let amount_b = U256::from(amount_a)
			.checked_mul(U256::from(reserve_b))
			.and_then(|l| l.checked_div(U256::from(reserve_a)))
			.and_then(|l| TryInto::<Balance>::try_into(l).ok())
			.ok_or(Error::<T>::Overflow)?;

		Ok(amount_b)
	}

	pub fn get_amount_out(
		amount_in: Balance,
		reserve_in: Balance,
		reserve_out: Balance,
	) -> Result<Balance, DispatchError> {
		ensure!(amount_in > Zero::zero(), Error::<T>::InsufficientInputAmount);
		ensure!(
			reserve_in > Zero::zero() && reserve_out > Zero::zero(),
			Error::<T>::InsufficientLiquidity
		);

		let amount_in_with_fee: U256 = U256::from(amount_in).saturating_mul(U256::from(997u128));

		let numerator: U256 =
			U256::from(amount_in_with_fee).saturating_mul(U256::from(reserve_out));

		let denominator: U256 = (U256::from(reserve_in).saturating_mul(U256::from(1000u128)))
			.saturating_add(amount_in_with_fee);

		let amount_out = numerator
			.checked_div(denominator)
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())
			.ok_or(Error::<T>::Overflow)?;
		Ok(amount_out)
	}

	pub fn get_amount_in(
		amount_out: Balance,
		reserve_in: Balance,
		reserve_out: Balance,
	) -> Result<Balance, DispatchError> {
		ensure!(amount_out > Zero::zero(), Error::<T>::InsufficientOutputAmount);
		ensure!(
			reserve_in > Zero::zero() && reserve_out > Zero::zero(),
			Error::<T>::InsufficientLiquidity
		);

		let numerator: U256 = U256::from(reserve_in)
			.saturating_mul(U256::from(amount_out))
			.saturating_mul(U256::from(1000u128));
		let denominator: U256 = (U256::from(reserve_out).saturating_sub(U256::from(amount_out)))
			.saturating_mul(U256::from(997u128));

		let amount_in = U256::from(numerator)
			.checked_div(U256::from(denominator))
			.and_then(|r| r.checked_add(U256::one()))
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())
			.ok_or(Error::<T>::Overflow)?;

		Ok(amount_in)
	}

	pub fn get_amounts_out(
		amount_in: Balance,
		path: Vec<T::FungibleTokenId>,
	) -> Result<Vec<Balance>, DispatchError> {
		ensure!(path.len() >= 2, Error::<T>::InvalidPath);

		let mut amounts = vec![Balance::from(0u128); path.len()];
		amounts[0] = amount_in;
		for i in 0..(path.len() - 1) {
			let (reserve_in, reserve_out) = Self::get_reserves(path[i], path[i + 1])?;
			amounts[i + 1] = Self::get_amount_out(amounts[i], reserve_in, reserve_out)?;
		}
		Ok(amounts)
	}

	pub fn get_amounts_in(
		amount_out: Balance,
		path: Vec<T::FungibleTokenId>,
	) -> Result<Vec<Balance>, DispatchError> {
		ensure!(path.len() >= 2, Error::<T>::InvalidPath);

		let mut amounts = vec![Balance::from(0u128); path.len()];
		amounts[path.len() - 1] = amount_out;
		for i in (1..path.len()).rev() {
			let (reserve_in, reserve_out) = Self::get_reserves(path[i - 1], path[i])?;
			amounts[i - 1] = Self::get_amount_in(amounts[i], reserve_in, reserve_out)?;
		}

		Ok(amounts)
	}

	fn generate_lp_token_id(
		token_0: T::FungibleTokenId,
		token_1: T::FungibleTokenId,
	) -> T::FungibleTokenId {
		let (random_seed, _) =
			T::Randomness::random(&(Self::account_id(), token_0, token_1).encode());
		let lp_token_id = <T::FungibleTokenId>::decode(&mut random_seed.as_ref())
			.expect("Failed to decode random seed");
		lp_token_id
	}

	pub fn vec_to_u128(data: Vec<u8>) -> Result<u128, DispatchError> {
		let data_str = sp_std::str::from_utf8(&data).map_err(|_| Error::<T>::VecToU128Failed)?;
		let data_u128 = data_str.parse::<u128>().map_err(|_| Error::<T>::VecToU128Failed)?;
		Ok(data_u128)
	}

	pub fn vecs_to_u128(data: Vec<Vec<u8>>) -> Result<Vec<u128>, DispatchError> {
		let mut new_data: Vec<u128> = vec![];
		for d in data {
			new_data.push(Self::vec_to_u128(d)?);
		}
		Ok(new_data)
	}
}
