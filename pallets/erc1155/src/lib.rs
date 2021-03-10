#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact};
use frame_support::{
    ensure,
    dispatch::{DispatchError, DispatchResult},
    traits::{Currency, Get, ReservableCurrency},
};
use primitives::Balance;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedAdd, One, Zero},
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
pub struct Instance<AccountId> {
    owner: AccountId,
    data: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Token<InstanceId, AccountId> {
    instance_id: InstanceId,
    creator: AccountId,
    is_nf: bool,
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

        /// The minimum balance to create instance
        #[pallet::constant]
        type CreateInstanceDeposit: Get<BalanceOf<Self>>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        type TokenId: Member + Parameter + Default + Copy + HasCompact + From<u64> + Into<u64>;

        type InstanceId: Member
            + Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + From<u64>
            + Into<u64>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type Instances<T: Config> = StorageMap<_, Blake2_128Concat, T::InstanceId, Instance<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn next_instance_id)]
    pub(super) type NextInstanceId<T: Config> = StorageValue<_, T::InstanceId, ValueQuery>;

    #[pallet::storage]
    pub(super) type Tokens<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::InstanceId,
        Blake2_128,
        T::TokenId,
        Token<T::InstanceId, T::AccountId>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn balances)]
    pub(super) type Balances<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        (T::InstanceId, T::TokenId),
        Balance,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn operator_approvals)]
    pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::InstanceId,
        Blake2_128Concat,
        ApprovalKey<T::AccountId>,
        bool,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        InstanceCreated(T::InstanceId, T::AccountId),
        TokenCreated(T::InstanceId, T::TokenId, T::AccountId),
        Mint(T::AccountId, T::InstanceId, T::TokenId, Balance),
        BatchMint(T::AccountId, T::InstanceId, Vec<T::TokenId>, Vec<Balance>),
        Burn(T::AccountId, T::InstanceId, T::TokenId, Balance),
        BatchBurn(T::AccountId, T::InstanceId, Vec<T::TokenId>, Vec<Balance>),
        Transferred(T::AccountId, T::AccountId, T::InstanceId, T::TokenId, Balance),
        BatchTransferred(
            T::AccountId,
            T::AccountId,
            T::InstanceId,
            Vec<T::TokenId>,
            Vec<Balance>,
        ),
        ApprovalForAll(T::AccountId, T::AccountId, T::InstanceId, bool),
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
        NoAvailableInstanceId,
        InvalidInstanceId,
        NoPermission,
        InstanceNotFound,
        TokenNotFound,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_instance(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create_instance(&who, data)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn set_approval_for_all(
            origin: OriginFor<T>,
            operator: T::AccountId,
            instance_id: T::InstanceId,
            approved: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_set_approval_for_all(&who, &operator, instance_id, approved)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn transfer_from(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: T::AccountId,
            instance_id: T::InstanceId,
            token_id: T::TokenId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_transfer_from(&who, &from, &to, instance_id, token_id, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn batch_transfer_from(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: T::AccountId,
            instance_id: T::InstanceId,
            token_ids: Vec<T::TokenId>,
            amounts: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_batch_transfer_from(&who, &from, &to, instance_id, token_ids, amounts)?;

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn do_create_instance(who: &T::AccountId, data: Vec<u8>) -> Result<T::InstanceId, DispatchError> {
        let deposit = T::CreateInstanceDeposit::get();
        T::Currency::reserve(&who, deposit.clone())?;

        let instance_id = NextInstanceId::<T>::try_mutate(|id| -> Result<T::InstanceId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableInstanceId)?;
            Ok(current_id)
        })?;

        let instance = Instance {
            owner: who.clone(),
            data,
        };

        Instances::<T>::insert(instance_id, instance);

        Self::deposit_event(Event::InstanceCreated(instance_id, who.clone()));

        Ok(instance_id)
    }

    pub fn do_create_token(
        who: &T::AccountId,
        instance_id: T::InstanceId,
        token_id: T::TokenId,
        is_nf: bool,
        uri: Vec<u8>,
    ) -> DispatchResult {
        Self::mayby_check_owner(who, instance_id)?;
        ensure!(!Tokens::<T>::contains_key(instance_id, token_id), Error::<T>::InUse);

        Tokens::<T>::insert(
            instance_id,
            token_id,
            Token {
                instance_id,
                creator: who.clone(),
                is_nf,
                uri,
            },
        );

        Self::deposit_event(Event::TokenCreated(instance_id, token_id, who.clone()));
        Ok(())
    }

    pub fn do_set_approval_for_all(
        owner: &T::AccountId,
        operator: &T::AccountId,
        instance_id: T::InstanceId,
        approved: bool,
    ) -> DispatchResult {
        ensure!(Instances::<T>::contains_key(instance_id), Error::<T>::InstanceNotFound);

        let key = ApprovalKey {
            owner: owner.clone(),
            operator: operator.clone(),
        };
        OperatorApprovals::<T>::try_mutate(instance_id, &key, |status| -> DispatchResult {
            *status = approved;
            Ok(())
        })?;

        Self::deposit_event(Event::ApprovalForAll(
            owner.clone(),
            operator.clone(),
            instance_id,
            approved,
        ));

        Ok(())
    }

    pub fn do_mint(
        who: &T::AccountId,
        to: &T::AccountId,
        instance_id: T::InstanceId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        Self::mayby_check_owner(who, instance_id)?;
        ensure!(Tokens::<T>::contains_key(instance_id, token_id), Error::<T>::TokenNotFound);

        Balances::<T>::try_mutate(to, (instance_id, token_id), |balance| -> DispatchResult {
            *balance = balance.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
            Ok(())
        })?;

        Self::deposit_event(Event::Mint(to.clone(), instance_id, token_id, amount));

        Ok(())
    }

    pub fn do_batch_mint(
        who: &T::AccountId,
        to: &T::AccountId,
        instance_id: T::InstanceId,
        token_ids: Vec<T::TokenId>,
        amounts: Vec<Balance>,
    ) -> DispatchResult {
        Self::mayby_check_owner(who, instance_id)?;
        ensure!(token_ids.len() == amounts.len(), Error::<T>::InvalidArrayLength);

        let n = token_ids.len();
        for i in 0..n {
            let token_id = token_ids[i];
            let amount = amounts[i];

            ensure!(Tokens::<T>::contains_key(instance_id, token_id), Error::<T>::TokenNotFound);

            Balances::<T>::try_mutate(to, (instance_id, token_id), |balance| -> DispatchResult {
                *balance = balance.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;
        }

        Self::deposit_event(Event::BatchMint(to.clone(), instance_id, token_ids, amounts));

        Ok(())
    }

    pub fn do_burn(
        who: &T::AccountId,
        from: &T::AccountId,
        instance_id: T::InstanceId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        Self::mayby_check_owner(who, instance_id)?;
        ensure!(Tokens::<T>::contains_key(instance_id, token_id), Error::<T>::TokenNotFound);

        Balances::<T>::try_mutate(from, (instance_id, token_id), |balance| -> DispatchResult {
            *balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
            Ok(())
        })?;

        Self::deposit_event(Event::Burn(from.clone(), instance_id, token_id, amount));

        Ok(())
    }

    pub fn do_batch_burn(
        who: &T::AccountId,
        from: &T::AccountId,
        instance_id: T::InstanceId,
        token_ids: Vec<T::TokenId>,
        amounts: Vec<Balance>,
    ) -> DispatchResult {
        Self::mayby_check_owner(who, instance_id)?;
        ensure!(token_ids.len() == amounts.len(), Error::<T>::InvalidArrayLength);

        let n = token_ids.len();
        for i in 0..n {
            let token_id = token_ids[i];
            let amount = amounts[i];

            ensure!(Tokens::<T>::contains_key(instance_id, token_id), Error::<T>::TokenNotFound);

            Balances::<T>::try_mutate(from, (instance_id, token_id), |balance| -> DispatchResult {
                *balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;
        }

        Self::deposit_event(Event::BatchBurn(from.clone(), instance_id, token_ids, amounts));

        Ok(())
    }

    pub fn do_transfer_from(
        who: &T::AccountId,
        from: &T::AccountId,
        to: &T::AccountId,
        instance_id: T::InstanceId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        log::info!("run erc1155: do_transfer_from");

        ensure!(Self::approved_or_owner(who, from, instance_id), Error::<T>::NoPermission);

        if from == to || amount == Zero::zero() {
            return Ok(());
        }

        Balances::<T>::try_mutate(from, (instance_id, token_id), |balance| -> DispatchResult {
            *balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
            Ok(())
        })?;

        Balances::<T>::try_mutate(to, (instance_id, token_id), |balance| -> DispatchResult {
            *balance = balance.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
            Ok(())
        })?;

        Self::deposit_event(Event::Transferred(
            from.clone(),
            to.clone(),
            instance_id,
            token_id,
            amount,
        ));

        Ok(())
    }

    pub fn do_batch_transfer_from(
        who: &T::AccountId,
        from: &T::AccountId,
        to: &T::AccountId,
        instance_id: T::InstanceId,
        token_ids: Vec<T::TokenId>,
        amounts: Vec<Balance>,
    ) -> DispatchResult {
        ensure!(Self::approved_or_owner(who, from, instance_id), Error::<T>::NoPermission);

        if from == to {
            return Ok(());
        }

        ensure!(
            token_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        let n = token_ids.len();
        for i in 0..n {
            let token_id = &token_ids[i];
            let amount = amounts[i];

            Balances::<T>::try_mutate(from, (instance_id, token_id), |balance| -> DispatchResult {
                *balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            Balances::<T>::try_mutate(to, (instance_id, token_id), |balance| -> DispatchResult {
                *balance = balance.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;
        }

        Self::deposit_event(Event::BatchTransferred(
            from.clone(),
            to.clone(),
            instance_id,
            token_ids,
            amounts,
        ));

        Ok(())
    }

    pub fn approved_or_owner(
        who: &T::AccountId,
        account: &T::AccountId,
        instance_id: T::InstanceId,
    ) -> bool {
        *who == *account || Self::is_approved_for_all(who, account, instance_id)
    }

    pub fn is_approved_for_all(
        owner: &T::AccountId,
        operator: &T::AccountId,
        instance_id: T::InstanceId,
    ) -> bool {
        let key = ApprovalKey {
            owner: owner.clone(),
            operator: operator.clone(),
        };
        Self::operator_approvals(instance_id, &key)
    }

    pub fn balance_of(owner: &T::AccountId, instance_id: T::InstanceId, token_id: T::TokenId) -> Balance {
        log::info!("run erc1155: balance_of");

        Self::balances(owner, (instance_id, token_id))
    }

    pub fn balance_of_batch(
        owners: &Vec<T::AccountId>,
        instance_id: T::InstanceId,
        token_ids: Vec<T::TokenId>,
    ) -> Result<Vec<Balance>, DispatchError> {
        ensure!(
            owners.len() == token_ids.len(),
            Error::<T>::InvalidArrayLength
        );

        let mut batch_balances = vec![Balance::from(0u32); owners.len()];

        let n = owners.len();
        for i in 0..n {
            let owner = &owners[i];
            let token_id = token_ids[i];

            batch_balances[i] = Self::balances(owner, (instance_id, token_id));
        }

        Ok(batch_balances)
    }

    fn mayby_check_owner(who: &T::AccountId, instance_id: T::InstanceId) -> DispatchResult {
        let instance = Instances::<T>::get(instance_id).ok_or(Error::<T>::InvalidInstanceId)?;
        ensure!(*who == instance.owner, Error::<T>::NoPermission);

        Ok(())
    }
}
