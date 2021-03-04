#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_runtime::{
	RuntimeDebug, ModuleId,
};
use frame_support::dispatch::DispatchError;
use sp_std::{fmt::Debug, prelude::*};
use orml_traits::{
    MultiCurrency, MultiCurrencyExtended,
};
use primitives::{CurrencyId, Balance};

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
    pub trait Config: frame_system::Config + token::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type ModuleId: Get<ModuleId>;
        type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
	pub(super) type CurrencyTokens<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		CurrencyId,
        TokenInfo<T::TaoId, T::TokenId, Balance>
    >;
    
    #[pallet::storage]
	#[pallet::getter(fn currency_tao)]
	pub(super) type CurrencyTao<T: Config> = StorageValue<
		_,
		T::TaoId,
		ValueQuery
	>;

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
        NumOverflow,
        AlreadyTaoCreated,
        CurrencyTaoNotFound,
        CurrencyTokenNotFound,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T:Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_tao(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(!CurrencyTao::<T>::exists(), Error::<T>::AlreadyTaoCreated);

            let tao_id = token::Module::<T>::do_create_tao(&who, data)?;

            CurrencyTao::<T>::put(tao_id);

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn create_token(origin: OriginFor<T>, currency_id: CurrencyId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(CurrencyTao::<T>::exists(), Error::<T>::CurrencyTaoNotFound);
            let tao_id = CurrencyTao::<T>::get();
    
			let token_id = Self::convert_to_token_id(currency_id);
			token::Module::<T>::do_create_token(&who, tao_id, token_id, false, [].to_vec())?;
	
			let token_info = TokenInfo {
                tao_id,
				token_id: token_id.clone(),
				total_supply: Default::default()
			};

			CurrencyTokens::<T>::insert(currency_id, token_info);

			Self::deposit_event(Event::TokenCreated(currency_id, who));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn mint(origin: OriginFor<T>, currency_id: CurrencyId, amount: Balance) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(CurrencyTao::<T>::exists(), Error::<T>::CurrencyTaoNotFound);
            let tao_id = CurrencyTao::<T>::get();

            T::Currency::deposit(currency_id, &who, amount)?;

            CurrencyTokens::<T>::try_mutate(currency_id, |token_info| -> DispatchResult {
                let info = token_info
                    .as_mut()
                    .ok_or(Error::<T>::CurrencyTokenNotFound)?;

                token::Module::<T>::do_mint(&who, tao_id, info.token_id, amount)?;

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
        pub fn burn(origin: OriginFor<T>, currency_id: CurrencyId, amount: Balance) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(CurrencyTao::<T>::exists(), Error::<T>::CurrencyTaoNotFound);
            let tao_id = CurrencyTao::<T>::get();

            T::Currency::withdraw(currency_id, &who, amount)?;

            CurrencyTokens::<T>::try_mutate(currency_id, |token_info| -> DispatchResult {
                let info = token_info
					.as_mut()
                    .ok_or(Error::<T>::CurrencyTokenNotFound)?;
                
                token::Module::<T>::do_burn(&who, tao_id, info.token_id, amount)?;

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
    TaoId: Encode + Decode + Clone + Debug + Eq + PartialEq,
	TokenId: Encode + Decode + Clone + Debug + Eq + PartialEq,
	Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
    tao_id: TaoId,
	token_id: TokenId,
	total_supply: Balance,
}

impl<T: Config> Pallet<T> {
    pub fn get_currency_token(currency_id: CurrencyId) -> Result<(T::TaoId, T::TokenId), DispatchError> {
        let token_info = CurrencyTokens::<T>::get(currency_id).ok_or(Error::<T>::CurrencyTokenNotFound)?;
        Ok((token_info.tao_id, token_info.token_id))
    }

    pub fn convert_to_token_id(id: CurrencyId) -> T::TokenId {
        let n: u64 = id.into();
        n.into()
    }
}
