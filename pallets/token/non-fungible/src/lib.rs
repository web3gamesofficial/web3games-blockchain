#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure, debug,
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
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_approved)]
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
	#[pallet::getter(fn is_approved_for_all)]
	pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		// (owner, operator)
		(T::AccountId, T::AccountId),
		bool,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn total_supply)]
	pub(super) type TotalSupply<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn token_by_index)]
	pub(super) type AllTokens<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		TokenIndex,
		TokenId,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type AllTokensIndex<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		TokenId,
		TokenIndex,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn token_of_owner_by_index)]
	pub(super) type OwnedTokens<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		(T::AccountId, TokenIndex),
		TokenId,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type OwnedTokensIndex<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		TokenId,
		TokenIndex,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenCreated(T::AccountId, T::AccountId),
		Transfer(T::AccountId, T::AccountId, T::AccountId, TokenId),
		Approval(T::AccountId, T::AccountId, T::AccountId, TokenId),
		ApprovalForAll(T::AccountId, T::AccountId, T::AccountId, bool),
	}

	#[pallet::error]
	pub enum Error<T> {
		Overflow,
		Underflow,
		TokenAlreadyMinted,
		InvalidTokenAccount,
		NoPermission,
		NotTokenOwner,
		TokenNonExistent,
		ApproveToCurrentOwner,
		NotOwnerOrApproved,
		ApproveToCaller,
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
		pub fn approve(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			to: T::AccountId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_approve(&who, &token_account, &to, token_id)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn set_approve_for_all(
			origin: OriginFor<T>,
			token_account: T::AccountId,
			operator: T::AccountId,
			approved: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_set_approval_for_all(&who, &token_account, &operator, approved)?;

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
			let new_count = count.checked_add(One::one()).ok_or(Error::<T>::Overflow)?;
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

	pub fn do_approve(
		who: &T::AccountId,
		token_account: &T::AccountId,
		to: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		let owner = Self::owner_of(token_account, token_id);
		ensure!(owner != T::AccountId::default(), Error::<T>::TokenNonExistent);

		ensure!(*to != owner, Error::<T>::ApproveToCurrentOwner);
		ensure!(*who == owner || Self::is_approved_for_all(token_account, (&owner, who)), Error::<T>::NotOwnerOrApproved);

		TokenApprovals::<T>::insert(token_account, token_id, to);

		Self::deposit_event(Event::Approval(token_account.clone(), owner.clone(), to.clone(), token_id));
		
		Ok(())
	}

	pub fn do_set_approval_for_all(
		who: &T::AccountId,
		token_account: &T::AccountId,
		operator: &T::AccountId,
		approved: bool,
	) -> DispatchResult {
		ensure!(operator != who, Error::<T>::ApproveToCaller);

		OperatorApprovals::<T>::insert(token_account, (who, operator), approved);

		Self::deposit_event(Event::ApprovalForAll(token_account.clone(), who.clone(), operator.clone(), approved));

		Ok(())
	}

	pub fn do_transfer_from(
		who: &T::AccountId,
		token_account: &T::AccountId,
		from: &T::AccountId,
		to: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		ensure!(Self::is_approved_or_owner(token_account, who, token_id), Error::<T>::NotOwnerOrApproved);
		
		let owner = Self::owner_of(token_account, token_id);
		ensure!(owner != T::AccountId::default(), Error::<T>::TokenNonExistent);
		ensure!(owner == *from, Error::<T>::NotTokenOwner);

		let balance_from = Self::balance_of(token_account, from);
		let balance_to = Self::balance_of(token_account, to);

		let new_balance_from = match balance_from.checked_sub(1) {
            Some (c) => c,
            None => return Err(Error::<T>::Underflow.into()),
        };

        let new_balance_to = match balance_to.checked_add(1) {
            Some(c) => c,
            None => return Err(Error::<T>::Overflow.into()),
        };

		Self::remove_token_from_owner_enumeration(token_account, from, token_id)?;
		Self::add_token_to_owner_enumeration(token_account, to, token_id)?;

		Self::clear_approval(token_account, token_id)?;

		Balances::<T>::insert(token_account, from, new_balance_from);
		Balances::<T>::insert(token_account, to, new_balance_to);
		Owners::<T>::insert(token_account, token_id, to);

		Self::deposit_event(Event::Transfer(token_account.clone(), from.clone(), to.clone(), token_id));

		Ok(())
	}

	pub fn do_mint(
		who: &T::AccountId,
		token_account: &T::AccountId,
		to: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		Self::maybe_check_permission(who, token_account)?;
		ensure!(!Self::exists(token_account, token_id), Error::<T>::TokenAlreadyMinted);

		let balance = Self::balance_of(token_account, to);

		let new_balance = match balance.checked_add(One::one()) {
            Some(c) => c,
            None => return Err(Error::<T>::Overflow.into()),
        };

		Self::add_token_to_all_tokens_enumeration(token_account, token_id)?;
		Self::add_token_to_owner_enumeration(token_account, to, token_id)?;

		Balances::<T>::insert(token_account, to, new_balance);
		Owners::<T>::insert(token_account, token_id, to);

		Self::deposit_event(Event::Transfer(token_account.clone(), T::AccountId::default(), to.clone(), token_id));

		Ok(())
	}

	pub fn do_burn(
		who: &T::AccountId,
		token_account: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		let owner = Self::owner_of(token_account, token_id);
		ensure!(owner != T::AccountId::default(), Error::<T>::TokenNonExistent);
		ensure!(*who == owner, Error::<T>::NotTokenOwner);

		let balance = Self::balance_of(token_account, &owner);

		let new_balance = match balance.checked_sub(One::one()) {
            Some(c) => c,
            None => return Err(Error::<T>::Underflow.into()),
        };

		Self::remove_token_from_all_tokens_enumeration(token_account, token_id)?;
		Self::remove_token_from_owner_enumeration(token_account, &owner, token_id)?;
		
		Self::clear_approval(token_account, token_id)?;

		Balances::<T>::insert(token_account, &owner, new_balance);
		Owners::<T>::remove(token_account, token_id);

		Self::deposit_event(Event::Transfer(token_account.clone(), owner.clone(), T::AccountId::default(), token_id));

		Ok(())
	}

	fn is_approved_or_owner(
		token_account: &T::AccountId,
		spender: &T::AccountId,
		token_id: TokenId,
	) -> bool {
		let owner = Self::owner_of(token_account, token_id);

		*spender == owner ||
		Self::get_approved(token_account, token_id) == *spender ||
		Self::is_approved_for_all(token_account, (&owner, spender))
	}

	fn maybe_check_permission(who: &T::AccountId, token_account: &T::AccountId) -> DispatchResult {
		let token = Tokens::<T>::get(token_account).ok_or(Error::<T>::InvalidTokenAccount)?;
		ensure!(*who == token.owner, Error::<T>::NoPermission);

		Ok(())
	}

	fn clear_approval(token_account: &T::AccountId, token_id: TokenId) -> DispatchResult {
		TokenApprovals::<T>::remove(token_account, token_id);
		Ok(())
	}

    fn add_token_to_owner_enumeration(
		token_account: &T::AccountId,
		to: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
        let new_token_index = Self::balance_of(token_account, to);

        OwnedTokensIndex::<T>::insert(token_account, token_id, new_token_index);
        OwnedTokens::<T>::insert(token_account, (to, new_token_index), token_id);

        Ok(())
    }

    fn add_token_to_all_tokens_enumeration(
		token_account: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
		TotalSupply::<T>::try_mutate(token_account, |total_supply| -> DispatchResult {
			let new_token_index = *total_supply;
			*total_supply = total_supply.checked_add(One::one()).ok_or(Error::<T>::Overflow)?;

			AllTokensIndex::<T>::insert(token_account, token_id, new_token_index);
			AllTokens::<T>::insert(token_account, new_token_index, token_id);

			Ok(())
		})?;

        Ok(())
    }

    fn remove_token_from_owner_enumeration(
		token_account: &T::AccountId,
		from: &T::AccountId,
		token_id: TokenId
	) -> DispatchResult {
        let balance_of_from = Self::balance_of(token_account, from);

        let last_token_index = match balance_of_from.checked_sub(One::one()) {
            Some (c) => c,
            None => return Err(Error::<T>::Overflow.into()),
        };

        let token_index = OwnedTokensIndex::<T>::get(token_account, token_id);

        if token_index != last_token_index {
            let last_token_id = OwnedTokens::<T>::get(token_account, (from, last_token_index));
            OwnedTokens::<T>::insert(token_account, (from, token_index), last_token_id);
            OwnedTokensIndex::<T>::insert(token_account, last_token_id, token_index);
        }

        OwnedTokensIndex::<T>::remove(token_account, token_id);
		OwnedTokens::<T>::remove(token_account, (from, last_token_index));

        Ok(())
    }

    fn remove_token_from_all_tokens_enumeration(
		token_account: &T::AccountId,
		token_id: TokenId,
	) -> DispatchResult {
        let total_supply = Self::total_supply(token_account);

        let new_total_supply = match total_supply.checked_sub(One::one()) {
            Some(c) => c,
            None => return Err(Error::<T>::Overflow.into()),
        };

        let last_token_index = new_total_supply;

        let token_index = AllTokensIndex::<T>::get(token_account, token_id);

        let last_token_id = AllTokens::<T>::get(token_account, last_token_index);

        AllTokens::<T>::insert(token_account, token_index, last_token_id);
        AllTokensIndex::<T>::insert(token_account, last_token_id, token_index);

        AllTokens::<T>::remove(token_account, last_token_index);
        AllTokensIndex::<T>::remove(token_account, token_id);

        TotalSupply::<T>::insert(token_account, new_total_supply);

        Ok(())
    }
}
