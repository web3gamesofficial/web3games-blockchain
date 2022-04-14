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
	traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, One, TrailingZeroInput, Zero},
	RuntimeDebug,
};
use sp_std::{cmp, prelude::*};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub const MINIMUM_LIQUIDITY: u128 = 1000; // 10**3;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Pool<AccountId, FungibleTokenId> {
	/// The owner of pool
	pub owner: AccountId,
	/// The id of first token
	pub token_0: FungibleTokenId,
	/// The id of second token
	pub token_1: FungibleTokenId,
	/// The id of liquidity pool token
	pub lp_token: FungibleTokenId,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_token_fungible::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type PalletId: Get<PalletId>;

		type PoolId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// The minimum balance to create pool
		#[pallet::constant]
		type CreatePoolDeposit: Get<BalanceOf<Self>>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Pools<T: Config> =
		StorageMap<_, Blake2_128, T::PoolId, Pool<T::AccountId, T::FungibleTokenId>>;

	#[pallet::storage]
	#[pallet::getter(fn next_pool_id)]
	pub(super) type NextPoolId<T: Config> = StorageValue<_, T::PoolId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_pool)]
	pub(super) type GetPool<T: Config> =
		StorageMap<_, Blake2_128, (T::FungibleTokenId, T::FungibleTokenId), T::PoolId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reserves)]
	pub(super) type Reserves<T: Config> =
		StorageMap<_, Blake2_128, T::PoolId, (Balance, Balance), ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated(T::PoolId, T::AccountId),
		LiquidityAdded(T::PoolId, Balance, Balance, Balance),
		LiquidityRemoved(T::PoolId),
		Swapped(T::PoolId),
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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_pool(
			origin: OriginFor<T>,
			token_a: T::FungibleTokenId,
			token_b: T::FungibleTokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(token_a != token_b, Error::<T>::TokenRepeat);
			ensure!(
				pallet_token_fungible::Pallet::<T>::exists(token_a),
				Error::<T>::TokenAccountNotFound,
			);
			ensure!(
				pallet_token_fungible::Pallet::<T>::exists(token_b),
				Error::<T>::TokenAccountNotFound,
			);
			ensure!(
				!GetPool::<T>::contains_key((token_a, token_b)),
				Error::<T>::PoolAlreadyCreated
			);

			Self::do_create_pool(&who, token_a, token_b)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			id: T::PoolId,
			amount_a_desired: Balance,
			amount_b_desired: Balance,
			amount_a_min: Balance,
			amount_b_min: Balance,
			to: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let pool = Pools::<T>::get(id).ok_or(Error::<T>::PoolNotFound)?;

			let (amount_a, amount_b) = Self::do_add_liquidity(
				id,
				amount_a_desired,
				amount_b_desired,
				amount_a_min,
				amount_b_min,
			)?;

			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.token_0,
				&who,
				&Self::account_id(),
				amount_a,
			)?;
			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.token_1,
				&who,
				&Self::account_id(),
				amount_b,
			)?;
			let liquidity = Self::mint(id, &to)?;

			Self::deposit_event(Event::LiquidityAdded(id, amount_a, amount_b, liquidity));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			id: T::PoolId,
			token_a: T::FungibleTokenId,
			token_b: T::FungibleTokenId,
			liquidity: Balance,
			amount_a_min: Balance,
			amount_b_min: Balance,
			to: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let pool = Pools::<T>::get(id).ok_or(Error::<T>::PoolNotFound)?;

			// return Lp to Pallet
			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.lp_token,
				&who,
				&Self::account_id(),
				liquidity,
			)?;

			let (amount_0, amount_1) = Self::burn(&who, id, &to)?;
			let (token_0, _token_1) = Self::sort_tokens(token_a, token_b);
			let (amount_a, amount_b) =
				if token_a == token_0 { (amount_0, amount_1) } else { (amount_1, amount_0) };

			ensure!(amount_a >= amount_a_min, Error::<T>::InsufficientAAmount);
			ensure!(amount_b >= amount_b_min, Error::<T>::InsufficientBAmount);

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn swap_exact_tokens_for_tokens(
			origin: OriginFor<T>,
			id: T::PoolId,
			amount_in: Balance,
			amount_out_min: Balance,
			path: Vec<T::FungibleTokenId>,
			to: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let _pool = Pools::<T>::get(id).ok_or(Error::<T>::PoolNotFound)?;

			let amounts = Self::get_amounts_out(id, amount_in, path.clone())?;
			ensure!(
				amounts[amounts.len() - 1] >= amount_out_min,
				Error::<T>::InsufficientOutAmount
			);

			pallet_token_fungible::Pallet::<T>::do_transfer(
				path[0],
				&who,
				&Self::account_id(),
				amounts[0],
			)?;
			Self::do_swap(id, amounts, path, &to).expect("swap fail");

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn swap_tokens_for_exact_tokens(
			origin: OriginFor<T>,
			id: T::PoolId,
			amount_out: Balance,
			amount_in_max: Balance,
			path: Vec<T::FungibleTokenId>,
			to: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let _pool = Pools::<T>::get(id).ok_or(Error::<T>::PoolNotFound)?;

			let amounts = Self::get_amounts_out(id, amount_out, path.clone())?;
			ensure!(amounts[0] <= amount_in_max, Error::<T>::InsufficientInputAmount);

			pallet_token_fungible::Pallet::<T>::do_transfer(
				path[0],
				&who,
				&Self::account_id(),
				amounts[0],
			)?;
			Self::do_swap(id, amounts, path, &to)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn zero_account_id() -> T::AccountId {
		T::AccountId::decode(&mut TrailingZeroInput::zeroes()).expect("infinite input; qed")
	}

	// The account ID of the vault
	fn account_id() -> T::AccountId {
		<T as Config>::PalletId::get().into_account()
	}

	pub fn exists(token_a: T::FungibleTokenId, token_b: T::FungibleTokenId) -> bool {
		GetPool::<T>::contains_key((token_a, token_b))
	}

	pub fn do_create_pool(
		who: &T::AccountId,
		token_a: T::FungibleTokenId,
		token_b: T::FungibleTokenId,
	) -> Result<T::PoolId, DispatchError> {
		let id = NextPoolId::<T>::try_mutate(|id| -> Result<T::PoolId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailablePoolId)?;
			Ok(current_id)
		})?;

		let deposit = T::CreatePoolDeposit::get();
		<T as Config>::Currency::transfer(who, &Self::account_id(), deposit, AllowDeath)?;

		let lp_token = Self::generate_random_token_id(id);
		let name: Vec<u8> = "LP Token".as_bytes().to_vec();
		let symbol: Vec<u8> = "LPV1".as_bytes().to_vec();

		pallet_token_fungible::Pallet::<T>::do_create_token(
			&Self::account_id(),
			lp_token,
			name,
			symbol,
			18,
		)?;

		let (token_0, token_1) = Self::sort_tokens(token_a, token_b);

		let pool = Pool { owner: who.clone(), token_0, token_1, lp_token };

		Pools::<T>::insert(id, pool);
		GetPool::<T>::insert((token_a, token_b), id);

		Self::deposit_event(Event::PoolCreated(id, who.clone()));

		Ok(id)
	}

	fn do_add_liquidity(
		id: T::PoolId,
		amount_a_desired: Balance,
		amount_b_desired: Balance,
		amount_a_min: Balance,
		amount_b_min: Balance,
	) -> Result<(Balance, Balance), DispatchError> {
		let (reserve_a, reserve_b) = Reserves::<T>::get(id);

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

	fn do_swap(
		id: T::PoolId,
		amounts: Vec<Balance>,
		path: Vec<T::FungibleTokenId>,
		to: &T::AccountId,
	) -> DispatchResult {
		let _pool = Pools::<T>::get(id).ok_or(Error::<T>::PoolNotFound)?;

		let vault_account = Self::account_id();

		for i in 0..path.len() - 1 {
			let (input, output) = (path[i], path[i + 1]);
			let (token_0, _) = Self::sort_tokens(input, output);
			let amount_out = amounts[i + 1];
			let (amount_0_out, amount_1_out) = if input == token_0 {
				(Balance::from(0u128), amount_out)
			} else {
				(amount_out, Balance::from(0u128))
			};
			let receiver = if i < path.len() - 2 { vault_account.clone() } else { to.clone() };

			Self::swap(id, amount_0_out, amount_1_out, &receiver)?;
		}

		Ok(())
	}

	pub fn swap(
		id: T::PoolId,
		amount_0_out: Balance,
		amount_1_out: Balance,
		to: &T::AccountId,
	) -> DispatchResult {
		let pool = Pools::<T>::get(id).ok_or(Error::<T>::PoolNotFound)?;

		ensure!(
			amount_0_out > Zero::zero() || amount_1_out > Zero::zero(),
			Error::<T>::InsufficientOutAmount
		);

		let (reserve_0, reserve_1) = Reserves::<T>::get(id);
		ensure!(
			amount_0_out < reserve_0 && amount_1_out < reserve_1,
			Error::<T>::InsufficientLiquidity
		);

		let vault_account = Self::account_id();

		if amount_0_out > Zero::zero() {
			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.token_0,
				&vault_account,
				to,
				amount_0_out,
			)?;
		}
		if amount_1_out > Zero::zero() {
			pallet_token_fungible::Pallet::<T>::do_transfer(
				pool.token_1,
				&vault_account,
				to,
				amount_1_out,
			)?;
		}

		let balance_0 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_0, &vault_account);
		let balance_1 =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_1, &vault_account);

		let amount_0_in = Self::init_amount_in(balance_0, reserve_0, amount_0_out);
		let amount_1_in = Self::init_amount_in(balance_1, reserve_1, amount_1_out);

		ensure!(
			amount_0_in > Zero::zero() || amount_1_in > Zero::zero(),
			Error::<T>::InsufficientInputAmount
		);

		let balance_0_adjusted =
			balance_0 * Balance::from(1000u128) - (amount_0_in * Balance::from(3u128));
		let balance_1_adjusted =
			balance_1 * Balance::from(1000u128) - (amount_1_in * Balance::from(3u128));
		ensure!(
			balance_0_adjusted * balance_1_adjusted
				>= reserve_0 * reserve_1 * Balance::from(1000u128) * Balance::from(1000u128),
			Error::<T>::AdjustedError
		);

		Self::do_update(id, balance_0, balance_1, reserve_0, reserve_1).expect("do update fail");

		Ok(())
	}

	pub fn mint(id: T::PoolId, to: &T::AccountId) -> Result<Balance, DispatchError> {
		let pool = Pools::<T>::get(id).ok_or(Error::<T>::PoolNotFound)?;

		let (reserve_a, reserve_b) = Reserves::<T>::get(id);
		let vault_account = Self::account_id();

		let balance_a =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_0, &vault_account);
		let balance_b =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_1, &vault_account);

		let amount_a = balance_a - reserve_a;
		let amount_b = balance_b - reserve_b;

		let liquidity: Balance;

		let total_supply = pallet_token_fungible::Pallet::<T>::total_supply(pool.lp_token);
		if total_supply == Zero::zero() {
			let value = amount_a * amount_b;
			liquidity = value.integer_sqrt() - Balance::from(MINIMUM_LIQUIDITY);
			// permanently lock the first MINIMUM_LIQUIDITY tokens
			pallet_token_fungible::Pallet::<T>::do_mint(
				pool.lp_token,
				&Self::account_id(),
				&Self::zero_account_id(),
				Balance::from(MINIMUM_LIQUIDITY),
			)?;
		} else {
			liquidity =
				cmp::min(amount_a * total_supply / reserve_a, amount_b * total_supply / reserve_b);
		}
		ensure!(liquidity >= Zero::zero(), Error::<T>::InsufficientLiquidityMinted);
		pallet_token_fungible::Pallet::<T>::do_mint(
			pool.lp_token,
			&Self::account_id(),
			&to,
			liquidity,
		)?;

		Self::do_update(id, balance_a, balance_b, reserve_a, reserve_b)?;

		Ok(liquidity)
	}

	pub fn burn(
		_who: &T::AccountId,
		id: T::PoolId,
		to: &T::AccountId,
	) -> Result<(Balance, Balance), DispatchError> {
		let pool = Pools::<T>::get(id).ok_or(Error::<T>::PoolNotFound)?;

		let (reserve_a, reserve_b) = Reserves::<T>::get(id);
		let vault_account = Self::account_id();

		let mut balance_a =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_0, &vault_account);
		let mut balance_b =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.token_1, &vault_account);

		let liquidity =
			pallet_token_fungible::Pallet::<T>::balance_of(pool.lp_token, &vault_account);

		let total_supply = pallet_token_fungible::Pallet::<T>::total_supply(pool.lp_token);

		let amount_a = liquidity * balance_a / total_supply;
		let amount_b = liquidity * balance_b / total_supply;
		ensure!(
			amount_a > Zero::zero() && amount_b > Zero::zero(),
			Error::<T>::InsufficientLiquidityBurned
		);

		pallet_token_fungible::Pallet::<T>::do_burn(pool.lp_token, &vault_account, liquidity)?;
		pallet_token_fungible::Pallet::<T>::do_transfer(
			pool.token_0,
			&vault_account,
			to,
			amount_a,
		)?;
		pallet_token_fungible::Pallet::<T>::do_transfer(
			pool.token_1,
			&vault_account,
			to,
			amount_b,
		)?;

		balance_a = pallet_token_fungible::Pallet::<T>::balance_of(pool.token_0, &vault_account);
		balance_b = pallet_token_fungible::Pallet::<T>::balance_of(pool.token_1, &vault_account);

		Self::do_update(id, balance_a, balance_b, reserve_a, reserve_b)?;

		Ok((amount_a, amount_b))
	}

	// update reserves
	fn do_update(
		id: T::PoolId,
		balance_a: Balance,
		balance_b: Balance,
		_reserve_a: Balance,
		_reserve_b: Balance,
	) -> DispatchResult {
		Reserves::<T>::mutate(id, |reserve| *reserve = (balance_a, balance_b));
		Ok(())
	}

	fn init_amount_in(balance: Balance, reserve: Balance, amount_out: Balance) -> Balance {
		if balance > reserve {
			balance - (reserve - amount_out)
		} else {
			Zero::zero()
		}
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

	fn get_reserves(
		id: T::PoolId,
		token_a: T::FungibleTokenId,
		token_b: T::FungibleTokenId,
	) -> (Balance, Balance) {
		let (token_0, _) = Self::sort_tokens(token_a, token_b);
		let (reserve_0, reserve_1) = Reserves::<T>::get(id);
		let (reserve_a, reserve_b) =
			if token_a == token_0 { (reserve_0, reserve_1) } else { (reserve_1, reserve_0) };

		(reserve_a, reserve_b)
	}

	fn quote(
		amount_a: Balance,
		reserve_a: Balance,
		reserve_b: Balance,
	) -> Result<Balance, DispatchError> {
		ensure!(amount_a > Zero::zero(), Error::<T>::InsufficientAmount);
		ensure!(
			reserve_a > Zero::zero() && reserve_b > Zero::zero(),
			Error::<T>::InsufficientLiquidity
		);
		let amount_b = amount_a * reserve_b / reserve_a;

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
			.unwrap_or_else(Zero::zero);
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
		let (amount_in, _) = Self::div_round(numerator, denominator);

		Ok(amount_in)
	}

	pub fn get_amounts_out(
		id: T::PoolId,
		amount_in: Balance,
		path: Vec<T::FungibleTokenId>,
	) -> Result<Vec<Balance>, DispatchError> {
		ensure!(path.len() >= 2, Error::<T>::InvalidPath);

		let mut amounts = vec![Balance::from(0u128); path.len()];
		amounts[0] = amount_in;
		for i in 0..path.len() - 1 {
			let (reserve_in, reserve_out) = Self::get_reserves(id, path[i], path[i + 1]);
			amounts[i + 1] = Self::get_amount_out(amounts[i], reserve_in, reserve_out)?;
		}

		Ok(amounts)
	}

	pub fn get_amounts_in(
		id: T::PoolId,
		amount_out: Balance,
		path: Vec<T::FungibleTokenId>,
	) -> Result<Vec<Balance>, DispatchError> {
		ensure!(path.len() >= 2, Error::<T>::InvalidPath);

		let mut amounts = vec![Balance::from(0u128); path.len()];
		amounts[path.len() - 1] = amount_out;
		for i in (0..path.len() - 1).rev() {
			let (reserve_in, reserve_out) = Self::get_reserves(id, path[i - 1], path[i]);
			amounts[i - 1] = Self::get_amount_in(amounts[i], reserve_in, reserve_out)?;
		}

		Ok(amounts)
	}

	/// Divides two numbers and add 1 if there is a rounding error
	fn div_round(numerator: U256, denominator: U256) -> (Balance, bool) {
		let remainder = numerator.checked_rem(denominator).unwrap();
		if remainder.is_zero() {
			(
				numerator
					.checked_div(denominator)
					.and_then(|n| TryInto::<Balance>::try_into(n).ok())
					.unwrap_or_else(Zero::zero),
				false,
			)
		} else {
			(
				numerator
					.checked_div(denominator)
					.and_then(|r| r.checked_add(U256::one()))
					.and_then(|n| TryInto::<Balance>::try_into(n).ok())
					.unwrap_or_else(Zero::zero),
				true,
			)
		}
	}

	fn generate_random_token_id(seed: T::PoolId) -> T::FungibleTokenId {
		let (random_seed, _) = T::Randomness::random(&(Self::account_id(), seed).encode());
		let random_id = <T::FungibleTokenId>::decode(&mut random_seed.as_ref())
			.expect("Failed to decode random seed");
		random_id
	}
}
