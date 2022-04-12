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
	traits::{ConstU32, Get},
	BoundedVec,
};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type Address = BoundedVec<u8, ConstU32<32>>;
pub type PlayerId = BoundedVec<u8, ConstU32<32>>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Chains {
	W3G,
	BTC,
	ETH,
	DOT,
	SOL,
	MATIC,
	BNB,
	NEAR,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type MaxAddressesPerChain: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type PlayerIdOf<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Address,
		Blake2_128Concat,
		Chains,
		PlayerId,
		OptionQuery,
	>;

	#[pallet::storage]
	pub(super) type Addresses<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		PlayerId,
		Blake2_128Concat,
		Chains,
		BoundedVec<Address, T::MaxAddressesPerChain>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Registered(PlayerId, T::AccountId),
		AddressAdded(Address, Chains, PlayerId),
		AddressRemoved(Address, Chains, PlayerId),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unknown,
		NotFound,
		PlayerIdTooLong,
		AddressTooLong,
		PlayerIdAlreadyRegistered,
		AddressAlreadyExists,
		NoPermission,
		TooManyAddresses,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T::AccountId: AsRef<[u8]>,
	{
		#[pallet::weight(10_000)]
		pub fn register(
			origin: OriginFor<T>,
			player_id: Vec<u8>,
			owner: T::AccountId,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let player_id: PlayerId =
				player_id.clone().try_into().map_err(|_| Error::<T>::PlayerIdTooLong)?;

			let chain = Self::native_chain();
			ensure!(
				Addresses::<T>::get(player_id.clone(), chain.clone()).is_none(),
				Error::<T>::PlayerIdAlreadyRegistered
			);

			let address: Address = Self::account_to_address(owner.clone());

			let addresses = BoundedVec::try_from(vec![address.clone()]).unwrap();
			Addresses::<T>::insert(player_id.clone(), chain.clone(), addresses);

			PlayerIdOf::<T>::insert(address, chain.clone(), player_id.clone());

			Self::deposit_event(Event::Registered(player_id, owner));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn add_address(
			origin: OriginFor<T>,
			player_id: PlayerId,
			chain: Chains,
			address: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::check_permission(who, player_id.clone()), Error::<T>::NoPermission);

			let address: Address =
				address.clone().try_into().map_err(|_| Error::<T>::AddressTooLong)?;

			Addresses::<T>::try_mutate(
				player_id.clone(),
				chain.clone(),
				|maybe_addresses| -> DispatchResult {
					match maybe_addresses {
						Some(ref mut a) => {
							ensure!(!a.contains(&address), Error::<T>::AddressAlreadyExists);
							a.try_push(address.clone())
								.map_err(|_| Error::<T>::TooManyAddresses)?;
						},
						maybe_addresses @ None => {
							*maybe_addresses =
								Some(BoundedVec::try_from(vec![address.clone()]).unwrap());
						},
					};
					Ok(())
				},
			)?;

			PlayerIdOf::<T>::insert(address.clone(), chain.clone(), player_id.clone());

			Self::deposit_event(Event::AddressAdded(address, chain, player_id));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn remove_address(
			origin: OriginFor<T>,
			player_id: PlayerId,
			chain: Chains,
			address: Address,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::check_permission(who, player_id.clone()), Error::<T>::NoPermission);

			Addresses::<T>::try_mutate_exists(
				player_id.clone(),
				chain.clone(),
				|maybe_addresses| -> DispatchResult {
					let mut addresses = maybe_addresses.take().ok_or(Error::<T>::Unknown)?;

					let pos = addresses
						.binary_search_by_key(&address, |a| a.clone())
						.map_err(|_| Error::<T>::NotFound)?;
					addresses.remove(pos);

					*maybe_addresses = Some(addresses);
					PlayerIdOf::<T>::remove(address.clone(), chain.clone());

					Ok(())
				},
			)?;

			Self::deposit_event(Event::AddressRemoved(address, chain, player_id));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T>
where
	T::AccountId: AsRef<[u8]>,
{
	fn account_to_address(account: T::AccountId) -> Address {
		BoundedVec::try_from(account.as_ref().to_vec()).unwrap()
	}

	fn native_chain() -> Chains {
		Chains::W3G
	}

	fn check_permission(who: T::AccountId, player_id: PlayerId) -> bool {
		let maybe_addresses = Addresses::<T>::get(player_id.clone(), Self::native_chain());
		let owner_address = Self::account_to_address(who);

		match maybe_addresses {
			Some(a) => a.contains(&owner_address),
			None => false,
		}
	}
}
