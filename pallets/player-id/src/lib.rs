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


pub use pallet::*;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_std::{convert::TryFrom, prelude::*};
use sp_std::vec::Vec;

pub use weights::WeightInfo;

pub mod weights;


type Address = Vec<u8>;
type PlayerID = Vec<u8>;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo, Copy)]
pub enum Chain {
	Ethereum,
	BSC,
	Polygon
}


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use log::error;
	use sp_core::crypto::AccountId32;
	use sp_core::ecdsa::Public;
	use sp_runtime::MultiSignature;
	// use crate::ethereum::EthereumSignature;
	use crate::WeightInfo;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn playerchain)]
	pub(super) type PlayerChain<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Address,
		Blake2_128Concat,
		Chain,
		PlayerID
	>;

	#[pallet::storage]
	#[pallet::getter(fn chainplayer)]
	pub(super) type ChainPlayer<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		PlayerID,
		Blake2_128Concat,
		Chain,
		Vec<Address>
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PlayerIDCreated(T::AccountId,Address,Chain,PlayerID),
		PlayerIDCheck(T::AccountId,bool),
		PlayerIDRemove(T::AccountId,bool),
		PlayerIDUpdate(T::AccountId,bool)
	}

	#[pallet::error]
	pub enum Error<T> {
		Repeat,
		NODATA
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_player_id(
			origin: OriginFor<T>,
			address:Vec<u8>,
			chain:Chain,
			player_id:PlayerID
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let repeat_data = ChainPlayer::<T>::get(player_id.clone(),chain.clone());

			let repeat_result:bool = match repeat_data {
				Some(repeat_value) =>{
					repeat_value.contains(&address)
				}
				None => {
					false
				}
			};

			ensure!(repeat_result == false,Error::<T>::Repeat);

			PlayerChain::<T>::insert(address.clone(),chain.clone(),player_id.clone());

			let new_address_list = ChainPlayer::<T>::get(player_id.clone(), chain.clone());

			match new_address_list {
				Some(mut address_list) =>{
					address_list.push(address.clone());
					ChainPlayer::<T>::insert(player_id.clone(),chain.clone(),address_list);
					Self::deposit_event(Event::PlayerIDCreated(who,address,chain,player_id));
				}
				None =>{
					let address_list = vec![address.clone()];
					ChainPlayer::<T>::insert(player_id.clone(),chain.clone(),address_list);
					Self::deposit_event(Event::PlayerIDCreated(who,address,chain,player_id));
				}
			};
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn delete_player_id(
			origin: OriginFor<T>,
			player_id:PlayerID,
			chain:Chain,
			address:Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut all_address = ChainPlayer::<T>::get(player_id.clone(), chain).unwrap_or(vec![address.clone()]);

			let check = vec![address.clone()];

			ensure!(all_address != check,Error::<T>::NODATA);

			ensure!(all_address.contains(&address),Error::<T>::NODATA);

			for (index, value) in all_address.clone().iter().enumerate() {
							if address == *value{
								all_address.remove(index);
							}
			}

			PlayerChain::<T>::remove(address.clone(),chain);
			ChainPlayer::<T>::insert(player_id.clone(),chain,all_address);
			Self::deposit_event(Event::PlayerIDRemove(who,true));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn update_player_id(
			origin: OriginFor<T>,
			player_id:PlayerID,
			chain:Chain,
			before_address:Vec<u8>,
			after_address:Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut all_address = ChainPlayer::<T>::get(player_id.clone(), chain).unwrap_or(vec![before_address.clone()]);

			let check = vec![before_address.clone()];

			ensure!(all_address != check,Error::<T>::NODATA);


			for (index, value) in all_address.clone().iter().enumerate() {
				if before_address == *value{
					all_address.remove(index);
					all_address.push(after_address.clone());
				}
			}

			PlayerChain::<T>::remove(before_address.clone(),chain);
			ChainPlayer::<T>::insert(player_id.clone(),chain,all_address);
			Self::deposit_event(Event::PlayerIDUpdate(who,true));
			Ok(().into())
		}
	}
}
