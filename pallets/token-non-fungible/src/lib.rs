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
use pallet_support::{NonFungibleEnumerable, NonFungibleMetadata};
use primitives::TokenIndex;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, One, TrailingZeroInput},
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
	base_uri: BoundedString,
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

		/// Identifier for the class of token.
		type NonFungibleTokenId: Member
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
		T::NonFungibleTokenId,
		Token<T::AccountId, BoundedVec<u8, T::StringLimit>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub(super) type Owners<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		T::TokenId,
		T::AccountId,
		OptionQuery,
		GetDefault,
		ConstU32<300_000>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		T::AccountId,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_approved)]
	pub(super) type TokenApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		T::TokenId,
		T::AccountId,
		OptionQuery,
		GetDefault,
		ConstU32<300_000>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn is_approved_for_all)]
	pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		// (owner, operator)
		(T::AccountId, T::AccountId),
		bool,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type TotalSupply<T: Config> =
		StorageMap<_, Blake2_128Concat, T::NonFungibleTokenId, u32, ValueQuery>;

	#[pallet::storage]
	pub(super) type AllTokens<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		TokenIndex,
		T::TokenId,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type AllTokensIndex<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		T::TokenId,
		TokenIndex,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type OwnedTokens<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		(T::AccountId, TokenIndex),
		T::TokenId,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type OwnedTokensIndex<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		T::TokenId,
		TokenIndex,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenCreated(T::NonFungibleTokenId, T::AccountId, Vec<u8>, Vec<u8>, Vec<u8>),
		Transfer(T::NonFungibleTokenId, T::AccountId, T::AccountId, T::TokenId),
		Approval(T::NonFungibleTokenId, T::AccountId, T::AccountId, T::TokenId),
		ApprovalForAll(T::NonFungibleTokenId, T::AccountId, T::AccountId, bool),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoAvailableTokenId,
		Overflow,
		Underflow,
		TokenAlreadyMinted,
		InvalidId,
		NoPermission,
		NotTokenOwner,
		TokenNonExistent,
		ApproveToCurrentOwner,
		NotOwnerOrApproved,
		BadMetadata,
		ConfuseBehavior,
		TransferTokenNotOwn,
		NotFound,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_token(
			origin: OriginFor<T>,
			id: T::NonFungibleTokenId,
			name: Vec<u8>,
			symbol: Vec<u8>,
			base_uri: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_create_token(&who, id, name, symbol, base_uri)
		}

		#[pallet::weight(10_000)]
		pub fn approve(
			origin: OriginFor<T>,
			id: T::NonFungibleTokenId,
			to: T::AccountId,
			token_id: T::TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_approve(&who, id, &to, token_id)
		}

		#[pallet::weight(10_000)]
		pub fn set_approve_for_all(
			origin: OriginFor<T>,
			id: T::NonFungibleTokenId,
			operator: T::AccountId,
			approved: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_set_approve_for_all(&who, id, &operator, approved)
		}

		#[pallet::weight(10_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			id: T::NonFungibleTokenId,
			from: T::AccountId,
			to: T::AccountId,
			token_id: T::TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_transfer_from(&who, id, &from, &to, token_id)
		}

		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
			id: T::NonFungibleTokenId,
			to: T::AccountId,
			token_id: T::TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_mint(&who, id, &to, token_id)
		}

		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			id: T::NonFungibleTokenId,
			token_id: T::TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_burn(&who, id, token_id)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn zero_account_id() -> T::AccountId {
		T::AccountId::decode(&mut TrailingZeroInput::zeroes()).expect("infinite input; qed")
	}

	pub fn exists(id: T::NonFungibleTokenId) -> bool {
		Tokens::<T>::contains_key(id)
	}

	pub fn token_exists(id: T::NonFungibleTokenId, token_id: T::TokenId) -> bool {
		Owners::<T>::contains_key(id, token_id)
	}

	pub fn do_create_token(
		who: &T::AccountId,
		id: T::NonFungibleTokenId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		base_uri: Vec<u8>,
	) -> DispatchResult {
		ensure!(!Self::exists(id.clone()), Error::<T>::InvalidId);

		let deposit = T::CreateTokenDeposit::get();
		T::Currency::reserve(&who, deposit.clone())?;

		let bounded_name: BoundedVec<u8, T::StringLimit> =
			name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		let bounded_symbol: BoundedVec<u8, T::StringLimit> =
			symbol.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		let bounded_base_uri: BoundedVec<u8, T::StringLimit> =
			base_uri.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		let token = Token {
			owner: who.clone(),
			name: bounded_name,
			symbol: bounded_symbol,
			base_uri: bounded_base_uri,
		};

		Tokens::<T>::insert(id, token);

		Self::deposit_event(Event::TokenCreated(id, who.clone(), name, symbol, base_uri));

		Ok(())
	}

	pub fn do_approve(
		who: &T::AccountId,
		id: T::NonFungibleTokenId,
		to: &T::AccountId,
		token_id: T::TokenId,
	) -> DispatchResult {
		let owner = Self::owner_of(id, token_id).ok_or(Error::<T>::NotFound)?;
		ensure!(to != &owner, Error::<T>::ApproveToCurrentOwner);

		ensure!(
			who == &owner || Self::is_approved_for_all(id, (&owner, who)),
			Error::<T>::NotOwnerOrApproved
		);

		TokenApprovals::<T>::insert(id, token_id, to);

		Self::deposit_event(Event::Approval(id, owner, to.clone(), token_id));

		Ok(())
	}

	pub fn do_set_approve_for_all(
		who: &T::AccountId,
		id: T::NonFungibleTokenId,
		operator: &T::AccountId,
		approved: bool,
	) -> DispatchResult {
		ensure!(operator != who, Error::<T>::ApproveToCurrentOwner);

		OperatorApprovals::<T>::insert(id, (who, operator), approved);

		Self::deposit_event(Event::ApprovalForAll(id, who.clone(), operator.clone(), approved));

		Ok(())
	}

	pub fn do_transfer_from(
		who: &T::AccountId,
		id: T::NonFungibleTokenId,
		from: &T::AccountId,
		to: &T::AccountId,
		token_id: T::TokenId,
	) -> DispatchResult {
		ensure!(Self::token_exists(id, token_id), Error::<T>::TokenNonExistent);
		ensure!(
			Self::is_approved_or_owner(id, &who, token_id).unwrap(),
			Error::<T>::NotOwnerOrApproved
		);
		Self::do_transfer(id, from, to, token_id)?;
		Ok(())
	}

	pub fn do_transfer(
		id: T::NonFungibleTokenId,
		from: &T::AccountId,
		to: &T::AccountId,
		token_id: T::TokenId,
	) -> DispatchResult {
		ensure!(
			Owners::<T>::get(id, token_id) == Some(from.clone()),
			Error::<T>::TransferTokenNotOwn
		);

		if from == to {
			return Ok(());
		}

		let balance_from = Self::balance_of(id, from);
		let balance_to = Self::balance_of(id, to);

		let new_balance_from = match balance_from.checked_sub(1) {
			Some(c) => c,
			None => return Err(Error::<T>::Underflow.into()),
		};

		let new_balance_to = match balance_to.checked_add(1) {
			Some(c) => c,
			None => return Err(Error::<T>::Overflow.into()),
		};

		Self::remove_token_from_owner_enumeration(id, from, token_id)?;
		Self::add_token_to_owner_enumeration(id, to, token_id)?;

		Self::clear_approval(id, token_id)?;

		Balances::<T>::insert(id, from, new_balance_from);
		Balances::<T>::insert(id, to, new_balance_to);
		Owners::<T>::insert(id, token_id, to);

		Self::deposit_event(Event::Transfer(id.clone(), from.clone(), to.clone(), token_id));

		Ok(())
	}

	pub fn do_mint(
		who: &T::AccountId,
		id: T::NonFungibleTokenId,
		to: &T::AccountId,
		token_id: T::TokenId,
	) -> DispatchResult {
		ensure!(Self::has_permission(id, who), Error::<T>::NoPermission);
		ensure!(!Self::token_exists(id, token_id), Error::<T>::TokenAlreadyMinted);

		let balance = Self::balance_of(id, to);

		let new_balance = match balance.checked_add(One::one()) {
			Some(c) => c,
			None => return Err(Error::<T>::Overflow.into()),
		};

		Self::add_token_to_all_tokens_enumeration(id, token_id)?;
		Self::add_token_to_owner_enumeration(id, to, token_id)?;

		Balances::<T>::insert(id, to, new_balance);
		Owners::<T>::insert(id, token_id, to);

		Self::deposit_event(Event::Transfer(
			id.clone(),
			Self::zero_account_id(),
			to.clone(),
			token_id,
		));

		Ok(())
	}

	pub fn do_burn(
		who: &T::AccountId,
		id: T::NonFungibleTokenId,
		token_id: T::TokenId,
	) -> DispatchResult {
		let owner = Self::owner_of(id, token_id).ok_or(Error::<T>::NotFound)?;
		ensure!(who == &owner, Error::<T>::NotTokenOwner);

		let balance = Self::balance_of(id, &owner);

		let new_balance = match balance.checked_sub(One::one()) {
			Some(c) => c,
			None => return Err(Error::<T>::Underflow.into()),
		};

		Self::remove_token_from_all_tokens_enumeration(id, token_id)?;
		Self::remove_token_from_owner_enumeration(id, &owner, token_id)?;

		Self::clear_approval(id, token_id)?;

		Balances::<T>::insert(id, &owner, new_balance);
		Owners::<T>::remove(id, token_id);

		Self::deposit_event(Event::Transfer(id.clone(), owner, Self::zero_account_id(), token_id));

		Ok(())
	}

	fn is_approved_or_owner(
		id: T::NonFungibleTokenId,
		spender: &T::AccountId,
		token_id: T::TokenId,
	) -> Result<bool, DispatchError> {
		let owner = Self::owner_of(id, token_id).ok_or(Error::<T>::NotFound)?;

		Ok(*spender == owner
			|| Self::get_approved(id, token_id) == Some(spender.clone())
			|| Self::is_approved_for_all(id, (&owner, spender)))
	}

	fn has_permission(id: T::NonFungibleTokenId, who: &T::AccountId) -> bool {
		let token = Tokens::<T>::get(id).unwrap();
		*who == token.owner
	}

	fn clear_approval(id: T::NonFungibleTokenId, token_id: T::TokenId) -> DispatchResult {
		TokenApprovals::<T>::remove(id, token_id);
		Ok(())
	}

	fn add_token_to_owner_enumeration(
		id: T::NonFungibleTokenId,
		to: &T::AccountId,
		token_id: T::TokenId,
	) -> DispatchResult {
		let new_token_index = Self::balance_of(id, to);

		OwnedTokensIndex::<T>::insert(id, token_id, new_token_index);
		OwnedTokens::<T>::insert(id, (to, new_token_index), token_id);

		Ok(())
	}

	fn add_token_to_all_tokens_enumeration(
		id: T::NonFungibleTokenId,
		token_id: T::TokenId,
	) -> DispatchResult {
		TotalSupply::<T>::try_mutate(id, |total_supply| -> DispatchResult {
			let new_token_index = *total_supply;
			*total_supply = total_supply.checked_add(One::one()).ok_or(Error::<T>::Overflow)?;

			AllTokensIndex::<T>::insert(id, token_id, new_token_index);
			AllTokens::<T>::insert(id, new_token_index, token_id);

			Ok(())
		})?;

		Ok(())
	}

	fn remove_token_from_owner_enumeration(
		id: T::NonFungibleTokenId,
		from: &T::AccountId,
		token_id: T::TokenId,
	) -> DispatchResult {
		let balance_of_from = Self::balance_of(id, from);

		let last_token_index = match balance_of_from.checked_sub(One::one()) {
			Some(c) => c,
			None => return Err(Error::<T>::Overflow.into()),
		};

		let token_index = OwnedTokensIndex::<T>::get(id, token_id);

		if token_index != last_token_index {
			let last_token_id = OwnedTokens::<T>::get(id, (from, last_token_index));
			OwnedTokens::<T>::insert(id, (from, token_index), last_token_id);
			OwnedTokensIndex::<T>::insert(id, last_token_id, token_index);
		}

		OwnedTokensIndex::<T>::remove(id, token_id);
		OwnedTokens::<T>::remove(id, (from, last_token_index));

		Ok(())
	}

	fn remove_token_from_all_tokens_enumeration(
		id: T::NonFungibleTokenId,
		token_id: T::TokenId,
	) -> DispatchResult {
		let total_supply = Self::total_supply(id);

		let new_total_supply = match total_supply.checked_sub(One::one()) {
			Some(c) => c,
			None => return Err(Error::<T>::Overflow.into()),
		};

		let last_token_index = new_total_supply;

		let token_index = AllTokensIndex::<T>::get(id, token_id);

		let last_token_id = AllTokens::<T>::get(id, last_token_index);

		AllTokens::<T>::insert(id, token_index, last_token_id);
		AllTokensIndex::<T>::insert(id, last_token_id, token_index);

		AllTokens::<T>::remove(id, last_token_index);
		AllTokensIndex::<T>::remove(id, token_id);

		TotalSupply::<T>::insert(id, new_total_supply);

		Ok(())
	}
}

impl<T: Config> NonFungibleMetadata for Pallet<T>
where
	T::TokenId: From<u128> + Into<u128>,
{
	type NonFungibleTokenId = T::NonFungibleTokenId;
	type TokenId = T::TokenId;

	fn token_name(id: Self::NonFungibleTokenId) -> Vec<u8> {
		Tokens::<T>::get(id).unwrap().name.to_vec()
	}

	fn token_symbol(id: Self::NonFungibleTokenId) -> Vec<u8> {
		Tokens::<T>::get(id).unwrap().symbol.to_vec()
	}

	fn token_uri(id: Self::NonFungibleTokenId, token_id: Self::TokenId) -> Vec<u8> {
		let base_uri_buf: Vec<u8> = Tokens::<T>::get(id).unwrap().base_uri.to_vec();
		let token_id: u128 = token_id.into();
		let token_id_buf: Vec<u8> = token_id.to_be_bytes().to_vec();
		base_uri_buf.into_iter().chain(token_id_buf).collect::<Vec<_>>()
	}
}

impl<T: Config> NonFungibleEnumerable<T::AccountId> for Pallet<T> {
	type NonFungibleTokenId = T::NonFungibleTokenId;
	type TokenId = T::TokenId;

	fn total_supply(id: Self::NonFungibleTokenId) -> TokenIndex {
		TotalSupply::<T>::get(id)
	}
	fn token_by_index(id: Self::NonFungibleTokenId, index: TokenIndex) -> Self::TokenId {
		AllTokens::<T>::get(id, index)
	}
	fn token_of_owner_by_index(
		id: Self::NonFungibleTokenId,
		owner: T::AccountId,
		index: TokenIndex,
	) -> Self::TokenId {
		OwnedTokens::<T>::get(id, (owner, index))
	}
}
