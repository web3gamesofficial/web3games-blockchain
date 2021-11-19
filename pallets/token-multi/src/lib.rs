#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{Currency, Get, ReservableCurrency},
	BoundedVec, PalletId,
};
use primitives::{Balance, TokenId};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedAdd, One},
	RuntimeDebug,
};
use sp_std::{convert::TryInto, prelude::*};

pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Token<AccountId, BoundedString> {
	owner: AccountId,
	uri: BoundedString,
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

		type MultiTokenId: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaxEncodedLen;

		/// The maximum length of base uri stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// The minimum balance to create token
		#[pallet::constant]
		type CreateTokenDeposit: Get<BalanceOf<Self>>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Tokens<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::MultiTokenId,
		Token<T::AccountId, BoundedVec<u8, T::StringLimit>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_token_id)]
	pub(super) type NextTokenId<T: Config> = StorageValue<_, T::MultiTokenId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::MultiTokenId,
		Blake2_128Concat,
		(TokenId, T::AccountId),
		Balance,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn is_approved_for_all)]
	pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::MultiTokenId,
		Blake2_128Concat,
		// (owner, operator)
		(T::AccountId, T::AccountId),
		bool,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenCreated(T::MultiTokenId, T::AccountId),
		Mint(T::MultiTokenId, T::AccountId, TokenId, Balance),
		BatchMint(T::MultiTokenId, T::AccountId, Vec<TokenId>, Vec<Balance>),
		Burn(T::MultiTokenId, T::AccountId, TokenId, Balance),
		BatchBurn(T::MultiTokenId, T::AccountId, Vec<TokenId>, Vec<Balance>),
		Transferred(T::MultiTokenId, T::AccountId, T::AccountId, TokenId, Balance),
		BatchTransferred(T::MultiTokenId, T::AccountId, T::AccountId, Vec<TokenId>, Vec<Balance>),
		ApprovalForAll(T::MultiTokenId, T::AccountId, T::AccountId, bool),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoAvailableTokenId,
		NumOverflow,
		LengthMismatch,
		NoPermission,
		NotOwner,
		InvalidId,
		TokenNonExistent,
		BadMetadata,
		NotOwnerOrApproved,
		ConfuseBehavior,
		InsufficientTokens,
		InsufficientAuthorizedTokens,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_token(origin: OriginFor<T>, uri: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_token(&who, uri)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			to: T::AccountId,
			token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::has_permission(id, &who), Error::<T>::NoPermission);

			Self::do_mint(id, &to, token_id, amount)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn batch_mint(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			to: T::AccountId,
			token_ids: Vec<TokenId>,
			amounts: Vec<Balance>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::has_permission(id, &who), Error::<T>::NoPermission);

			Self::do_batch_mint(id, &to, token_ids, amounts)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn set_approval_for_all(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			operator: T::AccountId,
			approved: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Tokens::<T>::contains_key(id), Error::<T>::InvalidId,);

			OperatorApprovals::<T>::insert(id, (&who, &operator), approved);

			Self::deposit_event(Event::ApprovalForAll(id, who.clone(), operator.clone(), approved));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			to: T::AccountId,
			token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Tokens::<T>::contains_key(id), Error::<T>::InvalidId,);

			ensure!(who != to, Error::<T>::ConfuseBehavior);

			ensure!(
				Balances::<T>::get(id, (token_id, who.clone())) == amount,
				Error::<T>::InsufficientTokens
			);

			Self::do_transfer_from(id, &who, &to, token_id, amount)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn batch_transfer(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			to: T::AccountId,
			token_ids: Vec<TokenId>,
			amounts: Vec<Balance>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Tokens::<T>::contains_key(id), Error::<T>::InvalidId,);

			ensure!(who != to, Error::<T>::ConfuseBehavior);

			ensure!(token_ids.len() == amounts.len(), Error::<T>::LengthMismatch);

			let n = token_ids.len();
			for i in 0..n {
				let token_id = token_ids[i];
				let amount = amounts[i];

				ensure!(
					Balances::<T>::get(id, (token_id, who.clone())) == amount,
					Error::<T>::InsufficientTokens
				);
			}

			Self::do_batch_transfer_from(id, &who, &to, token_ids, amounts)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			from: T::AccountId,
			to: T::AccountId,
			token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Tokens::<T>::contains_key(id), Error::<T>::InvalidId,);

			ensure!(who != to, Error::<T>::ConfuseBehavior);

			ensure!(Self::owner_or_approved(id, &who, &from), Error::<T>::NotOwnerOrApproved);
			ensure!(
				Balances::<T>::get(id, (token_id, who.clone())) == amount,
				Error::<T>::InsufficientTokens
			);

			Self::do_transfer_from(id, &from, &to, token_id, amount)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn batch_transfer_from(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			from: T::AccountId,
			to: T::AccountId,
			token_ids: Vec<TokenId>,
			amounts: Vec<Balance>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Tokens::<T>::contains_key(id), Error::<T>::InvalidId,);

			ensure!(who != to, Error::<T>::ConfuseBehavior);

			ensure!(Self::owner_or_approved(id, &who, &from), Error::<T>::NotOwnerOrApproved);

			ensure!(token_ids.len() == amounts.len(), Error::<T>::LengthMismatch);

			let n = token_ids.len();
			for i in 0..n {
				let token_id = token_ids[i];
				let amount = amounts[i];

				ensure!(
					Balances::<T>::get(id, (token_id, who.clone())) == amount,
					Error::<T>::InsufficientTokens
				);
			}

			Self::do_batch_transfer_from(id, &from, &to, token_ids, amounts)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			token_id: TokenId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_burn(id, &who, token_id, amount)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn batch_burn(
			origin: OriginFor<T>,
			id: T::MultiTokenId,
			token_ids: Vec<TokenId>,
			amounts: Vec<Balance>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_batch_burn(id, &who, token_ids, amounts)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn exists(id: T::MultiTokenId) -> bool {
		Tokens::<T>::contains_key(id)
	}

	pub fn do_create_token(
		who: &T::AccountId,
		uri: Vec<u8>,
	) -> Result<T::MultiTokenId, DispatchError> {
		// let deposit = T::CreateTokenDeposit::get();
		// T::Currency::reserve(&who, deposit.clone())?;

		let bounded_uri: BoundedVec<u8, T::StringLimit> =
			uri.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		let id = NextTokenId::<T>::try_mutate(|id| -> Result<T::MultiTokenId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableTokenId)?;
			Ok(current_id)
		})?;

		let token = Token { owner: who.clone(), uri: bounded_uri };

		Tokens::<T>::insert(id, token);

		Self::deposit_event(Event::TokenCreated(id, who.clone()));

		Ok(id)
	}

	pub fn do_mint(
		id: T::MultiTokenId,
		to: &T::AccountId,
		token_id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		Self::add_balance_to(id, to, token_id, amount)?;

		Self::deposit_event(Event::Mint(id, to.clone(), token_id, amount));

		Ok(())
	}

	pub fn do_batch_mint(
		id: T::MultiTokenId,
		to: &T::AccountId,
		token_ids: Vec<TokenId>,
		amounts: Vec<Balance>,
	) -> DispatchResult {
		ensure!(token_ids.len() == amounts.len(), Error::<T>::LengthMismatch);

		let n = token_ids.len();
		for i in 0..n {
			let token_id = token_ids[i];
			let amount = amounts[i];

			Self::add_balance_to(id, to, token_id, amount)?;
		}

		Self::deposit_event(Event::BatchMint(id, to.clone(), token_ids, amounts));

		Ok(())
	}

	pub fn do_burn(
		id: T::MultiTokenId,
		account: &T::AccountId,
		token_id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		Self::remove_balance_from(id, account, token_id, amount)?;

		Self::deposit_event(Event::Burn(id, account.clone(), token_id, amount));

		Ok(())
	}

	pub fn do_batch_burn(
		id: T::MultiTokenId,
		account: &T::AccountId,
		token_ids: Vec<TokenId>,
		amounts: Vec<Balance>,
	) -> DispatchResult {
		ensure!(token_ids.len() == amounts.len(), Error::<T>::LengthMismatch);

		let n = token_ids.len();
		for i in 0..n {
			let token_id = token_ids[i];
			let amount = amounts[i];

			Self::remove_balance_from(id, account, token_id, amount)?;
		}

		Self::deposit_event(Event::BatchBurn(id, account.clone(), token_ids, amounts));

		Ok(())
	}

	pub fn do_transfer_from(
		id: T::MultiTokenId,
		from: &T::AccountId,
		to: &T::AccountId,
		token_id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		Self::remove_balance_from(id, from, token_id, amount)?;

		Self::add_balance_to(id, to, token_id, amount)?;

		Self::deposit_event(Event::Transferred(id, from.clone(), to.clone(), token_id, amount));

		Ok(())
	}

	pub fn do_batch_transfer_from(
		id: T::MultiTokenId,
		from: &T::AccountId,
		to: &T::AccountId,
		token_ids: Vec<TokenId>,
		amounts: Vec<Balance>,
	) -> DispatchResult {
		let n = token_ids.len();
		for i in 0..n {
			let token_id = token_ids[i];
			let amount = amounts[i];

			Self::remove_balance_from(id, from, token_id, amount)?;

			Self::add_balance_to(id, to, token_id, amount)?;
		}

		Self::deposit_event(Event::BatchTransferred(
			id,
			from.clone(),
			to.clone(),
			token_ids,
			amounts,
		));

		Ok(())
	}

	pub fn balance_of_batch(
		id: T::MultiTokenId,
		accounts: &Vec<T::AccountId>,
		token_ids: Vec<TokenId>,
	) -> Result<Vec<Balance>, DispatchError> {
		ensure!(accounts.len() == token_ids.len(), Error::<T>::LengthMismatch);

		let mut batch_balances = vec![Balance::from(0u128); accounts.len()];

		let n = accounts.len();
		for i in 0..n {
			let account = &accounts[i];
			let token_id = token_ids[i];

			batch_balances[i] = Self::balance_of(id, (token_id, account));
		}

		Ok(batch_balances)
	}

	fn add_balance_to(
		id: T::MultiTokenId,
		to: &T::AccountId,
		token_id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		Balances::<T>::try_mutate(id, (token_id, to), |balance| -> DispatchResult {
			*balance = balance.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Ok(())
	}

	fn remove_balance_from(
		id: T::MultiTokenId,
		from: &T::AccountId,
		token_id: TokenId,
		amount: Balance,
	) -> DispatchResult {
		Balances::<T>::try_mutate(id, (token_id, from), |balance| -> DispatchResult {
			*balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Ok(())
	}

	fn owner_or_approved(id: T::MultiTokenId, who: &T::AccountId, owner: &T::AccountId) -> bool {
		*who == *owner || Self::is_approved_for_all(id, (owner, who))
	}

	fn has_permission(id: T::MultiTokenId, who: &T::AccountId) -> bool {
		let token = Tokens::<T>::get(id).unwrap();
		*who == token.owner
	}
}
