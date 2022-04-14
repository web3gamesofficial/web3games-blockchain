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
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, Get, ReservableCurrency},
	BoundedVec, PalletId,
};
use pallet_support::FungibleMetadata;
use primitives::Balance;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, TrailingZeroInput},
	RuntimeDebug,
};
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Token<AccountId, BoundedString> {
	owner: AccountId,
	name: BoundedString,
	symbol: BoundedString,
	decimals: u8,
	total_supply: Balance,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type PalletId: Get<PalletId>;

		/// Identifier for the class of token.
		type FungibleTokenId: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaxEncodedLen;

		/// The maximum length of a name or symbol stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// The minimum balance to create token
		#[pallet::constant]
		type CreateTokenDeposit: Get<BalanceOf<Self>>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::storage]
	pub(super) type Tokens<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::FungibleTokenId,
		Token<T::AccountId, BoundedVec<u8, T::StringLimit>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::FungibleTokenId,
		Blake2_128Concat,
		T::AccountId,
		Balance,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn allowances)]
	pub(super) type Allowances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::FungibleTokenId,
		Blake2_128Concat,
		// (owner, operator)
		(T::AccountId, T::AccountId),
		Balance,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenCreated(T::FungibleTokenId, T::AccountId, Vec<u8>, Vec<u8>, u8),
		Transfer(T::FungibleTokenId, T::AccountId, T::AccountId, Balance),
		Approval(T::FungibleTokenId, T::AccountId, T::AccountId, Balance),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unknown,
		NotFound,
		NoAvailableTokenId,
		Overflow,
		NoPermission,
		NotOwner,
		InvalidId,
		BadMetadata,
		InsufficientBalance,
		InsufficientAllowance,
		ApproveToCurrentOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_token(
			origin: OriginFor<T>,
			id: T::FungibleTokenId,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_create_token(&who, id, name, symbol, decimals)
		}

		#[pallet::weight(10_000)]
		pub fn approve(
			origin: OriginFor<T>,
			id: T::FungibleTokenId,
			spender: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_approve(id, &who, &spender, amount)
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			id: T::FungibleTokenId,
			recipient: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_transfer(id, &who, &recipient, amount)
		}

		#[pallet::weight(10_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			id: T::FungibleTokenId,
			sender: T::AccountId,
			recipient: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_transfer_from(id, who, sender, recipient, amount)
		}

		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
			id: T::FungibleTokenId,
			account: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_mint(id, &who, &account, amount)
		}

		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			id: T::FungibleTokenId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_burn(id, &who, amount)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn zero_account_id() -> T::AccountId {
		T::AccountId::decode(&mut TrailingZeroInput::zeroes()).expect("infinite input; qed")
	}

	pub fn exists(id: T::FungibleTokenId) -> bool {
		Tokens::<T>::contains_key(id)
	}

	pub fn total_supply(id: T::FungibleTokenId) -> Balance {
		Tokens::<T>::get(id).unwrap().total_supply
	}

	pub fn do_create_token(
		who: &T::AccountId,
		id: T::FungibleTokenId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> DispatchResult {
		let deposit = T::CreateTokenDeposit::get();
		T::Currency::reserve(&who, deposit.clone())?;

		let bounded_name: BoundedVec<u8, T::StringLimit> =
			name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		let bounded_symbol: BoundedVec<u8, T::StringLimit> =
			symbol.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		ensure!(!Self::exists(id.clone()), Error::<T>::InvalidId);

		let token = Token {
			owner: who.clone(),
			name: bounded_name,
			symbol: bounded_symbol,
			decimals,
			total_supply: Balance::default(),
		};

		Tokens::<T>::insert(id, token);

		Self::deposit_event(Event::TokenCreated(id, who.clone(), name, symbol, decimals));

		Ok(())
	}

	pub fn do_approve(
		id: T::FungibleTokenId,
		who: &T::AccountId,
		spender: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		ensure!(spender != who, Error::<T>::ApproveToCurrentOwner);

		ensure!(Balances::<T>::get(id, who.clone()) >= amount, Error::<T>::InsufficientBalance);

		Allowances::<T>::try_mutate(id, (&who, &spender), |allowance| -> DispatchResult {
			*allowance = allowance.checked_add(amount).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;

		Self::deposit_event(Event::Transfer(id, who.clone(), spender.clone(), amount));

		Ok(())
	}

	pub fn do_transfer_from(
		id: T::FungibleTokenId,
		who: T::AccountId,
		sender: T::AccountId,
		recipient: T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		ensure!(
			Allowances::<T>::get(id, (&sender, &who)) >= amount,
			Error::<T>::InsufficientAllowance
		);

		if sender == recipient {
			return Ok(());
		}

		Allowances::<T>::try_mutate(id, (&sender, &who), |allowance| -> DispatchResult {
			*allowance = allowance.checked_sub(amount).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;

		Self::internal_transfer(id, &sender, &recipient, amount)?;

		Ok(())
	}

	pub fn do_transfer(
		id: T::FungibleTokenId,
		who: &T::AccountId,
		recipient: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		if who == recipient {
			return Ok(());
		}

		ensure!(Balances::<T>::get(id, who.clone()) >= amount, Error::<T>::InsufficientBalance);

		Self::internal_transfer(id, who, recipient, amount)?;

		Ok(())
	}

	fn internal_transfer(
		id: T::FungibleTokenId,
		sender: &T::AccountId,
		recipient: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		Self::decrease_balance(id, sender, amount)?;
		Self::increase_balance(id, recipient, amount)?;

		Self::deposit_event(Event::Transfer(id, sender.clone(), recipient.clone(), amount));

		Ok(())
	}

	pub fn do_mint(
		id: T::FungibleTokenId,
		who: &T::AccountId,
		account: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		Self::maybe_check_permission(id, &who)?;

		Self::internal_mint(id, account, amount)?;

		Ok(())
	}

	fn internal_mint(
		id: T::FungibleTokenId,
		account: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		Tokens::<T>::try_mutate_exists(id, |maybe_token| -> DispatchResult {
			let token = maybe_token.as_mut().ok_or(Error::<T>::Unknown)?;

			Self::increase_balance(id, account, amount)?;

			let new_total_supply = token.total_supply.saturating_add(amount);
			token.total_supply = new_total_supply;
			Ok(())
		})?;

		Self::deposit_event(Event::Transfer(id, Self::zero_account_id(), account.clone(), amount));

		Ok(())
	}

	pub fn do_burn(
		id: T::FungibleTokenId,
		account: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		Tokens::<T>::try_mutate_exists(id, |maybe_token| -> DispatchResult {
			let token = maybe_token.as_mut().ok_or(Error::<T>::Unknown)?;

			Self::decrease_balance(id, account, amount)?;

			let new_total_supply = token.total_supply.saturating_sub(amount);
			token.total_supply = new_total_supply;
			Ok(())
		})?;

		Self::deposit_event(Event::Transfer(id, account.clone(), Self::zero_account_id(), amount));

		Ok(())
	}

	fn increase_balance(
		id: T::FungibleTokenId,
		to: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		Balances::<T>::try_mutate(id, to, |balance| -> DispatchResult {
			*balance = balance.checked_add(amount).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;

		Ok(())
	}

	fn decrease_balance(
		id: T::FungibleTokenId,
		from: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		Balances::<T>::try_mutate(id, from, |balance| -> DispatchResult {
			*balance = balance.checked_sub(amount).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;

		Ok(())
	}

	fn maybe_check_permission(id: T::FungibleTokenId, who: &T::AccountId) -> DispatchResult {
		let token = Tokens::<T>::get(id).ok_or(Error::<T>::NotFound)?;
		ensure!(*who == token.owner, Error::<T>::NoPermission);

		Ok(())
	}
}

impl<T: Config> FungibleMetadata for Pallet<T> {
	type FungibleTokenId = T::FungibleTokenId;

	fn token_name(id: Self::FungibleTokenId) -> Vec<u8> {
		Tokens::<T>::get(id).unwrap().name.to_vec()
	}

	fn token_symbol(id: Self::FungibleTokenId) -> Vec<u8> {
		Tokens::<T>::get(id).unwrap().symbol.to_vec()
	}

	fn token_decimals(id: Self::FungibleTokenId) -> u8 {
		Tokens::<T>::get(id).unwrap().decimals
	}
}
