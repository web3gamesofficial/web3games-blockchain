#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{Currency, Get, ReservableCurrency},
	PalletId,
};
use primitives::{Balance, TokenId, TokenIndex};
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

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Token<AccountId> {
	owner: AccountId,
	uri: Vec<u8>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct ApprovalKey<AccountId> {
	owner: AccountId,
	operator: AccountId,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
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
	#[pallet::getter(fn balances)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		(TokenId, T::AccountId),
		Balance,
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
		Mint(T::AccountId, T::AccountId, TokenId, Balance),
		BatchMint(T::AccountId, T::AccountId, Vec<TokenId>, Vec<Balance>),
		Burn(T::AccountId, T::AccountId, TokenId, Balance),
		BatchBurn(T::AccountId, T::AccountId, Vec<TokenId>, Vec<Balance>),
		Transferred(T::AccountId, T::AccountId, T::AccountId, TokenId, Balance),
		BatchTransferred(
			T::AccountId,
			T::AccountId,
			T::AccountId,
			Vec<TokenId>,
			Vec<Balance>,
		),
		ApprovalForAll(T::AccountId, T::AccountId, T::AccountId, bool),
	}

	#[pallet::error]
	pub enum Error<T> {
		NumOverflow,
		LengthMismatch,
		NoPermission,
		NotOwner,
		TokenNotFound,
		InvalidTokenAccount,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_token(origin: OriginFor<T>, uri: Vec<u8>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_create_token(&who, uri)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn set_approval_for_all(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			operator: T::AccountId,
			approved: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_set_approval_for_all(&who, &token_account, &operator, approved)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			from: T::AccountId,
			to: T::AccountId,
			id: TokenId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_transfer_from(&who, &token_account, &from, &to, id, amount)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn batch_transfer_from(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			from: T::AccountId,
			to: T::AccountId,
			ids: Vec<TokenId>,
			amounts: Vec<Balance>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_batch_transfer_from(&who, &token_account, &from, &to, ids, amounts)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			to: T::AccountId,
			id: TokenId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_mint(&who, &token_account, &to, id, amount)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn batch_mint(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			to: T::AccountId,
			ids: Vec<TokenId>,
			amounts: Vec<Balance>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_batch_mint(&who, &token_account, &to, ids, amounts)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			account: T::AccountId,
			id: TokenId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_burn(&who, &token_account, &account, id, amount)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn batch_burn(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			account: T::AccountId,
			ids: Vec<TokenId>,
			amounts: Vec<Balance>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_batch_burn(&who, &token_account, &account, ids, amounts)?;

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn exists(token_account: &T::AccountId) -> bool {
		Tokens::<T>::contains_key(token_account)
	}

	pub fn do_create_token(
		who: &T::AccountId,
		uri: Vec<u8>,
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
			uri,
		};

		Tokens::<T>::insert(&token_account, token);

		Self::deposit_event(Event::TokenCreated(token_account.clone(), who.clone()));

		Ok(token_account)
	}

	pub fn do_set_approval_for_all(
		who: &T::AccountId,
		token_account: &T::AccountId,
		operator: &T::AccountId,
		approved: bool,
	) -> DispatchResult {
		ensure!(
			Tokens::<T>::contains_key(token_account),
			Error::<T>::TokenNotFound
		);

		let key = ApprovalKey {
			owner: who.clone(),
			operator: operator.clone(),
		};
		OperatorApprovals::<T>::try_mutate(token_account, &key, |status| -> DispatchResult {
			*status = approved;
			Ok(())
		})?;

		Self::deposit_event(Event::ApprovalForAll(
			token_account.clone(),
			who.clone(),
			operator.clone(),
			approved,
		));

		Ok(())
	}

	pub fn do_mint(
		who: &T::AccountId,
		token_account: &T::AccountId,
		to: &T::AccountId,
		id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		Self::maybe_check_owner(who, token_account)?;

		Self::add_balance_to(token_account, to, id, amount)?;

		Self::deposit_event(Event::Mint(token_account.clone(), to.clone(), id, amount));

		Ok(())
	}

	pub fn do_batch_mint(
		who: &T::AccountId,
		token_account: &T::AccountId,
		to: &T::AccountId,
		ids: Vec<TokenId>,
		amounts: Vec<Balance>,
	) -> DispatchResult {
		Self::maybe_check_owner(who, token_account)?;
		ensure!(ids.len() == amounts.len(), Error::<T>::LengthMismatch);

		let n = ids.len();
		for i in 0..n {
			let id = ids[i];
			let amount = amounts[i];

			Self::add_balance_to(token_account, to, id, amount)?;
		}

		Self::deposit_event(Event::BatchMint(
			token_account.clone(),
			to.clone(),
			ids,
			amounts,
		));

		Ok(())
	}

	pub fn do_burn(
		who: &T::AccountId,
		token_account: &T::AccountId,
		account: &T::AccountId,
		id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		ensure!(who == account, Error::<T>::NotOwner);

		Self::remove_balance_from(token_account, account, id, amount)?;

		Self::deposit_event(Event::Burn(token_account.clone(), account.clone(), id, amount));

		Ok(())
	}

	pub fn do_batch_burn(
		who: &T::AccountId,
		token_account: &T::AccountId,
		account: &T::AccountId,
		ids: Vec<TokenId>,
		amounts: Vec<Balance>,
	) -> DispatchResult {
		ensure!(who == account, Error::<T>::NotOwner);
		ensure!(ids.len() == amounts.len(), Error::<T>::LengthMismatch);

		let n = ids.len();
		for i in 0..n {
			let id = ids[i];
			let amount = amounts[i];

			Self::remove_balance_from(token_account, account, id, amount)?;
		}

		Self::deposit_event(Event::BatchBurn(
			token_account.clone(),
			account.clone(),
			ids,
			amounts,
		));

		Ok(())
	}

	pub fn do_transfer_from(
		who: &T::AccountId,
		token_account: &T::AccountId,
		from: &T::AccountId,
		to: &T::AccountId,
		id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		ensure!(
			Self::owner_or_approved(who, token_account, from),
			Error::<T>::NoPermission
		);

		if from == to || amount == Zero::zero() {
			return Ok(());
		}

		Self::remove_balance_from(token_account, from, id, amount)?;

		Self::add_balance_to(token_account, to, id, amount)?;

		Self::deposit_event(Event::Transferred(
			token_account.clone(),
			from.clone(),
			to.clone(),
			id,
			amount,
		));

		Ok(())
	}

	pub fn do_batch_transfer_from(
		who: &T::AccountId,
		token_account: &T::AccountId,
		from: &T::AccountId,
		to: &T::AccountId,
		ids: Vec<TokenId>,
		amounts: Vec<Balance>,
	) -> DispatchResult {
		ensure!(
			Self::owner_or_approved(who, token_account, from),
			Error::<T>::NoPermission
		);

		if from == to {
			return Ok(());
		}

		ensure!(ids.len() == amounts.len(), Error::<T>::LengthMismatch);

		let n = ids.len();
		for i in 0..n {
			let id = ids[i];
			let amount = amounts[i];

			Self::remove_balance_from(token_account, from, id, amount)?;

			Self::add_balance_to(token_account, to, id, amount)?;
		}

		Self::deposit_event(Event::BatchTransferred(
			token_account.clone(),
			from.clone(),
			to.clone(),
			ids,
			amounts,
		));

		Ok(())
	}

	pub fn owner_or_approved(
		token_account: &T::AccountId,
		who: &T::AccountId,
		account: &T::AccountId,
	) -> bool {
		*who == *account || Self::is_approved_for_all(token_account, account, who)
	}

	pub fn is_approved_for_all(
		token_account: &T::AccountId,
		owner: &T::AccountId,
		operator: &T::AccountId,
	) -> bool {
		let key = ApprovalKey {
			owner: owner.clone(),
			operator: operator.clone(),
		};
		Self::operator_approvals(token_account, &key)
	}

	pub fn balance_of(
		token_account: &T::AccountId,
		account: &T::AccountId,
		id: TokenId,
	) -> Balance {
		Self::balances(token_account, (id, account))
	}

	pub fn balance_of_batch(
		token_account: &T::AccountId,
		accounts: &Vec<T::AccountId>,
		ids: Vec<TokenId>,
	) -> Result<Vec<Balance>, DispatchError> {
		ensure!(accounts.len() == ids.len(), Error::<T>::LengthMismatch);

		let mut batch_balances = vec![Balance::from(0u128); accounts.len()];

		let n = accounts.len();
		for i in 0..n {
			let account = &accounts[i];
			let id = ids[i];

			batch_balances[i] = Self::balances(token_account, (id, account));
		}

		Ok(batch_balances)
	}

	fn add_balance_to(
		token_account: &T::AccountId,
		to: &T::AccountId,
		id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		Balances::<T>::try_mutate(token_account, (id, to), |balance| -> DispatchResult {
			*balance = balance.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Ok(())
	}

	fn remove_balance_from(
		token_account: &T::AccountId,
		from: &T::AccountId,
		id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		Balances::<T>::try_mutate(token_account, (id, from), |balance| -> DispatchResult {
			*balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Ok(())
	}

	fn maybe_check_owner(who: &T::AccountId, token_account: &T::AccountId) -> DispatchResult {
		let token = Tokens::<T>::get(token_account).ok_or(Error::<T>::InvalidTokenAccount)?;
		ensure!(*who == token.owner, Error::<T>::NoPermission);

		Ok(())
	}
}
