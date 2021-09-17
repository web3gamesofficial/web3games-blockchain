#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{Currency, Get, ReservableCurrency},
	PalletId,
};
use primitives::{Balance, TokenIndex};
use sp_runtime::{
	traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, One, Zero},
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

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct Token<AccountId> {
	owner: AccountId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
	total_supply: Balance,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct ApprovalKey<AccountId> {
	owner: AccountId,
	operator: AccountId,
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

		/// The minimum balance to create token
		#[pallet::constant]
		type CreateTokenDeposit: Get<BalanceOf<Self>>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Tokens<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Token<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn token_count)]
	pub(super) type TokenCount<T> = StorageValue<_, TokenIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		Balance,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn allowances)]
	pub(super) type Allowances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		ApprovalKey<T::AccountId>,
		Balance,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenCreated(T::AccountId, T::AccountId),
		TokenTransferred(T::AccountId, T::AccountId, T::AccountId, Balance),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unknown,
		NumOverflow,
		NoPermission,
		NotOwner,
		InvalidTokenAccount,
		AmountExceedAllowance,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_token(
			origin: OriginFor<T>,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_token(&who, name, symbol, decimals)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			recipient: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_transfer(&who, &token_account, &recipient, amount)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn approve(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			spender: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_approve(&who, &token_account, &spender, amount)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			sender: T::AccountId,
			recipient: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_transfer_from(&who, &token_account, &sender, &recipient, amount)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			account: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_mint(&who, &token_account, &account, amount)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			account: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_burn(&who, &token_account, &account, amount)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn exists(token_account: &T::AccountId) -> bool {
		Tokens::<T>::contains_key(token_account)
	}

	pub fn total_supply(token_account: &T::AccountId) -> Result<Balance, DispatchError> {
		let token = Tokens::<T>::get(token_account).ok_or(Error::<T>::InvalidTokenAccount)?;
		Ok(token.total_supply)
	}

	pub fn do_create_token(
		who: &T::AccountId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> Result<T::AccountId, DispatchError> {
		let deposit = T::CreateTokenDeposit::get();
		T::Currency::reserve(&who, deposit.clone())?;

		let token_id = TokenCount::<T>::try_mutate(|count| -> Result<TokenIndex, DispatchError> {
			let new_count = count.checked_add(One::one()).ok_or(Error::<T>::NumOverflow)?;
			*count = new_count;
			Ok(new_count)
		})?;

		let token_account: T::AccountId = <T as Config>::PalletId::get().into_sub_account(token_id);

		let token = Token {
			owner: who.clone(),
			name,
			symbol,
			decimals,
			total_supply: Balance::default(),
		};

		Tokens::<T>::insert(&token_account, token);

		Self::deposit_event(Event::TokenCreated(token_account.clone(), who.clone()));

		Ok(token_account)
	}

	pub fn do_transfer(
		who: &T::AccountId,
		token_account: &T::AccountId,
		recipient: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {

		Self::decrease_balance(token_account, who, amount)?;
		Self::increase_balance(token_account, recipient, amount)?;

		Self::deposit_event(Event::TokenTransferred(token_account.clone(), who.clone(), recipient.clone(), amount));

		Ok(())
	}

	pub fn do_transfer_from(
		who: &T::AccountId,
		token_account: &T::AccountId,
		sender: &T::AccountId,
		recipient: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		let key = ApprovalKey {
			owner: sender.clone(),
			operator: who.clone(),
		};

		Allowances::<T>::try_mutate(token_account, &key, |allowance| -> DispatchResult {
			*allowance = allowance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Self::do_transfer(sender, token_account, recipient, amount)?;
		Ok(())
	}

	pub fn do_mint(
		who: &T::AccountId,
		token_account: &T::AccountId,
		account: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		Tokens::<T>::try_mutate(token_account, |maybe_token| -> DispatchResult {
			let token = maybe_token.as_mut().ok_or(Error::<T>::Unknown)?;
			ensure!(who == &token.owner, Error::<T>::NoPermission);

			let new_total_supply = token.total_supply.saturating_add(amount);
			token.total_supply = new_total_supply;
			Ok(())
		})?;
		Self::increase_balance(token_account, account, amount)?;

		Ok(())
	}

	pub fn do_burn(
		who: &T::AccountId,
		token_account: &T::AccountId,
		account: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		ensure!(who == account, Error::<T>::NotOwner);

		Self::decrease_balance(token_account, account, amount)?;

		Tokens::<T>::try_mutate(token_account, |maybe_token| -> DispatchResult {
			let token = maybe_token.as_mut().ok_or(Error::<T>::Unknown)?;
			let new_total_supply = token.total_supply.saturating_sub(amount);
			token.total_supply = new_total_supply;
			Ok(())
		})?;

		Ok(())
	}

	pub fn do_approve(
		who: &T::AccountId,
		token_account: &T::AccountId,
		spender: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		let key = ApprovalKey {
			owner: who.clone(),
			operator: spender.clone(),
		};

		Allowances::<T>::try_mutate(token_account, &key, |allowance| -> DispatchResult {
			*allowance = amount;
			Ok(())
		})?;
		Ok(())
	}

	fn increase_balance(
		token_account: &T::AccountId,
		to: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		Balances::<T>::try_mutate(token_account, to, |balance| -> DispatchResult {
			*balance = balance.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Ok(())
	}

	fn decrease_balance(
		token_account: &T::AccountId,
		from: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		Balances::<T>::try_mutate(token_account, from, |balance| -> DispatchResult {
			*balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Ok(())
	}
}
