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
	traits::{
		Currency,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		Get, ReservableCurrency, Time,
	},
	PalletId,
};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	Percent, RuntimeDebug,
};
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type MomentOf<T> = <<T as Config>::Time as Time>::Moment;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

type AssetOf<T> = Asset<
	<T as pallet_token_non_fungible::Config>::NonFungibleTokenId,
	<T as pallet_token_non_fungible::Config>::TokenId,
	<T as pallet_token_multi::Config>::MultiTokenId,
	<T as pallet_token_multi::Config>::TokenId,
>;

type OrderOf<T> =
	Order<<T as frame_system::Config>::AccountId, BalanceOf<T>, MomentOf<T>, AssetOf<T>>;

type BidOf<T> = Bid<<T as frame_system::Config>::AccountId, BalanceOf<T>, MomentOf<T>>;

pub type OrderId = u32;
pub type BidId = u32;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct NFTAsset<NFTCollectionId, NFTokenId> {
	pub collection_id: NFTCollectionId,
	pub token_id: NFTokenId,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct MTAsset<MTCollectionId, MTokenId> {
	pub collection_id: MTCollectionId,
	pub token_id: MTokenId,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Asset<NFTCollectionId, NFTokenId, MTCollectionId, MTokenId> {
	NFT(NFTAsset<NFTCollectionId, NFTokenId>),
	MT(MTAsset<MTCollectionId, MTokenId>),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Order<AccountId, Balance, Moment, AssetOf> {
	pub id: OrderId,
	pub seller: AccountId,
	pub asset: AssetOf,
	pub price: Balance,
	pub expires_at: Moment,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Bid<AccountId, Balance, Moment> {
	pub id: BidId,
	pub bidder: AccountId,
	pub price: Balance,
	pub expires_at: Moment,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Royalty<AccountId> {
	share_cut: Percent,
	beneficiary: AccountId,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::ExistenceRequirement::AllowDeath;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_token_non_fungible::Config + pallet_token_multi::Config
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Time: Time;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type FeesCollectorShareCut: Get<Percent>;

		#[pallet::constant]
		type FeesCollector: Get<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Orders<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetOf<T>, OrderOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_order_id)]
	pub(super) type NextOrderId<T: Config> = StorageValue<_, OrderId, ValueQuery>;

	#[pallet::storage]
	pub(super) type Bids<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetOf<T>, BidOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_bid_id)]
	pub(super) type NextBidId<T: Config> = StorageValue<_, BidId, ValueQuery>;

	#[pallet::storage]
	pub(super) type Royalties<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetOf<T>, Royalty<T::AccountId>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OrderCreated(OrderId, T::AccountId, AssetOf<T>),
		OrderCancelled(OrderId),
		OrderExecuted(OrderId, T::AccountId, AssetOf<T>, BalanceOf<T>),
		BidCreated(BidId, T::AccountId, AssetOf<T>),
		BidCancelled(BidId),
		BidAccepted(OrderId, T::AccountId, AssetOf<T>, BalanceOf<T>),
		RoyaltiesSet(AssetOf<T>, Percent, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AssetNotFound,
		OrderNotFound,
		BidNotFound,
		NotOwner,
		ZeroPrice,
		InvalidExpiresAt,
		NoAvailableOrderId,
		NeedHigherPrice,
		NoAvailableBidId,
		NoPermission,
		InvalidPrice,
		CallerIsSeller,
		OrderExpired,
		BidExpired,
	}
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_order(
			origin: OriginFor<T>,
			asset: AssetOf<T>,
			price: BalanceOf<T>,
			expires_at: MomentOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(price > Zero::zero(), Error::<T>::ZeroPrice);
			ensure!(expires_at > T::Time::now(), Error::<T>::InvalidExpiresAt);

			// check owner
			match asset {
				Asset::NFT(ref a) => {
					let owner = pallet_token_non_fungible::Pallet::<T>::owner_of(
						a.collection_id,
						a.token_id,
					)
					.ok_or(Error::<T>::AssetNotFound)?;
					ensure!(owner == who, Error::<T>::NotOwner);
					pallet_token_non_fungible::Pallet::<T>::do_transfer_from(
						&who,
						a.collection_id,
						&who,
						&Self::account_id(),
						a.token_id,
					)?;
				},
				Asset::MT(ref a) => {
					let balance = pallet_token_multi::Pallet::<T>::balance_of(
						a.collection_id,
						(a.token_id, &who),
					);
					ensure!(balance > Zero::zero(), Error::<T>::NotOwner);
					pallet_token_multi::Pallet::<T>::do_transfer_from(
						&who,
						a.collection_id,
						&who,
						&Self::account_id(),
						a.token_id,
						One::one(),
					)?;
				},
			}

			let id = NextOrderId::<T>::try_mutate(|id| -> Result<OrderId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableOrderId)?;
				Ok(current_id)
			})?;

			let order = Order { id, seller: who.clone(), asset: asset.clone(), price, expires_at };

			Orders::<T>::insert(asset.clone(), order);

			Self::deposit_event(Event::OrderCreated(id, who, asset));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_order(origin: OriginFor<T>, asset: AssetOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let order = Orders::<T>::get(&asset).ok_or(Error::<T>::OrderNotFound)?;
			ensure!(order.seller == who, Error::<T>::NoPermission);

			let maybe_bid = Bids::<T>::get(&asset);
			if let Some(bid) = maybe_bid {
				Self::do_cancel_bid(&asset, bid)?;
			}

			Self::transfer_asset_to(&order, &who)?;

			Orders::<T>::remove(order.asset);

			Self::deposit_event(Event::OrderCancelled(order.id));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn execute_order(
			origin: OriginFor<T>,
			asset: AssetOf<T>,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let order = Orders::<T>::get(&asset).ok_or(Error::<T>::OrderNotFound)?;

			ensure!(order.price == price, Error::<T>::InvalidPrice);
			ensure!(order.seller != who, Error::<T>::CallerIsSeller);
			ensure!(T::Time::now() < order.expires_at, Error::<T>::OrderExpired);

			// Transfer share fees
			let saled_amount = Self::transfer_share_fees(&who, &asset, price)?;

			// Transfer sale amount to seller
			<T as Config>::Currency::transfer(&who, &order.seller, saled_amount, KeepAlive)?;

			let maybe_bid = Bids::<T>::get(&asset);
			if let Some(bid) = maybe_bid {
				Self::do_cancel_bid(&asset, bid)?;
			}

			Self::transfer_asset_to(&order, &who)?;

			Orders::<T>::remove(order.asset);

			Self::deposit_event(Event::OrderExecuted(order.id, who, asset, price));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn place_bid(
			origin: OriginFor<T>,
			asset: AssetOf<T>,
			price: BalanceOf<T>,
			expires_at: MomentOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// check order validity
			let order = Orders::<T>::get(&asset).ok_or(Error::<T>::OrderNotFound)?;

			// check on expire time
			let expires_at =
				if expires_at > order.expires_at { order.expires_at } else { expires_at };

			let maybe_bid = Bids::<T>::get(&asset);

			// if theres no previous bid, just check price > 0
			if let Some(bid) = maybe_bid {
				if bid.expires_at >= T::Time::now() {
					ensure!(price > bid.price, Error::<T>::NeedHigherPrice);
				} else {
					ensure!(price > Zero::zero(), Error::<T>::ZeroPrice);
				}

				Self::do_cancel_bid(&asset, bid)?;
			} else {
				ensure!(price > Zero::zero(), Error::<T>::ZeroPrice);
			}

			<T as Config>::Currency::transfer(&who, &Self::account_id(), price, KeepAlive)?;

			let id = NextBidId::<T>::try_mutate(|id| -> Result<BidId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableBidId)?;
				Ok(current_id)
			})?;

			let bid = Bid { id, bidder: who.clone(), price, expires_at };

			Bids::<T>::insert(asset.clone(), bid);

			Self::deposit_event(Event::BidCreated(id, who, asset));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_bid(origin: OriginFor<T>, asset: AssetOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bid = Bids::<T>::get(&asset).ok_or(Error::<T>::BidNotFound)?;

			ensure!(bid.bidder == who, Error::<T>::NoPermission);

			Self::do_cancel_bid(&asset, bid)
		}

		#[pallet::weight(10_000)]
		pub fn accept_bid(
			origin: OriginFor<T>,
			asset: AssetOf<T>,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let order = Orders::<T>::get(&asset).ok_or(Error::<T>::OrderNotFound)?;
			ensure!(order.seller == who, Error::<T>::NoPermission);

			let bid = Bids::<T>::get(&asset).ok_or(Error::<T>::BidNotFound)?;

			ensure!(bid.price == price, Error::<T>::InvalidPrice);
			ensure!(T::Time::now() <= bid.expires_at, Error::<T>::BidExpired);

			// Transfer share fees
			let saled_amount = Self::transfer_share_fees(&who, &asset, price)?;

			// Transfer bid amount to seller
			<T as Config>::Currency::transfer(
				&Self::account_id(),
				&order.seller,
				saled_amount,
				AllowDeath,
			)?;

			Self::transfer_asset_to(&order, &bid.bidder)?;

			Orders::<T>::remove(asset.clone());

			Self::deposit_event(Event::BidAccepted(bid.id, who, asset, price));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_royalties(
			origin: OriginFor<T>,
			asset: AssetOf<T>,
			share_cut: Percent,
			beneficiary: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			let royalty = Royalty { share_cut, beneficiary: beneficiary.clone() };

			Royalties::<T>::insert(asset.clone(), royalty);

			Self::deposit_event(Event::RoyaltiesSet(asset, share_cut, beneficiary));
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn account_id() -> T::AccountId {
		<T as pallet::Config>::PalletId::get().into_account()
	}

	fn transfer_share_fees(
		who: &T::AccountId,
		asset: &AssetOf<T>,
		price: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let mut saled_amount = price;

		// Royalties share fee
		let maybe_royalty = Royalties::<T>::get(&asset);
		if let Some(royalty) = maybe_royalty {
			let royalty_share_amount = royalty.share_cut * price;
			<T as Config>::Currency::transfer(
				who,
				&royalty.beneficiary,
				royalty_share_amount,
				KeepAlive,
			)?;
			saled_amount -= royalty_share_amount;
		}

		// Fees collector share
		let fees_collector = T::FeesCollector::get();
		let fees_collector_share_cut = T::FeesCollectorShareCut::get();
		let fees_collector_share_amount = fees_collector_share_cut * price;
		<T as Config>::Currency::transfer(
			who,
			&fees_collector,
			fees_collector_share_amount,
			KeepAlive,
		)?;
		saled_amount -= fees_collector_share_amount;

		Ok(saled_amount)
	}

	fn transfer_asset_to(order: &OrderOf<T>, to: &T::AccountId) -> DispatchResult {
		match &order.asset {
			Asset::NFT(a) => {
				pallet_token_non_fungible::Pallet::<T>::do_transfer_from(
					&Self::account_id(),
					a.collection_id,
					&Self::account_id(),
					to,
					a.token_id,
				)?;
			},
			Asset::MT(a) => {
				pallet_token_multi::Pallet::<T>::do_transfer_from(
					&Self::account_id(),
					a.collection_id,
					&Self::account_id(),
					to,
					a.token_id,
					One::one(),
				)?;
			},
		}
		Ok(())
	}

	fn do_cancel_bid(asset: &AssetOf<T>, bid: BidOf<T>) -> DispatchResult {
		<T as Config>::Currency::transfer(&Self::account_id(), &bid.bidder, bid.price, AllowDeath)?;

		Bids::<T>::remove(asset.clone());

		Self::deposit_event(Event::BidCancelled(bid.id));
		Ok(())
	}
}
