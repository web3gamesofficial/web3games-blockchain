#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_runtime::{
	RuntimeDebug, ModuleId,
	traits::{
		AccountIdConversion, One, Zero,
	},
};
use frame_support::{
	ensure,
	dispatch::{DispatchResult, DispatchError},
};
use sp_core::U256;
use sp_std::{fmt::Debug, convert::TryInto, prelude::*};
use primitives::{CurrencyId, Balance};
use frame_support::debug;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type ExchangeId = u32;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + currency_token::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type ModuleId: Get<ModuleId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Exchanges<T: Config> = StorageMap<
		_,
		Blake2_128,
		ExchangeId,
		Exchange<T::TaoId, T::TokenId, T::AccountId>
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_exchange_id)]
	pub(super) type NextExchangeId<T: Config> = StorageValue<
		_,
		ExchangeId,
		ValueQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn total_supplies)]
	pub(super) type TotalSupplies<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::TokenId,
		Balance,
		ValueQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn currency_reserves)]
	pub(super) type CurrencyReserves<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::TokenId,
		Balance,
		ValueQuery
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ExchangeCreated(ExchangeId, T::AccountId),
		CurrencyToToken(ExchangeId, T::AccountId, T::AccountId, Vec<T::TokenId>, Vec<Balance>, Vec<Balance>),
		TokenToCurrency(ExchangeId, T::AccountId, T::AccountId, Vec<T::TokenId>, Vec<Balance>, Vec<Balance>),
		LiquidityAdded(T::AccountId, T::AccountId, Vec<T::TokenId>, Vec<Balance>, Vec<Balance>),
		LiquidityRemoved(T::AccountId, T::AccountId, Vec<T::TokenId>, Vec<Balance>, Vec<Balance>),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unknown,
		Overflow,
		InvalidExchangeId,
		NoAvailableExchangeId,
		InvalidMaxCurrency,
		InsufficientCurrencyAmount,
		InsufficientTokenAmount,
		SameCurrencyAndToken,
		MaxCurrencyAmountExceeded,
		InvalidCurrencyAmount,
		InsufficientLiquidity,
		InsufficientOutputAmount,
		InsufficientInputAmount,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
		pub fn create_exchange(origin: OriginFor<T>, currency_id: CurrencyId, token_tao: T::TaoId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let exchange_id =
				NextExchangeId::<T>::try_mutate(|id| -> Result<ExchangeId, DispatchError> {
					let current_id = *id;
					*id = id
						.checked_add(One::one())
						.ok_or(Error::<T>::NoAvailableExchangeId)?;
					Ok(current_id)
				})?;

			let fund_id = <T as Config>::ModuleId::get().into_sub_account(exchange_id);
			let liquidity_tao = token::Module::<T>::do_create_tao(&fund_id, [].to_vec())?;

			let (currency_tao, currency_token) = currency_token::Module::<T>::get_currency_token(currency_id)?;

			let new_exchange = Exchange {
				creator: who.clone(),
				token_tao,
				currency_tao,
				currency_token,
				liquidity_tao,
				vault: fund_id,
			};

			Exchanges::<T>::insert(exchange_id, new_exchange);

			Self::deposit_event(Event::ExchangeCreated(exchange_id, who));

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn buy_tokens(
			origin: OriginFor<T>,
			exchange_id: ExchangeId,
			token_ids: Vec<T::TokenId>,
			token_amounts_out: Vec<Balance>,
			max_currency: Balance,
			to: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_buy_tokens(&who, exchange_id, token_ids, token_amounts_out, max_currency, &to)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn sell_tokens(
			origin: OriginFor<T>,
			exchange_id: ExchangeId,
			token_ids: Vec<T::TokenId>,
			token_amounts_in: Vec<Balance>,
			min_currency: Balance,
			to: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_sell_tokens(&who, exchange_id, token_ids, token_amounts_in, min_currency, &to)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			exchange_id: ExchangeId,
			to: T::AccountId,
			token_ids: Vec<T::TokenId>,
			token_amounts: Vec<Balance>,
			max_currencys: Vec<Balance>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_add_liquidity(&who, exchange_id, &to, token_ids, token_amounts, max_currencys)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			exchange_id: ExchangeId,
			to: T::AccountId,
			token_ids: Vec<T::TokenId>,
			liquidities: Vec<Balance>,
			min_currencys: Vec<Balance>,
			min_tokens: Vec<Balance>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_remove_liquidity(&who, exchange_id, &to, token_ids, liquidities, min_currencys, min_tokens)?;

			Ok(().into())
		}

	}

}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Exchange<
	TaoId: Encode + Decode + Clone + Debug + Eq + PartialEq,
	TokenId: Encode + Decode + Clone + Debug + Eq + PartialEq,
	AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
	/// The creator of Exchange
	pub creator: AccountId,
	/// The tao of the ERC-1155 token
	pub token_tao: TaoId,
	/// The tao of the currency
	pub currency_tao: TaoId,
	/// The token of the currency tao
	pub currency_token: TokenId,
	/// The tao of exchange liquidity pool 
	pub liquidity_tao: TaoId,
	/// The fund account of exchange
	pub vault: AccountId,
}

impl<T: Config> Pallet<T> {
	// currency to token
	pub fn do_buy_tokens(
		who: &T::AccountId,
		exchange_id: ExchangeId,
		token_ids: Vec<T::TokenId>,
		token_amounts_out: Vec<Balance>,
		max_currency: Balance,
		to: &T::AccountId,
	) -> DispatchResult {
		let exchange = Exchanges::<T>::get(exchange_id).ok_or(Error::<T>::InvalidExchangeId)?;

		// Transfer currency token to exchange vault
		token::Module::<T>::do_transfer_from(who, &exchange.vault, exchange.currency_tao, exchange.currency_token, max_currency)?;

		let n = token_ids.len();
		let mut total_refund_currency: Balance = max_currency;
		let mut amounts_in = vec![Balance::from(0u128); n];

		let token_reserves = Self::get_token_reserves(&exchange.vault, exchange.token_tao, token_ids.clone());

		for i in 0..n {
			let id = token_ids[i];
			let amount_out = token_amounts_out[i];
			let token_reserve = token_reserves[i];

			let currency_reserve = Self::currency_reserves(id);
			let currency_amount = Self::get_amount_in(amount_out, currency_reserve, token_reserve)?;

			total_refund_currency -= currency_amount;

			amounts_in[i] = currency_amount;

			CurrencyReserves::<T>::mutate(id, |currency_reserve| *currency_reserve += currency_amount);
		}

		// Refund currency token if any
		if total_refund_currency > Zero::zero()  {
			token::Module::<T>::do_transfer_from(&exchange.vault, &to, exchange.currency_tao, exchange.currency_token, total_refund_currency)?;
		}

		// Send Tokens all tokens purchased
		token::Module::<T>::do_batch_transfer_from(&exchange.vault, &to, exchange.token_tao, token_ids.clone(), token_amounts_out.clone())?;

		Self::deposit_event(Event::CurrencyToToken(exchange_id, who.clone(), to.clone(), token_ids, token_amounts_out, amounts_in));

		Ok(())
	}

	// token to currency
	pub fn do_sell_tokens(
		who: &T::AccountId,
		exchange_id: ExchangeId,
		token_ids: Vec<T::TokenId>,
		token_amounts_in: Vec<Balance>,
		min_currency: Balance,
		to: &T::AccountId,
	) -> DispatchResult {
		let exchange = Exchanges::<T>::get(exchange_id).ok_or(Error::<T>::InvalidExchangeId)?;

		// Transfer the tokens to sell to exchange vault
		token::Module::<T>::do_batch_transfer_from(who, &exchange.vault, exchange.token_tao, token_ids.clone(), token_amounts_in.clone())?;

		let n = token_ids.len();
		let mut total_currency = Balance::from(0u128);
		let mut amounts_out = vec![Balance::from(0u128); n];

		let token_reserves = Self::get_token_reserves(&exchange.vault, exchange.token_tao, token_ids.clone());

		for i in 0..n {
			let id = token_ids[i];
			let amount_in = token_amounts_in[i];
			let token_reserve = token_reserves[i];

			let currency_reserve = Self::currency_reserves(id);
			let currency_amount = Self::get_amount_out(amount_in, token_reserve - amount_in, currency_reserve)?;

			total_currency += currency_amount;
			amounts_out[i] = currency_amount;

			CurrencyReserves::<T>::mutate(id, |currency_reserve| *currency_reserve -= currency_amount);
		}

		ensure!(total_currency >= min_currency, Error::<T>::InsufficientCurrencyAmount);

		// Transfer currency here
		token::Module::<T>::do_transfer_from(&exchange.vault, &to, exchange.currency_tao, exchange.currency_token, total_currency)?;

		Self::deposit_event(Event::TokenToCurrency(exchange_id, who.clone(), to.clone(), token_ids, token_amounts_in, amounts_out));

		Ok(())
	}

	// add liquidity
	pub fn do_add_liquidity(
		who: &T::AccountId,
		exchange_id: ExchangeId,
		to: &T::AccountId,
		token_ids: Vec<T::TokenId>,
		token_amounts: Vec<Balance>,
		max_currencys: Vec<Balance>,
	) -> DispatchResult {
		let exchange = Exchanges::<T>::get(exchange_id).ok_or(Error::<T>::InvalidExchangeId)?;

		debug::info!("do_add_liquidity");
		debug::info!("amount0: {:?}, amount1: {:?}", token_amounts[0], token_amounts[1]);
		debug::info!("currency0: {:?}, currency1: {:?}", max_currencys[0], max_currencys[1]);

		// Transfer the tokens to add to the exchange liquidity pools
		token::Module::<T>::do_batch_transfer_from(who, &exchange.vault, exchange.token_tao, token_ids.clone(), token_amounts.clone())?;

		let n = token_ids.len();
		let mut total_currency = Balance::from(0u128);
		let mut liquidities_to_mint = vec![Balance::from(0u128); n];
		let mut currency_amounts = vec![Balance::from(0u128); n];

		let token_reserves = Self::get_token_reserves(&exchange.vault, exchange.token_tao, token_ids.clone());

		for i in 0..n {
			let id = token_ids[i];
			let amount = token_amounts[i];

			ensure!(max_currencys[i] > Zero::zero() , Error::<T>::InvalidMaxCurrency);
			ensure!(amount > Zero::zero() , Error::<T>::InsufficientTokenAmount);

			if exchange.currency_tao == exchange.token_tao {
				ensure!(exchange.currency_token != id, Error::<T>::SameCurrencyAndToken);
			}

			let total_liquidity = Self::total_supplies(id);

			if total_liquidity > Zero::zero()  {
				let currency_reserve = Self::currency_reserves(id);
				let token_reserve = token_reserves[i];

				let (currency_amount, rounded) = Self::div_round(
					U256::from(amount).saturating_mul(U256::from(currency_reserve)),
					U256::from(token_reserve).saturating_sub(U256::from(amount)));
				ensure!(max_currencys[i] >= currency_amount, Error::<T>::MaxCurrencyAmountExceeded);

				total_currency = total_currency + currency_amount;

				let fixed_currency_amount = if rounded { currency_amount - 1u128 } else { currency_amount };
				liquidities_to_mint[i] = (fixed_currency_amount * total_liquidity) / currency_reserve;
				currency_amounts[i] = currency_amount;

				CurrencyReserves::<T>::mutate(id, |currency_reserve| *currency_reserve += currency_amount);
				TotalSupplies::<T>::mutate(id, |total_supply| *total_supply = total_liquidity + liquidities_to_mint[i]);
			} else {
				let max_currency = max_currencys[i];

				// Otherwise rounding error could end up being significant on second deposit
				ensure!(max_currency >= 1000000000u128.into(), Error::<T>::InvalidCurrencyAmount);

				total_currency = total_currency + max_currency;
				liquidities_to_mint[i] = max_currency;
				currency_amounts[i] = max_currency;

				CurrencyReserves::<T>::mutate(id, |currency_reserve| *currency_reserve = max_currency);
				TotalSupplies::<T>::mutate(id, |total_supply| *total_supply = max_currency);
			}
		}

		// Mint liquidity pool tokens
		token::Module::<T>::do_batch_mint(&to, exchange.liquidity_tao, token_ids.clone(), liquidities_to_mint)?;

		// Transfer all currency to this contract
		token::Module::<T>::do_transfer_from(&who, &exchange.vault, exchange.currency_tao, exchange.currency_token, total_currency)?;

		Self::deposit_event(Event::LiquidityAdded(who.clone(), to.clone(), token_ids, token_amounts, currency_amounts));

		Ok(())
	}

	// remove liquidity
	pub fn do_remove_liquidity(
		who: &T::AccountId,
		exchange_id: ExchangeId,
		to: &T::AccountId,
		token_ids: Vec<T::TokenId>,
		liquidities: Vec<Balance>,
		min_currencys: Vec<Balance>,
		min_tokens: Vec<Balance>,
	) -> DispatchResult {
		let exchange = Exchanges::<T>::get(exchange_id).ok_or(Error::<T>::InvalidExchangeId)?;

		// Transfer the liquidity pool tokens to burn to exchange vault
		token::Module::<T>::do_batch_transfer_from(who, &exchange.vault, exchange.liquidity_tao, token_ids.clone(), liquidities.clone())?;

		let n = token_ids.len();
		let mut total_currency = Balance::from(0u128);
		let mut token_amounts = vec![Balance::from(0u128); n];
		let mut currency_amounts = vec![Balance::from(0u128); n];

		let token_reserves = Self::get_token_reserves(&exchange.vault, exchange.token_tao, token_ids.clone());

		for i in 0..n {
			let id = token_ids[i];
			let liquidity = liquidities[i];
			let token_reserve = token_reserves[i];

			let total_liquidity = Self::total_supplies(id);
			ensure!(total_liquidity > Zero::zero() , Error::<T>::InsufficientLiquidity);

			let currency_reserve = Self::currency_reserves(id);

			let currency_amount = liquidity * currency_reserve / total_liquidity;
			let token_amount = liquidity * token_reserve / total_liquidity;

			ensure!(currency_amount >= min_currencys[i], Error::<T>::InsufficientCurrencyAmount);
			ensure!(token_amount >= min_tokens[i], Error::<T>::InsufficientTokenAmount);

			total_currency += currency_amount;
			token_amounts[i] = token_amount;
			currency_amounts[i] = currency_amount;

			CurrencyReserves::<T>::mutate(id, |currency_reserve| *currency_reserve -= currency_amount);
			TotalSupplies::<T>::mutate(id, |total_supply| *total_supply = total_liquidity - liquidity);
		}

		// Burn liquidity pool tokens for offchain supplies
		token::Module::<T>::do_batch_burn(&exchange.vault, exchange.liquidity_tao, token_ids.clone(), liquidities)?;

		// Transfer total currency and all Tokens ids
		token::Module::<T>::do_transfer_from(&exchange.vault, &to, exchange.currency_tao, exchange.currency_token, total_currency)?;
		token::Module::<T>::do_batch_transfer_from(&exchange.vault, &to, exchange.token_tao, token_ids.clone(), token_amounts.clone())?;

		Self::deposit_event(Event::LiquidityRemoved(who.clone(), to.clone(), token_ids, token_amounts, currency_amounts));

		Ok(())
	}

	fn get_amount_in(
		amount_out: Balance,
		reserve_in: Balance,
		reserve_out: Balance,
	) -> Result<Balance, DispatchError> {
		ensure!(amount_out > Zero::zero() , Error::<T>::InsufficientOutputAmount);
		ensure!(reserve_in > Zero::zero()  && reserve_out > Zero::zero() , Error::<T>::InsufficientLiquidity);

		let numerator: U256 = U256::from(reserve_in).saturating_mul(U256::from(amount_out)).saturating_mul(U256::from(1000u128));
		let denominator: U256 = (U256::from(reserve_out).saturating_sub(U256::from(amount_out))).saturating_mul(U256::from(995u128));
		let (amount_in, _) = Self::div_round(numerator, denominator);

		Ok(amount_in)
	}

	fn get_amount_out(
		amount_in: Balance,
		reserve_in: Balance,
		reserve_out: Balance,
	) -> Result<Balance, DispatchError> {
		ensure!(amount_in > Zero::zero() , Error::<T>::InsufficientInputAmount);
		ensure!(reserve_in > Zero::zero()  && reserve_out > Zero::zero() , Error::<T>::InsufficientLiquidity);

		let amount_in_with_fee: U256 = U256::from(amount_in).saturating_mul(U256::from(995u128));
		let numerator: U256 = U256::from(amount_in_with_fee).saturating_mul(U256::from(reserve_out));
		let denominator: U256 = (U256::from(reserve_in).saturating_mul(U256::from(1000u128))).saturating_add(amount_in_with_fee);

		let amount_out = numerator
			.checked_div(denominator)
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())
			.unwrap_or_else(Zero::zero);

		Ok(amount_out)
	}

	fn get_token_reserves(vault: &T::AccountId, tao_id: T::TaoId, token_ids: Vec<T::TokenId>) -> Vec<Balance> {
		let n = token_ids.len();

		if n == 1 {
			let mut token_reserves = vec![Balance::from(0u128); n];
			token_reserves[0] = token::Module::<T>::balance_of(vault, tao_id, token_ids[0]);
			token_reserves
		} else {
			let vaults = vec![vault.clone(); n];
			let token_reserves = token::Module::<T>::balance_of_batch(&vaults, tao_id, token_ids).unwrap();
			token_reserves
		}
	}

	/// Divides two numbers and add 1 if there is a rounding error
	fn div_round(numerator: U256, denominator: U256) -> (Balance, bool) {
		let remainder = numerator.checked_rem(denominator).unwrap();
		if remainder.is_zero() {
			(numerator
				.checked_div(denominator)
				.and_then(|n| TryInto::<Balance>::try_into(n).ok())
				.unwrap_or_else(Zero::zero)
			, false)
		} else {
			(numerator
				.checked_div(denominator)
				.and_then(|r| r.checked_add(U256::one()))
				.and_then(|n| TryInto::<Balance>::try_into(n).ok())
				.unwrap_or_else(Zero::zero)
			, true)
		}
	}
}
