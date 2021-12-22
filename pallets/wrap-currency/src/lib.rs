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
use frame_support::{traits::Get, PalletId};
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, CurrencyId};
use scale_info::TypeInfo;
use sp_runtime::{traits::AccountIdConversion, RuntimeDebug};
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct WrapToken<FungibleTokenId> {
	token_id: FungibleTokenId,
	total_supply: Balance,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_token_fungible::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		// #[pallet::constant]
		type PalletId: Get<PalletId>;

		type Currency: MultiCurrencyExtended<
			Self::AccountId,
			CurrencyId = CurrencyId,
			Balance = Balance,
		>;

		#[pallet::constant]
		type CreateWrapTokenDeposit: Get<Balance>;

		#[pallet::constant]
		type GetNativeCurrencyId: Get<CurrencyId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type WrapTokens<T: Config> =
		StorageMap<_, Blake2_128Concat, CurrencyId, WrapToken<T::FungibleTokenId>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenCreated(CurrencyId, T::AccountId),
		TokenMint(CurrencyId, Balance, T::AccountId),
		TokenBurn(CurrencyId, Balance, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		Unknown,
		NumOverflow,
		WrapTokenNotFound,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_wrap_token(origin: OriginFor<T>, currency_id: CurrencyId) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let vault_account = Self::account_id();

			let token_id = pallet_token_fungible::Pallet::<T>::do_create_token(
				&vault_account,
				[].to_vec(),
				[].to_vec(),
				18,
			)?;

			let wrap_token = WrapToken { token_id, total_supply: Default::default() };

			WrapTokens::<T>::insert(currency_id, wrap_token);

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn deposit(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(WrapTokens::<T>::contains_key(currency_id), Error::<T>::WrapTokenNotFound);

			let vault_account = Self::account_id();

			<T as Config>::Currency::transfer(currency_id, &who, &vault_account, amount)?;

			WrapTokens::<T>::try_mutate(currency_id, |wrap_token| -> DispatchResult {
				let token = wrap_token.as_mut().ok_or(Error::<T>::Unknown)?;

				pallet_token_fungible::Pallet::<T>::do_mint(
					token.token_id,
					&who,
					who.clone(),
					amount,
				)?;

				token.total_supply =
					token.total_supply.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;

			Self::deposit_event(Event::TokenMint(currency_id, amount, who));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn withdraw(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			WrapTokens::<T>::try_mutate(currency_id, |wrap_token| -> DispatchResult {
				let token = wrap_token.as_mut().ok_or(Error::<T>::Unknown)?;

				let vault_account = Self::account_id();

				<T as Config>::Currency::transfer(currency_id, &vault_account, &who, amount)?;

				pallet_token_fungible::Pallet::<T>::do_burn(token.token_id, &who, amount)?;

				token.total_supply =
					token.total_supply.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;

			Self::deposit_event(Event::TokenBurn(currency_id, amount, who));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> T::AccountId {
		<T as pallet::Config>::PalletId::get().into_account()
	}
}
