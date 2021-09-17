#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{Currency, Get, ReservableCurrency},
	PalletId,
};
use primitives::{Balance, TokenId, TokenIndex};
use sp_runtime::{
	traits::{AccountIdConversion, One},
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
	base_uri: Vec<u8>,
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
	#[pallet::getter(fn owner_of)]
	pub(super) type Owners<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		TokenId,
		T::AccountId,
		ValueQuery,
	>;

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
	#[pallet::getter(fn token_approvals)]
	pub(super) type TokenApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		TokenId,
		T::AccountId,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn operator_approvals)]
	pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		ApprovalKey<T::AccountId>,
		bool,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenCreated(T::AccountId, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NumOverflow,
		AlreadyMinted,
		InvalidTokenAccount,
		NoPermission,
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
			base_uri: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_token(&who, name, symbol, base_uri)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			from: T::AccountId,
			to: T::AccountId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_transfer_from(&who, &token_account, &from, &to, token_id)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			to: T::AccountId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_mint(&who, &token_account, &to, token_id)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_burn(&who, &token_account, token_id)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn exists(token_account: &T::AccountId, token_id: TokenId) -> bool {
		Owners::<T>::contains_key(token_account, token_id)
	}

	pub fn do_create_token(
		who: &T::AccountId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		base_uri: Vec<u8>,
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
			base_uri,
		};

		Tokens::<T>::insert(&token_account, token);

		Self::deposit_event(Event::TokenCreated(token_account.clone(), who.clone()));

		Ok(token_account)
	}

	pub fn do_transfer_from(
		_who: &T::AccountId,
		_token_account: &T::AccountId,
		_from: &T::AccountId,
		_to: &T::AccountId,
		_token_id: TokenId,
	) -> DispatchResult {
		Ok(())
	}

	pub fn do_mint(
		who: &T::AccountId,
		token_account: &T::AccountId,
		to: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		Self::maybe_check_owner(who, token_account)?;

		ensure!(
			!Self::exists(token_account, token_id),
			Error::<T>::AlreadyMinted
		);

		Balances::<T>::mutate(token_account, to, |balance| *balance += 1);
		Owners::<T>::mutate(token_account, token_id, |owner| *owner = to.clone());

		Ok(())
	}

	pub fn do_burn(
		who: &T::AccountId,
		token_account: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		Balances::<T>::mutate(token_account, who, |balance| *balance -= 1);
		Owners::<T>::remove(token_account, token_id);

		Ok(())
	}

	fn maybe_check_owner(who: &T::AccountId, token_account: &T::AccountId) -> DispatchResult {
		let token = Tokens::<T>::get(token_account).ok_or(Error::<T>::InvalidTokenAccount)?;
		ensure!(*who == token.owner, Error::<T>::NoPermission);

		Ok(())
	}

	fn maybe_check_token_owner(
		who: &T::AccountId,
		token_account: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		let owner = Owners::<T>::get(token_account, token_id);
		ensure!(*who == owner, Error::<T>::NoPermission);

		Ok(())
	}
}
