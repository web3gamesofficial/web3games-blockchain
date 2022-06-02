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
	traits::{Currency, Get, ReservableCurrency},
	BoundedVec, PalletId,
};
use pallet_support::MultiMetadata;
use primitives::Balance;
use scale_info::TypeInfo;
use sp_runtime::{traits::AtLeast32BitUnsigned, RuntimeDebug};
use sp_std::prelude::*;

pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Token<AccountId, BoundedString> {
	owner: AccountId,
	uri: BoundedString,
	total_supply: Balance,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type PalletId: Get<PalletId>;

		type MultiTokenId: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaxEncodedLen;

		type TokenId: Member + Parameter + Default + MaxEncodedLen + Copy;

		/// The maximum length of base uri stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// The minimum balance to create token
		#[pallet::constant]
		type CreateTokenDeposit: Get<BalanceOf<Self>>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Tokens<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::MultiTokenId,
		Token<T::AccountId, BoundedVec<u8, T::StringLimit>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::MultiTokenId,
		Blake2_128Concat,
		(T::TokenId, T::AccountId),
		Balance,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn is_approved_for_all)]
	pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::MultiTokenId,
		Blake2_128Concat,
		// (owner, operator)
		(T::AccountId, T::AccountId),
		bool,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenCreated(T::MultiTokenId, T::AccountId),
		Mint(T::MultiTokenId, T::AccountId, T::TokenId, Balance),
		BatchMint(T::MultiTokenId, T::AccountId, Vec<T::TokenId>, Vec<Balance>),
		Burn(T::MultiTokenId, T::AccountId, T::TokenId, Balance),
		BatchBurn(T::MultiTokenId, T::AccountId, Vec<T::TokenId>, Vec<Balance>),
		Transferred(T::MultiTokenId, T::AccountId, T::AccountId, T::TokenId, Balance),
		BatchTransferred(
			T::MultiTokenId,
			T::AccountId,
			T::AccountId,
			Vec<T::TokenId>,
			Vec<Balance>,
		),
		ApprovalForAll(T::MultiTokenId, T::AccountId, T::AccountId, bool),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unknown,
		NoAvailableTokenId,
		NumOverflow,
		LengthMismatch,
		NoPermission,
		InvalidId,
		TokenNonExistent,
		BadMetadata,
		NotOwnerOrApproved,
		InsufficientTokens,
		InsufficientAuthorizedTokens,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_token(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			uri: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_create_token(&who, id, uri)
		}

		#[pallet::weight(10_000)]
		pub fn set_approval_for_all(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			operator: T::AccountId,
			approved: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_set_approval_for_all(&who, id, &operator, approved)
		}

		#[pallet::weight(10_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			from: T::AccountId,
			to: T::AccountId,
			token_id: T::TokenId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_transfer_from(&who, id, &from, &to, token_id, amount)
		}

		#[pallet::weight(10_000)]
		pub fn batch_transfer_from(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			from: T::AccountId,
			to: T::AccountId,
			token_ids: Vec<T::TokenId>,
			amounts: Vec<Balance>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_batch_transfer_from(&who, id, &from, &to, token_ids, amounts)
		}

		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			to: T::AccountId,
			token_id: T::TokenId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_mint(&who, id, &to, token_id, amount)
		}

		#[pallet::weight(10_000)]
		pub fn mint_batch(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			to: T::AccountId,
			token_ids: Vec<T::TokenId>,
			amounts: Vec<Balance>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_batch_mint(&who, id, &to, token_ids, amounts)
		}

		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			token_id: T::TokenId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_burn(&who, id, token_id, amount)
		}

		#[pallet::weight(10_000)]
		pub fn burn_batch(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			token_ids: Vec<T::TokenId>,
			amounts: Vec<Balance>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_batch_burn(&who, id, token_ids, amounts)
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn exists(id: T::MultiTokenId) -> bool {
		Tokens::<T>::contains_key(id)
	}

	pub fn do_create_token(
		who: &T::AccountId,
		id: T::MultiTokenId,
		uri: Vec<u8>,
	) -> DispatchResult {
		ensure!(!Self::exists(id.clone()), Error::<T>::InvalidId);

		let deposit = T::CreateTokenDeposit::get();
		T::Currency::reserve(&who, deposit.clone())?;

		let bounded_uri: BoundedVec<u8, T::StringLimit> =
			uri.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		let token =
			Token { owner: who.clone(), uri: bounded_uri, total_supply: Balance::default() };

		Tokens::<T>::insert(id, token);

		Self::deposit_event(Event::TokenCreated(id, who.clone()));

		Ok(())
	}

	pub fn do_set_approval_for_all(
		who: &T::AccountId,
		id: T::MultiTokenId,
		operator: &T::AccountId,
		approved: bool,
	) -> DispatchResult {
		ensure!(Tokens::<T>::contains_key(id), Error::<T>::InvalidId,);

		OperatorApprovals::<T>::insert(id, (&who, &operator), approved);

		Self::deposit_event(Event::ApprovalForAll(id, who.clone(), operator.clone(), approved));

		Ok(())
	}

	pub fn do_mint(
		who: &T::AccountId,
		id: T::MultiTokenId,
		to: &T::AccountId,
		token_id: T::TokenId,
		amount: Balance,
	) -> DispatchResult {
		ensure!(Self::has_permission(id, &who), Error::<T>::NoPermission);

		Tokens::<T>::try_mutate(id, |maybe_token| -> DispatchResult {
			let token = maybe_token.as_mut().ok_or(Error::<T>::Unknown)?;

			Self::increase_balance(id, to, token_id, amount)?;

			let new_total_supply = token.total_supply.saturating_add(amount);
			token.total_supply = new_total_supply;
			Ok(())
		})?;

		Self::deposit_event(Event::Mint(id, to.clone(), token_id, amount));

		Ok(())
	}

	pub fn do_batch_mint(
		who: &T::AccountId,
		id: T::MultiTokenId,
		to: &T::AccountId,
		token_ids: Vec<T::TokenId>,
		amounts: Vec<Balance>,
	) -> DispatchResult {
		ensure!(Self::has_permission(id, &who), Error::<T>::NoPermission);
		ensure!(token_ids.len() == amounts.len(), Error::<T>::LengthMismatch);

		let n = token_ids.len();
		for i in 0..n {
			let token_id = token_ids[i];
			let amount = amounts[i];

			Tokens::<T>::try_mutate(id, |maybe_token| -> DispatchResult {
				let token = maybe_token.as_mut().ok_or(Error::<T>::Unknown)?;

				Self::increase_balance(id, to, token_id, amount)?;

				let new_total_supply = token.total_supply.saturating_add(amount);
				token.total_supply = new_total_supply;
				Ok(())
			})?;
		}

		Self::deposit_event(Event::BatchMint(id, to.clone(), token_ids, amounts));

		Ok(())
	}

	pub fn do_burn(
		who: &T::AccountId,
		id: T::MultiTokenId,
		token_id: T::TokenId,
		amount: Balance,
	) -> DispatchResult {
		Tokens::<T>::try_mutate(id, |maybe_token| -> DispatchResult {
			let token = maybe_token.as_mut().ok_or(Error::<T>::Unknown)?;

			Self::decrease_balance(id, who, token_id, amount)?;

			let new_total_supply = token.total_supply.saturating_sub(amount);
			token.total_supply = new_total_supply;
			Ok(())
		})?;

		Self::deposit_event(Event::Burn(id, who.clone(), token_id, amount));

		Ok(())
	}

	pub fn do_batch_burn(
		who: &T::AccountId,
		id: T::MultiTokenId,
		token_ids: Vec<T::TokenId>,
		amounts: Vec<Balance>,
	) -> DispatchResult {
		ensure!(token_ids.len() == amounts.len(), Error::<T>::LengthMismatch);

		let n = token_ids.len();
		for i in 0..n {
			let token_id = token_ids[i];
			let amount = amounts[i];

			Tokens::<T>::try_mutate(id, |maybe_token| -> DispatchResult {
				let token = maybe_token.as_mut().ok_or(Error::<T>::Unknown)?;

				Self::decrease_balance(id, who, token_id, amount)?;

				let new_total_supply = token.total_supply.saturating_sub(amount);
				token.total_supply = new_total_supply;
				Ok(())
			})?;
		}

		Self::deposit_event(Event::BatchBurn(id, who.clone(), token_ids, amounts));

		Ok(())
	}

	pub fn do_transfer_from(
		who: &T::AccountId,
		id: T::MultiTokenId,
		from: &T::AccountId,
		to: &T::AccountId,
		token_id: T::TokenId,
		amount: Balance,
	) -> DispatchResult {
		ensure!(Self::owner_or_approved(id, &who, &from), Error::<T>::NotOwnerOrApproved);
		ensure!(
			Balances::<T>::get(id, (token_id, from.clone())) >= amount,
			Error::<T>::InsufficientTokens
		);

		if from == to {
			return Ok(());
		}

		Self::decrease_balance(id, from, token_id, amount)?;

		Self::increase_balance(id, to, token_id, amount)?;

		Self::deposit_event(Event::Transferred(id, from.clone(), to.clone(), token_id, amount));

		Ok(())
	}

	pub fn do_batch_transfer_from(
		who: &T::AccountId,
		id: T::MultiTokenId,
		from: &T::AccountId,
		to: &T::AccountId,
		token_ids: Vec<T::TokenId>,
		amounts: Vec<Balance>,
	) -> DispatchResult {
		ensure!(Self::owner_or_approved(id, &who, &from), Error::<T>::NotOwnerOrApproved);
		ensure!(token_ids.len() == amounts.len(), Error::<T>::LengthMismatch);

		if from == to {
			return Ok(());
		}

		let n = token_ids.len();
		for i in 0..n {
			let token_id = token_ids[i];
			let amount = amounts[i];

			ensure!(
				Balances::<T>::get(id, (token_id, from.clone())) >= amount,
				Error::<T>::InsufficientTokens
			);

			Self::decrease_balance(id, from, token_id, amount)?;

			Self::increase_balance(id, to, token_id, amount)?;
		}

		Self::deposit_event(Event::BatchTransferred(
			id,
			from.clone(),
			to.clone(),
			token_ids,
			amounts,
		));

		Ok(())
	}

	pub fn balance_of_batch(
		id: T::MultiTokenId,
		accounts: &Vec<T::AccountId>,
		token_ids: Vec<T::TokenId>,
	) -> Result<Vec<Balance>, DispatchError> {
		ensure!(accounts.len() == token_ids.len(), Error::<T>::LengthMismatch);

		let mut batch_balances = vec![Balance::from(0u128); accounts.len()];

		let n = accounts.len();
		for i in 0..n {
			let account = &accounts[i];
			let token_id = token_ids[i];

			batch_balances[i] = Self::balance_of(id, (token_id, account));
		}

		Ok(batch_balances)
	}

	fn increase_balance(
		id: T::MultiTokenId,
		to: &T::AccountId,
		token_id: T::TokenId,
		amount: Balance,
	) -> DispatchResult {
		Balances::<T>::try_mutate(id, (token_id, to), |balance| -> DispatchResult {
			*balance = balance.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;
		Ok(())
	}

	fn decrease_balance(
		id: T::MultiTokenId,
		from: &T::AccountId,
		token_id: T::TokenId,
		amount: Balance,
	) -> DispatchResult {
		Balances::<T>::try_mutate(id, (token_id, from), |balance| -> DispatchResult {
			*balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Ok(())
	}

	fn owner_or_approved(id: T::MultiTokenId, who: &T::AccountId, owner: &T::AccountId) -> bool {
		*who == *owner || Self::is_approved_for_all(id, (owner, who))
	}

	fn has_permission(id: T::MultiTokenId, who: &T::AccountId) -> bool {
		let token = Tokens::<T>::get(id).unwrap();
		*who == token.owner
	}
}

impl<T: Config> MultiMetadata for Pallet<T>
where
	T::TokenId: From<u128> + Into<u128>,
{
	type MultiTokenId = T::MultiTokenId;
	type TokenId = T::TokenId;

	fn uri(id: Self::MultiTokenId, token_id: T::TokenId) -> Vec<u8> {
		let base_uri_buf: Vec<u8> = Tokens::<T>::get(id).unwrap().uri.to_vec();
		let token_id: u128 = token_id.into();
		let token_id_buf: Vec<u8> = token_id.to_be_bytes().to_vec();
		base_uri_buf.into_iter().chain(token_id_buf).collect::<Vec<_>>()
	}
}
