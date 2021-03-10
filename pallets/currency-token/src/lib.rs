#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::dispatch::{DispatchResult, DispatchError};
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, CurrencyId};
use sp_runtime::{ModuleId, RuntimeDebug};
use sp_std::{fmt::Debug, prelude::*};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + token::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type ModuleId: Get<ModuleId>;

        type Currency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId = CurrencyId,
            Balance = Balance,
        >;
    }

    pub type GenesisInstance<T> = (
        <T as frame_system::Config>::AccountId,
        Vec<u8>,
    );

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub instance: GenesisInstance<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { instance: Default::default() }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::create_instance(&self.instance.0, self.instance.1.to_vec())
                .expect("Create instance cannot fail while building genesis");
        }
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn currency_instance)]
    pub(super) type CurrencyInstance<T: Config> = StorageValue<_, T::InstanceId, OptionQuery>;

    #[pallet::storage]
    pub(super) type CurrencyTokens<T: Config> =
        StorageMap<_, Blake2_128Concat, CurrencyId, TokenInfo<T::InstanceId, T::TokenId, Balance>>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
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
        CurrencyInstanceNotCreated,
        CurrencyTokenNotFound,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn mint(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            to: T::AccountId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            <T as Config>::Currency::deposit(currency_id, &who, amount)?;

            let instance_id = CurrencyInstance::<T>::get().ok_or(Error::<T>::CurrencyInstanceNotCreated)?;

            if !CurrencyTokens::<T>::contains_key(currency_id) {
                let token_id = Self::convert_to_token_id(currency_id);
                token::Module::<T>::do_create_token(&who, instance_id, token_id, false, [].to_vec())?;

                let token_info = TokenInfo {
                    instance_id,
                    token_id: token_id.clone(),
                    total_supply: Default::default(),
                };

                CurrencyTokens::<T>::insert(currency_id, token_info);
            }

            CurrencyTokens::<T>::try_mutate(currency_id, |token_info| -> DispatchResult {
                let info = token_info
                    .as_mut()
                    .ok_or(Error::<T>::Unknown)?;

                token::Module::<T>::do_mint(&who, &to, instance_id, info.token_id, amount)?;

                info.total_supply = info
                    .total_supply
                    .checked_add(amount)
                    .ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::TokenMint(currency_id, amount, who));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn burn(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            from: T::AccountId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            <T as Config>::Currency::withdraw(currency_id, &who, amount)?;

            CurrencyTokens::<T>::try_mutate(currency_id, |token_info| -> DispatchResult {
                let info = token_info
                    .as_mut()
                    .ok_or(Error::<T>::CurrencyTokenNotFound)?;

                let instance_id = CurrencyInstance::<T>::get().ok_or(Error::<T>::CurrencyInstanceNotCreated)?;

                token::Module::<T>::do_burn(&who, &from, instance_id, info.token_id, amount)?;

                info.total_supply = info
                    .total_supply
                    .checked_sub(amount)
                    .ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::TokenBurn(currency_id, amount, who));

            Ok(().into())
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct TokenInfo<
    InstanceId: Encode + Decode + Clone + Debug + Eq + PartialEq,
    TokenId: Encode + Decode + Clone + Debug + Eq + PartialEq,
    Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
    instance_id: InstanceId,
    token_id: TokenId,
    total_supply: Balance,
}

impl<T: Config> Pallet<T> {
    pub fn create_instance(who: &T::AccountId, data: Vec<u8>) -> DispatchResult {
        let instance_id = token::Module::<T>::do_create_instance(who, data)?;
        CurrencyInstance::<T>::put(instance_id);

        Ok(())
    }

    pub fn get_currency_token(
        currency_id: CurrencyId,
    ) -> Result<(T::InstanceId, T::TokenId), DispatchError> {
        let token_info =
            CurrencyTokens::<T>::get(currency_id).ok_or(Error::<T>::CurrencyTokenNotFound)?;
        Ok((token_info.instance_id, token_info.token_id))
    }

    pub fn convert_to_token_id(id: CurrencyId) -> T::TokenId {
        let n: u64 = id.into();
        n.into()
    }
}
