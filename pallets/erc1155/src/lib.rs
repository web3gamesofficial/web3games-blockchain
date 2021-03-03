#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{fmt::Debug, prelude::*};
use sp_runtime::{
	RuntimeDebug,
	traits::{
		AtLeast32BitUnsigned, CheckedAdd, One,
	},
};
use codec::{Encode, Decode, HasCompact};
use frame_support::{
	ensure,
	dispatch::{DispatchResult, DispatchError},
};
use primitives::Balance;
use frame_support::debug;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type TokenId: Member + Parameter + Default + Copy + HasCompact + From<u64> + Into<u64>;

		type TaoId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + From<u64> + Into<u64>;
	}

	// pub type GenesisTaos<T> = (
	// 	<T as frame_system::Config>::AccountId,
	// 	Vec<u8>,
	// );

	// #[pallet::genesis_config]
	// pub struct GenesisConfig<T: Config> {
	// 	pub taos: Vec<GenesisTaos<T>>,
	// }

	// #[cfg(feature = "std")]
	// impl<T: Config> Default for GenesisConfig<T> {
	// 	fn default() -> Self {
	// 		GenesisConfig { tokens: vec![] }
	// 	}
	// }

	// #[pallet::genesis_build]
	// impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
	// 	fn build(&self) {
	// 		self.taos.iter().for_each(|tao| {
	// 			let _tao_id = Pallet::<T>::do_create_tao(&tao.0, tao.1.to_vec())
	// 				.expect("Create tao cannot fail while building genesis");
	// 		})
	// 	}
	// }

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Taos<T: Config> = StorageMap<
		_,
		Blake2_128,
		T::TaoId,
		Tao<T::AccountId>
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_tao_id)]
	pub(super) type NextTaoId<T: Config> = StorageValue<
		_,
		T::TaoId,
		ValueQuery
	>;

	#[pallet::storage]
	pub(super) type Tokens<T: Config> = StorageDoubleMap<
		_,
		Blake2_128,
		T::TaoId,
		Blake2_128,
		T::TokenId,
		Token<T::TaoId, T::AccountId>
	>;

	#[pallet::storage]
	#[pallet::getter(fn balances)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		(T::TaoId, T::TokenId),
		Balance,
		ValueQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn operator_approvals)]
	pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		bool,
		ValueQuery
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TaoCreated(T::TaoId, T::AccountId),
		TokenCreated(T::TaoId, T::TokenId, T::AccountId),
		Mint(T::AccountId, T::TaoId, T::TokenId, Balance),
		BatchMint(T::AccountId, T::TaoId, Vec<T::TokenId>, Vec<Balance>),
		Burn(T::AccountId, T::TaoId, T::TokenId, Balance),
		BatchBurn(T::AccountId, T::TaoId, Vec<T::TokenId>, Vec<Balance>),
		Transferred(T::AccountId, T::AccountId, T::TaoId, T::TokenId, Balance),
		BatchTransferred(T::AccountId, T::AccountId, T::TaoId, Vec<T::TokenId>, Vec<Balance>),
		ApprovalForAll(T::AccountId, T::AccountId, bool),
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
		NoAvailableTaoId,
		InvalidTaoId,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
		pub fn create_tao(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_create_tao(&who, data)?;
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn create_token(
			origin: OriginFor<T>,
			tao_id: T::TaoId,
			token_id: T::TokenId,
			is_nf: bool,
			uri: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_create_token(&who, tao_id, token_id, is_nf, uri)?;
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn set_approval_for_all(
			origin: OriginFor<T>,
			operator: T::AccountId,
			approved: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			if operator == who {
				return Ok(().into())
			}

			Self::do_set_approval_for_all(&who, &operator, approved)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			from: T::AccountId,
			to: T::AccountId,
			tao_id: T::TaoId,
			token_id: T::TokenId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			Self::do_transfer_from(&from, &to, tao_id, token_id, amount)?;
			
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn batch_transfer_from(
			origin: OriginFor<T>,
			from: T::AccountId,
			to: T::AccountId,
			tao_id: T::TaoId,
			token_ids: Vec<T::TokenId>,
			amounts: Vec<Balance>
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			Self::do_batch_transfer_from(&from, &to, tao_id, token_ids, amounts)?;
			
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
			to: T::AccountId,
			tao_id: T::TaoId,
			token_id: T::TokenId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			Self::do_mint(&to, tao_id, token_id, amount)?;
			
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn batch_mint(
			origin: OriginFor<T>,
			to: T::AccountId,
			tao_id: T::TaoId,
			token_ids: Vec<T::TokenId>,
			amounts: Vec<Balance>
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			Self::do_batch_mint(&to, tao_id, token_ids, amounts)?;
			
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			from: T::AccountId,
			tao_id: T::TaoId,
			token_id: T::TokenId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			Self::do_burn(&from, tao_id, token_id, amount)?;
			
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn batch_burn(
			origin: OriginFor<T>,
			from: T::AccountId,
			tao_id: T::TaoId,
			token_ids: Vec<T::TokenId>,
			amounts: Vec<Balance>
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			Self::do_batch_burn(&from, tao_id, token_ids, amounts)?;
			
			Ok(().into())
		}
	}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Tao<
	AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
	owner: AccountId,
	data: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Token<
	TaoId: Encode + Decode + Clone + Debug + Eq + PartialEq,
	AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
	tao_id: TaoId,
	creator: AccountId,
	is_nf: bool,
	uri: Vec<u8>,
}

impl<T: Config> Pallet<T> {

	// func_id 1002 do_create_tao(who: &T::AccountId, data: Vec<u8>) -> Result<T::TaoId, DispatchError>
	pub fn do_create_tao(who: &T::AccountId, data: Vec<u8>) -> Result<T::TaoId, DispatchError> {
		let tao_id =
			NextTaoId::<T>::try_mutate(|id| -> Result<T::TaoId, DispatchError> {
				let current_id = *id;
				*id = id
					.checked_add(&One::one())
					.ok_or(Error::<T>::NoAvailableTaoId)?;
				Ok(current_id)
			})?;

		let tao = Tao {
			owner: who.clone(),
			data,
		};

		Taos::<T>::insert(tao_id, tao);

		Self::deposit_event(Event::TaoCreated(tao_id, who.clone()));

		Ok(tao_id)
	}

	// func_id 1003
	// do_create_token(
	// 		who: &T::AccountId,
	// 		tao_id: T::TaoId,
	// 		token_id: T::TokenId,
	// 		is_nf: bool,
	// 		uri: Vec<u8>,
	// 	)
	pub fn do_create_token(
		who: &T::AccountId,
		tao_id: T::TaoId,
		token_id: T::TokenId,
		is_nf: bool,
		uri: Vec<u8>,
	) -> DispatchResult {
		ensure!(Taos::<T>::contains_key(tao_id), Error::<T>::InvalidTaoId);
		ensure!(!Tokens::<T>::contains_key(tao_id, token_id), Error::<T>::InUse);

		Tokens::<T>::insert(tao_id, token_id, Token {
			tao_id,
			creator: who.clone(),
			is_nf,
			uri,
		});

		Self::deposit_event(Event::TokenCreated(tao_id, token_id, who.clone()));
		Ok(())
	}

	// func_id 1004
	// do_set_approval_for_all(
	// 		owner: &T::AccountId,
	// 		operator: &T::AccountId,
	// 		approved: bool,
	// 	)
	pub fn do_set_approval_for_all(
		owner: &T::AccountId,
		operator: &T::AccountId,
		approved: bool,
	) -> DispatchResult {
		OperatorApprovals::<T>::try_mutate(owner, operator, |status| -> DispatchResult {
			*status = approved;
			Ok(())
		})?;

		Self::deposit_event(Event::ApprovalForAll(owner.clone(), operator.clone(), approved));

		Ok(())
	}

	// func_id 1005
	// do_mint(
	// 		to: &T::AccountId,
	// 		tao_id: T::TaoId,
	// 		token_id: T::TokenId,
	// 		amount: Balance
	// 	)
	pub fn do_mint(
		to: &T::AccountId,
		tao_id: T::TaoId,
		token_id: T::TokenId,
		amount: Balance
	) -> DispatchResult {
		Balances::<T>::try_mutate(to, (tao_id, token_id), |balance| -> DispatchResult {
			*balance = balance
				.checked_add(amount)
				.ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Self::deposit_event(Event::Mint(to.clone(), tao_id, token_id, amount));

		Ok(())
	}

	// func_id 1006
	// do_batch_mint(
	// 		to: &T::AccountId,
	// 		tao_id: T::TaoId,
	// 		token_ids: Vec<T::TokenId>,
	// 		amounts: Vec<Balance>
	// 	)
	pub fn do_batch_mint(
		to: &T::AccountId,
		tao_id: T::TaoId,
		token_ids: Vec<T::TokenId>,
		amounts: Vec<Balance>
	) -> DispatchResult {
		ensure!(token_ids.len() == amounts.len(), Error::<T>::InvalidArrayLength);

		let n = token_ids.len();
		for i in 0..n {
			let token_id = token_ids[i];
			let amount = amounts[i];

			Balances::<T>::try_mutate(to, (tao_id, token_id), |balance| -> DispatchResult {
				*balance = balance
					.checked_add(amount)
					.ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;
		}

		Self::deposit_event(Event::BatchMint(to.clone(), tao_id, token_ids, amounts));

		Ok(())
	}

	// func_id 1007
	// do_burn(
	// 		from: &T::AccountId,
	// 		tao_id: T::TaoId,
	// 		token_id: T::TokenId,
	// 		amount: Balance
	// 	)
	pub fn do_burn(
		from: &T::AccountId,
		tao_id: T::TaoId,
		token_id: T::TokenId,
		amount: Balance
	) -> DispatchResult {
		Balances::<T>::try_mutate(from, (tao_id, token_id), |balance| -> DispatchResult {
			*balance = balance
				.checked_sub(amount)
				.ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Self::deposit_event(Event::Burn(from.clone(), tao_id, token_id, amount));

		Ok(())
	}

	// func_id 1008
	// do_batch_burn(
	// 		from: &T::AccountId,
	// 		tao_id: T::TaoId,
	// 		token_ids: Vec<T::TokenId>,
	// 		amounts: Vec<Balance>
	// 	)

	pub fn do_batch_burn(
		from: &T::AccountId,
		tao_id: T::TaoId,
		token_ids: Vec<T::TokenId>,
		amounts: Vec<Balance>
	) -> DispatchResult {
		ensure!(token_ids.len() == amounts.len(), Error::<T>::InvalidArrayLength);

		let n = token_ids.len();
		for i in 0..n {
			let token_id = token_ids[i];
			let amount = amounts[i];

			Balances::<T>::try_mutate(from, (tao_id, token_id), |balance| -> DispatchResult {
				*balance = balance
					.checked_sub(amount)
					.ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;
		}

		Self::deposit_event(Event::BatchBurn(from.clone(), tao_id, token_ids, amounts));

		Ok(())
	}

	// func_id 1009
	// do_transfer_from(
	// 		from: &T::AccountId,
	// 		to: &T::AccountId,
	// 		tao_id: T::TaoId,
	// 		token_id: T::TokenId,
	// 		amount: Balance
	// 	)
	pub fn do_transfer_from(
		from: &T::AccountId,
		to: &T::AccountId,
		tao_id: T::TaoId,
		token_id: T::TokenId,
		amount: Balance
	) -> DispatchResult {
		debug::info!("run erc1155: do_transfer_from");

		if from == to {
			return Ok(());
		}

		Balances::<T>::try_mutate(from, (tao_id, token_id), |balance| -> DispatchResult {
			*balance = balance
				.checked_sub(amount)
				.ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Balances::<T>::try_mutate(to, (tao_id, token_id), |balance| -> DispatchResult {
			*balance = balance
				.checked_add(amount)
				.ok_or(Error::<T>::NumOverflow)?;
			Ok(())
		})?;

		Self::deposit_event(Event::Transferred(from.clone(), to.clone(), tao_id, token_id, amount));

		Ok(())
	}

	// func_id 1010
	// do_batch_transfer_from(
	// 		from: &T::AccountId,
	// 		to: &T::AccountId,
	// 		tao_id: T::TaoId,
	// 		token_ids: Vec<T::TokenId>,
	// 		amounts: Vec<Balance>
	// 	)
	pub fn do_batch_transfer_from(
		from: &T::AccountId,
		to: &T::AccountId,
		tao_id: T::TaoId,
		token_ids: Vec<T::TokenId>,
		amounts: Vec<Balance>
	) -> DispatchResult {
		if from == to {
			return Ok(());
		}

		ensure!(token_ids.len() == amounts.len(), Error::<T>::InvalidArrayLength);

		let n = token_ids.len();
		for i in 0..n {
			let token_id = &token_ids[i];
			let amount = amounts[i];

			Balances::<T>::try_mutate(from, (tao_id, token_id), |balance| -> DispatchResult {
				*balance = balance
					.checked_sub(amount)
					.ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;

			Balances::<T>::try_mutate(to, (tao_id, token_id), |balance| -> DispatchResult {
				*balance = balance
					.checked_add(amount)
					.ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;
		}

		Self::deposit_event(Event::BatchTransferred(from.clone(), to.clone(), tao_id, token_ids, amounts));

		Ok(())
	}

	// func_id 1011  approved_or_owner(who: &T::AccountId, account: &T::AccountId) -> bool
	pub fn approved_or_owner(who: &T::AccountId, account: &T::AccountId) -> bool {
		*account != T::AccountId::default()
			&& (*who == *account || Self::operator_approvals(who, account))
	}

	// func_id 1012 is_approved_for_all(owner: &T::AccountId, operator: &T::AccountId) -> bool
	pub fn is_approved_for_all(owner: &T::AccountId, operator: &T::AccountId) -> bool {
		Self::operator_approvals(owner, operator)
	}

	// func_id 1013 fn balance_of(owner: &T::AccountId, tao_id: T::TaoId, token_id: T::TokenId) -> Balance
	pub fn balance_of(owner: &T::AccountId, tao_id: T::TaoId, token_id: T::TokenId) -> Balance {
		debug::info!("run erc1155: balance_of");

		Self::balances(owner, (tao_id, token_id))
	}

	// func_id 1014 balance_of_batch(owners: &Vec<T::AccountId>, tao_id: T::TaoId, token_ids: Vec<T::TokenId>) -> Result<Vec<Balance>, DispatchError>
	pub fn balance_of_batch(owners: &Vec<T::AccountId>, tao_id: T::TaoId, token_ids: Vec<T::TokenId>) -> Result<Vec<Balance>, DispatchError> {
		ensure!(owners.len() == token_ids.len(), Error::<T>::InvalidArrayLength);

		let mut batch_balances = vec![Balance::from(0u32); owners.len()];

		let n = owners.len();
		for i in 0..n {
			let owner = &owners[i];
			let token_id = token_ids[i];

			batch_balances[i] = Self::balances(owner, (tao_id, token_id));
		}

		Ok(batch_balances)
	}
}
