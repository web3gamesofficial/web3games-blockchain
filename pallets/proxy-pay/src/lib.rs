// This file is part of Bifrost.

// Copyright (C) 2019-2022 Liebi Technologies (UK) Ltd.
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

use core::convert::Into;

use frame_support::{
	pallet_prelude::*,
	traits::{
		Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced, ReservableCurrency,
		WithdrawReasons,
	},
	transactional, PalletId,
};
use frame_system::{pallet_prelude::*, WeightInfo};

pub use pallet::*;
use pallet_transaction_payment::OnChargeTransaction;

use sp_runtime::{
	traits::{AccountIdConversion, DispatchInfoOf, PostDispatchInfoOf, Saturating, Zero},
	transaction_validity::TransactionValidityError,
};
use sp_std::vec::Vec;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
//
// mod mock;
// mod tests;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Program<AccountId> {
	pub owner: AccountId,
	pub proxy_pay_account: AccountId,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
		/// Event
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
		/// The currency type in which fees will be paid.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Handler for the unbalanced decrease
		type OnUnbalanced: OnUnbalanced<NegativeImbalanceOf<Self>>;

		type PalletId: Get<PalletId>;
	}

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type PalletBalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub type NegativeImbalanceOf<T> =
		<<T as Config>::Currency as Currency<AccountIdOf<T>>>::NegativeImbalance;
	pub type PositiveImbalanceOf<T> =
		<<T as Config>::Currency as Currency<AccountIdOf<T>>>::PositiveImbalance;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreateProgame(u128, Program<T::AccountId>),
		UpdateProgame(u128, Program<T::AccountId>),
		AddProxyPay(u128, T::AccountId, u128),
		SetProxyPay(u128, T::AccountId, u128),
	}

	#[pallet::storage]
	#[pallet::getter(fn next_program_id)]
	pub(super) type NextProgramId<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn program_info)]
	pub type ProgramInfo<T: Config> = StorageMap<_, Twox64Concat, u128, Program<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn account_programs)]
	pub type AccountPrograms<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<u128>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proxy_pay_times)]
	pub type ProxyPayTimes<T: Config> =
		StorageDoubleMap<_, Twox64Concat, u128, Blake2_128Concat, T::AccountId, u128, ValueQuery>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::error]
	pub enum Error<T> {
		NoAvailableProgramId,
		NoAuthorization,
		Overflow,
		ConversionError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10000)]
		#[transactional]
		pub fn create_program(origin: OriginFor<T>, owner: T::AccountId) -> DispatchResult {
			ensure_signed(origin)?;

			let id = NextProgramId::<T>::try_mutate(|id| -> Result<u128, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(1u128).ok_or(Error::<T>::NoAvailableProgramId)?;
				Ok(current_id)
			})?;
			let proxy_pay_account = Self::proxy_pay_account(id);

			let program = Program { owner, proxy_pay_account };

			ProgramInfo::<T>::insert(id, program.clone());

			// Deposit event.
			Pallet::<T>::deposit_event(Event::CreateProgame(id, program));

			Ok(())
		}

		#[pallet::weight(10000)]
		#[transactional]
		pub fn transfer_program_owner(
			origin: OriginFor<T>,
			program_id: u128,
			owner: T::AccountId,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let program_info =
				ProgramInfo::<T>::get(program_id).ok_or(Error::<T>::NoAvailableProgramId)?;
			ensure!(caller == program_info.owner, Error::<T>::NoAuthorization);

			let program = Program { owner, proxy_pay_account: program_info.proxy_pay_account };

			ProgramInfo::<T>::try_mutate(program_id, |p| {
				*p = Some(program.clone());
				Self::deposit_event(Event::UpdateProgame(program_id, program));
				Ok(())
			})
		}

		#[pallet::weight(10000)]
		#[transactional]
		pub fn add_proxy_pay(
			origin: OriginFor<T>,
			program_id: u128,
			accounts_times: Vec<(T::AccountId, u128)>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let program_info =
				ProgramInfo::<T>::get(program_id).ok_or(Error::<T>::NoAvailableProgramId)?;
			ensure!(caller == program_info.owner, Error::<T>::NoAuthorization);

			for (account, times) in accounts_times {
				if times != 0 {
					ProxyPayTimes::<T>::mutate(program_id, account.clone(), |time| {
						*time = *time + times;
					});

					AccountPrograms::<T>::mutate(account.clone(), |program_ids| {
						if !program_ids.contains(&program_id) {
							program_ids.push(program_id);
						}
					});

					// Deposit event.
					Pallet::<T>::deposit_event(Event::AddProxyPay(program_id, account, times));
				}
			}
			Ok(())
		}

		#[pallet::weight(10000)]
		#[transactional]
		pub fn set_proxy_pay(
			origin: OriginFor<T>,
			program_id: u128,
			accounts_times: Vec<(T::AccountId, u128)>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let program_info =
				ProgramInfo::<T>::get(program_id).ok_or(Error::<T>::NoAvailableProgramId)?;
			ensure!(caller == program_info.owner, Error::<T>::NoAuthorization);

			for (account, times) in accounts_times {
				ProxyPayTimes::<T>::insert(program_id, account.clone(), times);
				if times != 0 {
					AccountPrograms::<T>::mutate(account.clone(), |program_ids| {
						if !program_ids.contains(&program_id) {
							program_ids.push(program_id);
						}
					});
				} else {
					AccountPrograms::<T>::mutate(account.clone(), |program_ids| {
						if program_ids.contains(&program_id) {
							program_ids.retain(|p| *p != program_id);
						}
					});
				}
				// Deposit event.
				Pallet::<T>::deposit_event(Event::SetProxyPay(program_id, account, times));
			}
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn proxy_pay_account(program_id: u128) -> T::AccountId {
		<T as Config>::PalletId::get().into_sub_account_truncating(program_id)
	}
}

/// Default implementation for a Currency and an OnUnbalanced handler.
impl<T> OnChargeTransaction<T> for Pallet<T>
where
	T: Config,
	T::Currency: Currency<<T as frame_system::Config>::AccountId>,
	// <T as frame_system::Config>::AccountId: Copy,
	PositiveImbalanceOf<T>: Imbalance<PalletBalanceOf<T>, Opposite = NegativeImbalanceOf<T>>,
	NegativeImbalanceOf<T>: Imbalance<PalletBalanceOf<T>, Opposite = PositiveImbalanceOf<T>>,
{
	type Balance = PalletBalanceOf<T>;
	type LiquidityInfo = Option<NegativeImbalanceOf<T>>;

	/// Withdraw the predicted fee from the transaction origin.
	///
	/// Note: The `fee` already includes the `tip`.
	fn withdraw_fee(
		who: &T::AccountId,
		_call: &T::Call,
		_info: &DispatchInfoOf<T::Call>,
		fee: Self::Balance,
		tip: Self::Balance,
	) -> Result<Self::LiquidityInfo, TransactionValidityError> {
		if fee.is_zero() {
			return Ok(None)
		}

		let withdraw_reason = if tip.is_zero() {
			WithdrawReasons::TRANSACTION_PAYMENT
		} else {
			WithdrawReasons::TRANSACTION_PAYMENT | WithdrawReasons::TIP
		};

		let program_ids = AccountPrograms::<T>::get(who);
		if program_ids.len() == 0 {
			match T::Currency::withdraw(who, fee, withdraw_reason, ExistenceRequirement::AllowDeath)
			{
				Ok(imbalance) => Ok(Some(imbalance)),
				Err(_msg) => Err(InvalidTransaction::Payment.into()),
			}
		} else {
			ProxyPayTimes::<T>::mutate(program_ids[0], who, |time| {
				if *time == 1 {
					AccountPrograms::<T>::mutate(who, |ids| {
						if ids.contains(&program_ids[0]) {
							ids.retain(|p| *p != program_ids[0]);
						}
					});
				}
				*time = *time - 1;
			});
			let program = ProgramInfo::<T>::get(program_ids[0]).unwrap();

			// program.proxy_pay_account
			match T::Currency::withdraw(
				&program.proxy_pay_account,
				fee,
				withdraw_reason,
				ExistenceRequirement::AllowDeath,
			) {
				Ok(imbalance) => Ok(Some(imbalance)),
				Err(_msg) => Err(InvalidTransaction::Payment.into()),
			}
		}
	}

	/// Hand the fee and the tip over to the `[OnUnbalanced]` implementation.
	/// Since the predicted fee might have been too high, parts of the fee may
	/// be refunded.
	///
	/// Note: The `fee` already includes the `tip`.
	fn correct_and_deposit_fee(
		who: &T::AccountId,
		_dispatch_info: &DispatchInfoOf<T::Call>,
		_post_info: &PostDispatchInfoOf<T::Call>,
		corrected_fee: Self::Balance,
		tip: Self::Balance,
		already_withdrawn: Self::LiquidityInfo,
	) -> Result<(), TransactionValidityError> {
		if let Some(paid) = already_withdrawn {
			// Calculate how much refund we should return
			let refund_amount = paid.peek().saturating_sub(corrected_fee);

			// refund to the the account that paid the fees. If this fails, the
			// account might have dropped below the existential balance. In
			// that case we don't refund anything.
			let refund_imbalance = T::Currency::deposit_into_existing(who, refund_amount)
				.unwrap_or_else(|_| PositiveImbalanceOf::<T>::zero());
			// merge the imbalance caused by paying the fees and refunding parts of it again.
			let adjusted_paid = paid
				.offset(refund_imbalance)
				.same()
				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;
			// Call someone else to handle the imbalance (fee and tip separately)
			let imbalances = adjusted_paid.split(tip);
			T::OnUnbalanced::on_unbalanceds(
				Some(imbalances.0).into_iter().chain(Some(imbalances.1)),
			);
		}
		Ok(())
	}
}
