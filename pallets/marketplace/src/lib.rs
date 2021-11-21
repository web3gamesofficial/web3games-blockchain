// This file is part of Web3Games.

// Copyright (C) 2021 Web3Games https://web3games.org
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
	BoundedVec,
};
use primitives::{Balance, TokenId};
use scale_info::TypeInfo;
use sp_runtime::{traits::One, RuntimeDebug};
use sp_std::{convert::TryInto, prelude::*};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type CollectionId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum NftType {
	NonFungibleToken,
	MultiToken,
}

/// Collection info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Collection<AccountId, BoundedString> {
	/// Class owner
	pub owner: AccountId,
	// The type of nft
	pub nft_type: NftType,
	/// The account of nft
	pub nft_account: AccountId,
	/// Metadata from ipfs
	pub metadata: BoundedString,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The maximum length of metadata stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// The minimum balance to create collection
		#[pallet::constant]
		type CreateCollectionDeposit: Get<BalanceOf<Self>>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Collections<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		CollectionId,
		Collection<T::AccountId, BoundedVec<u8, T::StringLimit>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_collection_id)]
	pub(super) type NextCollectionId<T: Config> = StorageValue<_, CollectionId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CollectionCreated(CollectionId, T::AccountId),
		CollectionDestroyed(CollectionId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NumOverflow,
		NoAvailableCollectionId,
		CollectionNotFound,
		NoAvailableAssetId,
		AssetNotFound,
		InvalidQuantity,
		NoPermission,
		CannotDestroyCollection,
		BadMetadata,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_collection(
			origin: OriginFor<T>,
			nft_type: NftType,
			nft_account: T::AccountId,
			metadata: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_collection(&who, nft_type, &nft_account, metadata)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn add_sale(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			token_id: TokenId,
			price: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_add_sale(&who, collection_id, token_id, price)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn remove_sale(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_remove_sale(&who, collection_id, token_id)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn update_price(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			token_id: TokenId,
			price: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_update_price(&who, collection_id, token_id, price)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn offer(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_offer(&who, collection_id, token_id)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn accept_offer(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_accept_offer(&who, collection_id, token_id)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn destroy_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_destroy_collection(&who, collection_id)?;

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn do_create_collection(
		who: &T::AccountId,
		nft_type: NftType,
		nft_account: &T::AccountId,
		metadata: Vec<u8>,
	) -> Result<CollectionId, DispatchError> {
		let bounded_metadata: BoundedVec<u8, T::StringLimit> =
			metadata.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		let collection_id =
			NextCollectionId::<T>::try_mutate(|id| -> Result<CollectionId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableCollectionId)?;
				Ok(current_id)
			})?;

		let deposit = T::CreateCollectionDeposit::get();
		T::Currency::reserve(who, deposit.clone())?;

		let collection = Collection {
			owner: who.clone(),
			nft_type,
			nft_account: nft_account.clone(),
			metadata: bounded_metadata,
		};

		Collections::<T>::insert(collection_id, collection);

		Self::deposit_event(Event::CollectionCreated(collection_id, who.clone()));
		Ok(collection_id)
	}

	pub fn do_add_sale(
		_who: &T::AccountId,
		_collection_id: CollectionId,
		_token_id: TokenId,
		_price: Balance,
	) -> DispatchResult {
		Ok(())
	}

	pub fn do_remove_sale(
		_who: &T::AccountId,
		_collection_id: CollectionId,
		_token_id: TokenId,
	) -> DispatchResult {
		Ok(())
	}

	pub fn do_update_price(
		_who: &T::AccountId,
		_collection_id: CollectionId,
		_token_id: TokenId,
		_price: Balance,
	) -> DispatchResult {
		Ok(())
	}

	pub fn do_offer(
		_who: &T::AccountId,
		_collection_id: CollectionId,
		_token_id: TokenId,
	) -> DispatchResult {
		Ok(())
	}

	pub fn do_accept_offer(
		_who: &T::AccountId,
		_collection_id: CollectionId,
		_token_id: TokenId,
	) -> DispatchResult {
		Ok(())
	}

	pub fn do_destroy_collection(
		who: &T::AccountId,
		collection_id: CollectionId,
	) -> DispatchResult {
		Collections::<T>::try_mutate_exists(collection_id, |collection| -> DispatchResult {
			let c = collection.take().ok_or(Error::<T>::CollectionNotFound)?;
			ensure!(c.owner == *who, Error::<T>::NoPermission);

			let deposit = T::CreateCollectionDeposit::get();
			T::Currency::unreserve(who, deposit);

			Self::deposit_event(Event::CollectionDestroyed(collection_id, who.clone()));

			Ok(())
		})
	}
}
