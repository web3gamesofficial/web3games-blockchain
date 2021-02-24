#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{fmt::Debug, prelude::*};
use sp_runtime::{
	RuntimeDebug,
	traits::{
		AtLeast32Bit, MaybeSerializeDeserialize, Bounded, Member,
		One, AtLeast32BitUnsigned, CheckedAdd, CheckedSub,
	},
};
// use sp_io::hashing::blake2_128;
use codec::{Encode, Decode, HasCompact};
use frame_support::{
	ensure,
	// traits::Randomness,
	dispatch::{DispatchResult, DispatchError},
};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// type TokenBalance: Parameter + Member + AtLeast32BitUnsigned + Codec + Default +
		// Copy + MaybeSerializeDeserialize + From<u32> + From<u128> + Into<u128>;
		// type TokenId: Parameter + Member + Default + Codec + MaybeSerializeDeserialize +
		// Copy + From<[u8;32]> + Into<[u8;32]>;
		type TokenBalance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

		type TokenId: Member + Parameter + Default + Copy + HasCompact;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Tokens<T: Config> = StorageMap<
		_,
		Blake2_128,
		T::TokenId,
		Token<T::AccountId>
	>;

	#[pallet::storage]
	#[pallet::getter(fn balances)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::TokenId,
		Blake2_128Concat,
		T::AccountId,
		T::TokenBalance,
		ValueQuery
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Created(T::AccountId, T::TokenId),
		Mint(T::AccountId, T::TokenId, T::TokenBalance),
		BatchMint(T::AccountId, Vec<T::TokenId>, Vec<T::TokenBalance>),
		Burn(T::AccountId, T::TokenId, T::TokenBalance),
		BatchBurn(T::AccountId, Vec<T::TokenId>, Vec<T::TokenBalance>),
		Transferred(T::AccountId, T::AccountId, T::TokenId, T::TokenBalance),
		BatchTransferred(T::AccountId, T::AccountId, Vec<T::TokenId>, Vec<T::TokenBalance>),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unknown,
		InUse,
		InvalidTokenId,
		InsufficientBalance,
		NumOverflow,
		InvalidArrayLength,
		Overflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
		pub fn create(origin: OriginFor<T>, id: T::TokenId, is_nf: bool, uri: Vec<u8>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(!Tokens::<T>::contains_key(id), Error::<T>::InUse);

			Tokens::<T>::insert(id, Token {
				creator: who.clone(),
				is_nf,
				uri,
			});

			Self::deposit_event(Event::Created(who, id));
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn mint(origin: OriginFor<T>, to: T::AccountId, id: T::TokenId, amount: T::TokenBalance) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			Self::do_mint(&to, &id, amount)?;
			
			Ok(().into())
		}
	}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Token<
	AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
	creator: AccountId,
	is_nf: bool,
	uri: Vec<u8>,
}

impl<T: Config> Pallet<T> {
	pub fn do_mint(
		to: &T::AccountId,
		id: &T::TokenId,
		amount: T::TokenBalance
	) -> DispatchResult {
		Balances::<T>::try_mutate(id, to, |balance| -> DispatchResult {
			*balance = balance
				.checked_add(&amount)
				.ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Self::deposit_event(Event::Mint(to.clone(), id.clone(), amount));

		Ok(())
	}

	pub fn batch_mint(
		to: &T::AccountId,
		ids: &Vec<T::TokenId>,
		amounts: Vec<T::TokenBalance>
	) -> DispatchResult {
		ensure!(ids.len() == amounts.len(), Error::<T>::InvalidArrayLength);

		let n = ids.len();
		for i in 0..n {
			let id = ids[i];
			let amount = amounts[i];

			Balances::<T>::try_mutate(id, to, |balance| -> DispatchResult {
				*balance = balance
					.checked_add(&amount)
					.ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;
		}

		Self::deposit_event(Event::BatchMint(to.clone(), ids.clone(), amounts));

		Ok(())
	}

	pub fn burn(
		from: &T::AccountId,
		id: &T::TokenId,
		amount: T::TokenBalance
	) -> DispatchResult {
		Balances::<T>::try_mutate(id, from, |balance| -> DispatchResult {
			*balance = balance
				.checked_sub(&amount)
				.ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Self::deposit_event(Event::Burn(from.clone(), id.clone(), amount));

		Ok(())
	}

	pub fn batch_burn(
		from: &T::AccountId,
		ids: &Vec<T::TokenId>,
		amounts: Vec<T::TokenBalance>
	) -> DispatchResult {
		ensure!(ids.len() == amounts.len(), Error::<T>::InvalidArrayLength);

		let n = ids.len();
		for i in 0..n {
			let id = ids[i];
			let amount = amounts[i];

			Balances::<T>::try_mutate(id, from, |balance| -> DispatchResult {
				*balance = balance
					.checked_sub(&amount)
					.ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;
		}

		Self::deposit_event(Event::BatchBurn(from.clone(), ids.clone(), amounts));

		Ok(())
	}

	pub fn transfer_from(
		from: &T::AccountId,
		to: &T::AccountId,
		id: &T::TokenId,
		amount: T::TokenBalance
	) -> DispatchResult {
		if from == to {
			return Ok(());
		}

		Balances::<T>::try_mutate(id, from, |balance| -> DispatchResult {
			*balance = balance
				.checked_sub(&amount)
				.ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Balances::<T>::try_mutate(id, to, |balance| -> DispatchResult {
			*balance = balance
				.checked_add(&amount)
				.ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Self::deposit_event(Event::Transferred(from.clone(), to.clone(), id.clone(), amount));

		Ok(())
	}

	pub fn batch_transfer_from(
		from: &T::AccountId,
		to: &T::AccountId,
		ids: &Vec<T::TokenId>,
		amounts: Vec<T::TokenBalance>
	) -> DispatchResult {
		if from == to {
			return Ok(());
		}

		ensure!(ids.len() == amounts.len(), Error::<T>::InvalidArrayLength);

		let n = ids.len();
		for i in 0..n {
			let id = &ids[i];
			let amount = amounts[i];

			Balances::<T>::try_mutate(id, from, |balance| -> DispatchResult {
				*balance = balance
					.checked_sub(&amount)
					.ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;

			Balances::<T>::try_mutate(id, to, |balance| -> DispatchResult {
				*balance = balance
					.checked_add(&amount)
					.ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;
		}

		Self::deposit_event(Event::BatchTransferred(from.clone(), to.clone(), ids.to_vec(), amounts));

		Ok(())
	}

	pub fn balance_of(owner: &T::AccountId, id: &T::TokenId) -> T::TokenBalance {
		Self::balances(id, owner)
	}

	pub fn balance_of_batch(owners: &Vec<T::AccountId>, ids: &Vec<T::TokenId>) -> Result<Vec<T::TokenBalance>, DispatchError> {
		ensure!(owners.len() == ids.len(), Error::<T>::InvalidArrayLength);

		let mut batch_balances = vec![T::TokenBalance::from(0u32); owners.len()];

		let n = owners.len();
		for i in 0..n {
			let owner = &owners[i];
			let id = ids[i];

			batch_balances[i] = Self::balances(id, owner);
		}

		Ok(batch_balances)
	}
}
