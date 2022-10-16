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
	traits::{Currency, ExistenceRequirement::KeepAlive, Get},
	PalletId,
};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AccountIdConversion, One, Saturating, UniqueSaturatedFrom},
	RuntimeDebug,
};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub const MIN_DURATION: u32 = 100;
pub const MIN_PRICE: u32 = 10000;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type OrderOf<T> = Order<
	<T as frame_system::Config>::AccountId,
	BalanceOf<T>,
	<T as frame_system::Config>::BlockNumber,
>;

type NonFungibleGroupId = u128;
type NonFungibleTokenId = u128;
type MultiGroupId = u128;
type MultiTokenId = u128;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo, Copy)]
pub enum Asset {
	NonFungibleToken(NonFungibleGroupId, NonFungibleTokenId),
	MultiToken(MultiGroupId, MultiTokenId),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Order<AccountId, Balance, BlockNumber> {
	pub seller: AccountId,
	pub price: Balance,
	pub start: BlockNumber,
	pub duration: BlockNumber,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Bid<AccountId, Balance, BlockNumber> {
	pub bidder: AccountId,
	pub price: Balance,
	pub start: BlockNumber,
	pub duration: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_token_non_fungible::Config + pallet_token_multi::Config
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// The pallet admin key.
	#[pallet::storage]
	#[pallet::getter(fn admin_key)]
	pub(super) type Admin<T: Config> = StorageValue<_, T::AccountId>;

	/// The protocol fee point.
	/// [0, 255] -> [0%, 2.55%]
	#[pallet::storage]
	#[pallet::getter(fn point)]
	pub(super) type Point<T: Config> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub(super) type Orders<T: Config> =
		StorageMap<_, Blake2_128Concat, Asset, OrderOf<T>, OptionQuery>;

	#[pallet::storage]
	pub(super) type Bids<T: Config> =
		StorageMap<_, Blake2_128Concat, Asset, Bid<T::AccountId, BalanceOf<T>, T::BlockNumber>>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// The `AccountId` of the admin key.
		pub admin_key: Option<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { admin_key: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			if let Some(key) = &self.admin_key {
				<Admin<T>>::put(key.clone());
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OrderCreated(T::AccountId, Asset, OrderOf<T>),
		OrderCancelled(T::AccountId, Asset),
		OrderExecuted(T::AccountId, Asset, OrderOf<T>),
		BidCreated(T::AccountId, Asset, Bid<T::AccountId, BalanceOf<T>, T::BlockNumber>),
		BidCancelled(T::AccountId, Asset),
		BidAccepted(T::AccountId, Asset, Bid<T::AccountId, BalanceOf<T>, T::BlockNumber>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		OrderNotFound,
		BidNotFound,
		OrderExpired,
		BidExpired,
		TooLittleDuration,
		TooLittlePrice,
		NotBidder,
		NotSeller,
		NotAdmin,
		NotSetAdmin,
		NeedHigherPrice,
	}
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn set_admin(origin: OriginFor<T>, new_admin: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Admin::<T>::mutate(|admin| *admin = Some(new_admin));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn set_service_fee_point(origin: OriginFor<T>, new_point: u8) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_admin(who), Error::<T>::NotAdmin);

			Point::<T>::mutate(|point| *point = new_point);

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn create_order(
			origin: OriginFor<T>,
			asset: Asset,
			price: BalanceOf<T>,
			duration: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(price >= BalanceOf::<T>::from(MIN_PRICE), Error::<T>::TooLittlePrice);
			ensure!(duration >= T::BlockNumber::from(MIN_DURATION), Error::<T>::TooLittleDuration);

			// check owner
			Self::transfer_asset_to(who.clone(), asset, Self::account_id())?;

			let order = Order { seller: who.clone(), price, start: Self::now(), duration };

			Orders::<T>::insert(asset, order.clone());

			Self::deposit_event(Event::OrderCreated(who, asset, order));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_order(origin: OriginFor<T>, asset: Asset) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let order = Orders::<T>::get(asset).ok_or(Error::<T>::OrderNotFound)?;
			ensure!(order.seller == who, Error::<T>::NotSeller);

			//check Bids
			if let Some(bid) = Bids::<T>::get(asset) {
				Self::do_cancel_bid(asset, bid)?;
			}

			Self::transfer_asset_to(Self::account_id(), asset, who.clone())?;

			Orders::<T>::remove(asset);

			Self::deposit_event(Event::OrderCancelled(who, asset));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn execute_order(origin: OriginFor<T>, asset: Asset) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let order = Orders::<T>::get(asset).ok_or(Error::<T>::OrderNotFound)?;

			ensure!(order.start + order.duration >= Self::now(), Error::<T>::OrderExpired);

			// Transfer service fee
			let service_fee = match Admin::<T>::get() {
				Some(admin) => {
					let fee_point = Point::<T>::get();
					let service_fee = Self::calculate_service_fee(order.price, fee_point);

					// transfer `service_fee` to admin
					<T as pallet::Config>::Currency::transfer(
						&who,
						&admin,
						service_fee,
						KeepAlive,
					)?;

					service_fee
				},
				None => BalanceOf::<T>::default(),
			};

			//check Bids
			if let Some(bid) = Bids::<T>::get(asset) {
				Self::do_cancel_bid(asset, bid)?;
			}

			let to_seller = order.price.saturating_sub(service_fee);
			<T as pallet::Config>::Currency::transfer(&who, &order.seller, to_seller, KeepAlive)?;

			Self::transfer_asset_to(Self::account_id(), asset, who.clone())?;

			Orders::<T>::remove(asset);

			Self::deposit_event(Event::OrderExecuted(who, asset, order));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn place_bid(
			origin: OriginFor<T>,
			asset: Asset,
			price: BalanceOf<T>,
			duration: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// check order validity
			let order = Orders::<T>::get(asset).ok_or(Error::<T>::OrderNotFound)?;

			// check on expire time
			ensure!(order.start + order.duration >= Self::now(), Error::<T>::OrderExpired);
			//check price
			ensure!(price >= BalanceOf::<T>::from(MIN_PRICE), Error::<T>::TooLittlePrice);

			let bids = Bids::<T>::get(asset);

			// if theres no previous bid, just check price > 0
			if let Some(bid) = bids {
				if bid.start + bid.duration >= Self::now() {
					ensure!(price > bid.price, Error::<T>::NeedHigherPrice);
				}
				Self::do_cancel_bid(asset, bid)?;
			}

			//transfer price to admin
			let admin = Admin::<T>::get().ok_or(Error::<T>::NotSetAdmin)?;
			<T as Config>::Currency::transfer(&who, &admin, price, KeepAlive)?;

			let bid = Bid { bidder: who.clone(), price, start: Self::now(), duration };
			Bids::<T>::insert(asset, bid.clone());

			Self::deposit_event(Event::BidCreated(who, asset, bid));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_bid(origin: OriginFor<T>, asset: Asset) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bid = Bids::<T>::get(asset).ok_or(Error::<T>::BidNotFound)?;

			ensure!(bid.bidder == who, Error::<T>::NotBidder);

			Self::do_cancel_bid(asset, bid)
		}

		#[pallet::weight(10_000)]
		pub fn accept_bid(origin: OriginFor<T>, asset: Asset) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let order = Orders::<T>::get(asset).ok_or(Error::<T>::OrderNotFound)?;
			ensure!(order.seller == who, Error::<T>::NotSeller);

			let bid = Bids::<T>::get(asset).ok_or(Error::<T>::BidNotFound)?;
			ensure!(bid.start + bid.duration <= Self::now(), Error::<T>::BidExpired);

			let fee_point = Point::<T>::get();
			let service_fee = Self::calculate_service_fee(bid.price, fee_point);
			let to_seller = bid.price.saturating_sub(service_fee);

			// Transfer bid amount to seller
			let admin = Admin::<T>::get().ok_or(Error::<T>::NotSetAdmin)?;
			<T as Config>::Currency::transfer(&admin, &order.seller, to_seller, KeepAlive)?;

			Self::transfer_asset_to(Self::account_id(), asset, bid.bidder.clone())?;

			Orders::<T>::remove(asset);
			Bids::<T>::remove(asset);

			Self::deposit_event(Event::BidAccepted(who, asset, bid));
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn now() -> T::BlockNumber {
		frame_system::Pallet::<T>::block_number()
	}
	pub fn account_id() -> T::AccountId {
		<T as pallet::Config>::PalletId::get().into_account_truncating()
	}

	pub fn is_admin(who: T::AccountId) -> bool {
		matches!(Admin::<T>::get(), Some(admin) if admin == who)
	}

	pub fn calculate_service_fee(value: BalanceOf<T>, fee_point: u8) -> BalanceOf<T> {
		let point = BalanceOf::<T>::from(fee_point);
		let base_point = BalanceOf::<T>::from(10000u16);

		// Impossible to overflow
		value / base_point * point
	}

	fn transfer_asset_to(from: T::AccountId, asset: Asset, to: T::AccountId) -> DispatchResult {
		match asset {
			Asset::NonFungibleToken(group_id, token_id) => {
				pallet_token_non_fungible::Pallet::<T>::do_transfer_from(
					&from,
					<T as pallet_token_non_fungible::Config>::NonFungibleTokenId::unique_saturated_from(group_id),
					&from,
					&to,
					<T as pallet_token_non_fungible::Config>::TokenId::unique_saturated_from(token_id),
				)?;
			},
			Asset::MultiToken(group_id, token_id) => {
				pallet_token_multi::Pallet::<T>::do_transfer_from(
					&from,
					<T as pallet_token_multi::Config>::MultiTokenId::unique_saturated_from(
						group_id,
					),
					&from,
					&to,
					<T as pallet_token_multi::Config>::TokenId::unique_saturated_from(token_id),
					One::one(),
				)?;
			},
		}
		Ok(())
	}

	fn do_cancel_bid(
		asset: Asset,
		bid: Bid<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) -> DispatchResult {
		let admin = Admin::<T>::get().ok_or(Error::<T>::NotSetAdmin)?;
		<T as Config>::Currency::transfer(&admin, &bid.bidder, bid.price, KeepAlive)?;
		Bids::<T>::remove(asset);
		Self::deposit_event(Event::BidCancelled(bid.bidder, asset));
		Ok(())
	}
}
