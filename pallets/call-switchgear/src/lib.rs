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
#![allow(clippy::unused_unit)]
#![allow(deprecated)] // TODO: clean transactional

extern crate alloc;

use frame_support::{
	dispatch::{CallMetadata, GetCallMetadata},
	pallet_prelude::*,
	traits::{Contains, PalletInfoAccess},
};
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::prelude::*;

pub use pallet::*;
pub mod weights;
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// can not switch off
		CannotSwitchOff,
		/// Invalid character
		InvalidCharacter,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Switch off transaction . \[pallet_name, function_name\]
		TransactionSwitchedoff(Vec<u8>, Vec<u8>),
		/// Switch on transaction . \[pallet_name, function_name\]
		TransactionSwitchedOn(Vec<u8>, Vec<u8>),
	}

	/// Controls whether or not all of the pallets are banned.
	#[pallet::storage]
	#[pallet::getter(fn get_overall_indicator)]
	pub(crate) type OverallToggle<T: Config> = StorageValue<_, bool, ValueQuery, DefaultStatus>;

	// Defult release amount is 30 KSM
	#[pallet::type_value]
	pub fn DefaultStatus() -> bool {
		false
	}

	#[pallet::storage]
	#[pallet::getter(fn get_switchoff_transactions)]
	pub type SwitchedOffTransactions<T: Config> =
		StorageMap<_, Twox64Concat, (Vec<u8>, Vec<u8>), (), OptionQuery>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::switchoff_transaction())]
		pub fn switchoff_transaction(
			origin: OriginFor<T>,
			pallet_name: Vec<u8>,
			function_name: Vec<u8>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let pallet_name_string =
				sp_std::str::from_utf8(&pallet_name).map_err(|_| Error::<T>::InvalidCharacter)?;
			ensure!(
				pallet_name_string != <Self as PalletInfoAccess>::name(),
				Error::<T>::CannotSwitchOff
			);

			// If "all" received, ban all of the pallets. Otherwise, only the passed-in pallet.
			if pallet_name_string.to_lowercase() == "all" {
				OverallToggle::<T>::put(true);
			} else {
				SwitchedOffTransactions::<T>::mutate_exists(
					(pallet_name.clone(), function_name.clone()),
					|item| {
						if item.is_none() {
							*item = Some(());
						}
					},
				);
			}

			Self::deposit_event(Event::TransactionSwitchedoff(pallet_name, function_name));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::switchon_transaction())]
		pub fn switchon_transaction(
			origin: OriginFor<T>,
			pallet_name: Vec<u8>,
			function_name: Vec<u8>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let pallet_name_string =
				sp_std::str::from_utf8(&pallet_name).map_err(|_| Error::<T>::InvalidCharacter)?;

			if pallet_name_string.to_lowercase() == "all" {
				OverallToggle::<T>::put(false);
			} else {
				SwitchedOffTransactions::<T>::take((pallet_name.clone(), &function_name.clone()));
			}

			Self::deposit_event(Event::TransactionSwitchedOn(pallet_name, function_name));

			Ok(())
		}
	}
}

pub struct SwitchOffTransactionFilter<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> Contains<T::Call> for SwitchOffTransactionFilter<T>
where
	<T as frame_system::Config>::Call: GetCallMetadata,
{
	fn contains(call: &T::Call) -> bool {
		let CallMetadata { function_name, pallet_name } = call.get_call_metadata();
		SwitchedOffTransactions::<T>::contains_key((
			pallet_name.as_bytes(),
			function_name.as_bytes(),
		))
	}
}

pub struct OverallToggleFilter<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OverallToggleFilter<T> {
	#[allow(dead_code)]
	pub fn get_overall_toggle_status() -> bool {
		OverallToggle::<T>::get()
	}
}
