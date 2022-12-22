//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Exchange;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use pallet_balances;
use web3games_token_fungible as TokenFungible;
use sp_runtime::traits::{StaticLookup, UniqueSaturatedFrom};

fn lookup_of_account<T: Config>(
	who: T::AccountId,
) -> <<T as frame_system::Config>::Lookup as StaticLookup>::Source {
	<T as frame_system::Config>::Lookup::unlookup(who)
}

fn balance_of<T: pallet_balances::Config>(
	balance: u128,
) -> <T as pallet_balances::Config>::Balance {
	<T as pallet_balances::Config>::Balance::unique_saturated_from(balance)
}

fn set_balance<T: Config + pallet_balances::Config>(who: T::AccountId) -> DispatchResult {
	assert_ok!(pallet_balances::Pallet::<T>::set_balance(
		RawOrigin::Root.into(),
		lookup_of_account::<T>(who.clone()),
		balance_of::<T>(100_000_000_000_000_000_000u128),
		balance_of::<T>(0u128),
	));
	Ok(())
}

fn create_token<T: Config>(who: T::AccountId) -> DispatchResult {
	assert_ok!(TokenFungible::Pallet::<T>::create_token(
		RawOrigin::Signed(who.clone()).into(),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(1u128),
		b"TestToken".to_vec(),
		b"TK".to_vec(),
		18
	));
	assert_ok!(TokenFungible::Pallet::<T>::create_token(
		RawOrigin::Signed(who.clone()).into(),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(2u128),
		b"TestToken".to_vec(),
		b"TK".to_vec(),
		18
	));
	assert_ok!(TokenFungible::Pallet::<T>::create_token(
		RawOrigin::Signed(who.clone()).into(),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(3u128),
		b"TestToken".to_vec(),
		b"TK".to_vec(),
		18
	));
	Ok(())
}

fn mint_token<T: Config>(who: T::AccountId) -> DispatchResult {
	assert_ok!(TokenFungible::Pallet::<T>::mint(
		RawOrigin::Signed(who.clone()).into(),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(1u128),
		who.clone(),
		100_000_000_000_000_000_000u128,
	));
	assert_ok!(TokenFungible::Pallet::<T>::mint(
		RawOrigin::Signed(who.clone()).into(),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(2u128),
		who.clone(),
		100_000_000_000_000_000_000u128,
	));
	Ok(())
}

fn init_create_pool<T: Config>(token_a: u128, token_b: u128) -> DispatchResult {
	let alice: T::AccountId = whitelisted_caller();
	assert_ok!(Exchange::<T>::create_pool(
		RawOrigin::Signed(alice.clone()).into(),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(token_a),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(token_b),
	));
	Ok(())
}

benchmarks! {
	where_clause {
		where
			T: Config + pallet_balances::Config
	}

	create_pool {
		let alice: T::AccountId = whitelisted_caller();
		set_balance::<T>(alice.clone())?;
		create_token::<T>(alice.clone())?;
	}: _(RawOrigin::Signed(alice),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(1u128),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(2u128)
	)
	verify {
		assert!(TokenFungible::Pallet::<T>::exists(<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(1u128)));
	}

	add_liquidity {
		let alice: T::AccountId = whitelisted_caller();
		set_balance::<T>(alice.clone())?;
		create_token::<T>(alice.clone())?;
		init_create_pool::<T>(
			1u128,
			2u128
		)?;
		mint_token::<T>(alice.clone())?;

	}: _(
		RawOrigin::Signed(alice.clone()),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(1u128),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(2u128),
		10_000_000_000_000_000u128,
		10_000_000_000_000_000u128,
		0u128,
		0u128,
		alice.clone(),
		<T as frame_system::Config>::BlockNumber::unique_saturated_from(10000u128)
	)

	remove_liquidity {
		let alice: T::AccountId = whitelisted_caller();
		set_balance::<T>(alice.clone())?;
		create_token::<T>(alice.clone())?;
		init_create_pool::<T>(
			1u128,
			2u128
		)?;
		mint_token::<T>(alice.clone())?;
		assert_ok!(
			pallet::Pallet::<T>::add_liquidity(
				RawOrigin::Signed(alice.clone()).into(),
				<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(1u128),
				<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(2u128),
				10_000_000_000_000_000u128,
				10_000_000_000_000_000u128,
				0u128,
				0u128,
				alice.clone(),
				<T as frame_system::Config>::BlockNumber::unique_saturated_from(10000u128)
			));
	}: _(
		RawOrigin::Signed(alice.clone()),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(1u128),
		<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(2u128),
		10_000_000_000u128,
		0u128,
		0u128,
		alice.clone(),
		<T as frame_system::Config>::BlockNumber::unique_saturated_from(10000u128)
	)

	swap_exact_tokens_for_tokens {
		let alice: T::AccountId = whitelisted_caller();
		set_balance::<T>(alice.clone())?;
		create_token::<T>(alice.clone())?;
		init_create_pool::<T>(
			1u128,
			2u128
		)?;
		mint_token::<T>(alice.clone())?;
		assert_ok!(
			pallet::Pallet::<T>::add_liquidity(
				RawOrigin::Signed(alice.clone()).into(),
				<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(1u128),
				<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(2u128),
				10_000_000_000_000_000u128,
				10_000_000_000_000_000u128,
				0u128,
				0u128,
				alice.clone(),
				<T as frame_system::Config>::BlockNumber::unique_saturated_from(10000u128)
			));
	}: _(
		RawOrigin::Signed(alice.clone()),
		1_000_000_000_000_000u128,
		0u128,
		vec![
			<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(1u128),
			<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(2u128),
		],
		alice.clone(),
		<T as frame_system::Config>::BlockNumber::unique_saturated_from(10000u128)
	)

	swap_tokens_for_exact_tokens {
		let alice: T::AccountId = whitelisted_caller();
		set_balance::<T>(alice.clone())?;
		create_token::<T>(alice.clone())?;
		init_create_pool::<T>(
			1u128,
			2u128
		)?;
		mint_token::<T>(alice.clone())?;
		assert_ok!(
			pallet::Pallet::<T>::add_liquidity(
				RawOrigin::Signed(alice.clone()).into(),
				<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(1u128),
				<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(2u128),
				10_000_000_000_000_000u128,
				10_000_000_000_000_000u128,
				0u128,
				0u128,
				alice.clone(),
				<T as frame_system::Config>::BlockNumber::unique_saturated_from(10000u128)
			));
	}: _(
		RawOrigin::Signed(alice.clone()),
		1_000_000_000_000_000u128,
		100_000_000_000_000_000u128,
		vec![
			<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(1u128),
			<T as TokenFungible::Config>::FungibleTokenId::unique_saturated_from(2u128),
		],
		alice.clone(),
		<T as frame_system::Config>::BlockNumber::unique_saturated_from(10000u128)
	)

	impl_benchmark_test_suite!(Exchange, crate::mock::new_test_ext(), crate::mock::Test);

}
