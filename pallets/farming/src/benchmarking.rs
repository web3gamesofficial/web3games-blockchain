//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Farming;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::{Origin, Pallet as System, RawOrigin};
use pallet_token_fungible as TokenFungbile;
use pallet_token_fungible::Pallet as TokenFungible;
use sp_runtime::traits::{StaticLookup, UniqueSaturatedFrom};

const W3G: u128 = 1;
const USDT: u128 = 2;
const W3G_DECIMALS: u128 = 1_000_000_000_000_000_000;
const USDT_DECIMALS: u128 = 1_000_000;

fn setup<T: Config>() -> DispatchResult {
	let alice: T::AccountId = account("alice", 0, 0);
	let bob: T::AccountId = account("bob", 0, 0);

	assert_ok!(TokenFungible::<T>::create_token(
		RawOrigin::Signed(alice.clone()).into(),
		<T as pallet_token_fungible::Config>::FungibleTokenId::unique_saturated_from(W3G),
		b"TestToken".to_vec(),
		b"TK".to_vec(),
		18
	));
	assert_ok!(TokenFungible::<T>::create_token(
		RawOrigin::Signed(alice.clone()).into(),
		<T as pallet_token_fungible::Config>::FungibleTokenId::unique_saturated_from(USDT),
		b"TestToken".to_vec(),
		b"TK".to_vec(),
		18
	));
	assert_ok!(TokenFungible::<T>::mint(
		RawOrigin::Signed(alice.clone()).into(),
		<T as pallet_token_fungible::Config>::FungibleTokenId::unique_saturated_from(USDT),
		alice.clone(),
		100 * USDT_DECIMALS,
	));
	assert_ok!(TokenFungible::<T>::mint(
		RawOrigin::Signed(alice.clone()).into(),
		<T as pallet_token_fungible::Config>::FungibleTokenId::unique_saturated_from(W3G),
		bob,
		100 * W3G_DECIMALS,
	));
	Ok(())
}

benchmarks! {
	create_pool {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		setup::<T>()?;
		assert_ok!(Farming::<T>::set_admin(RawOrigin::Root.into(),alice.clone()));
	}: _(RawOrigin::Signed(alice),T::BlockNumber::from(1u32),T::BlockNumber::from(10u32),T::BlockNumber::from(10u32),W3G,USDT,10 * USDT_DECIMALS)

	staking {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		setup::<T>()?;
		assert_ok!(Farming::<T>::set_admin(RawOrigin::Root.into(),alice.clone()));
		assert_ok!(Farming::<T>::create_pool(
				RawOrigin::Signed(alice.clone()).into(),
				T::BlockNumber::from(10u32),
				T::BlockNumber::from(10u32),
				T::BlockNumber::from(10u32),
				W3G,
				USDT,
				10 * USDT_DECIMALS,
		));
		System::<T>::set_block_number(T::BlockNumber::from(10u32));
	}: _(RawOrigin::Signed(bob),0,10 * W3G_DECIMALS)

	claim {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		setup::<T>()?;
		assert_ok!(Farming::<T>::set_admin(RawOrigin::Root.into(),alice.clone()));
		assert_ok!(Farming::<T>::create_pool(
				RawOrigin::Signed(alice.clone()).into(),
				T::BlockNumber::from(10u32),
				T::BlockNumber::from(10u32),
				T::BlockNumber::from(10u32),
				W3G,
				USDT,
				10 * USDT_DECIMALS,
		));
		System::<T>::set_block_number(T::BlockNumber::from(10u32));
		assert_ok!(Farming::<T>::staking(
				RawOrigin::Signed(bob.clone()).into(),
				0,
				10 * W3G_DECIMALS,
		));
		System::<T>::set_block_number(T::BlockNumber::from(30u32));
	}: _(RawOrigin::Signed(bob),0)

	force_claim {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		setup::<T>()?;
		assert_ok!(Farming::<T>::set_admin(RawOrigin::Root.into(),alice.clone()));
		assert_ok!(Farming::<T>::create_pool(
				RawOrigin::Signed(alice.clone()).into(),
		T::BlockNumber::from(10u32),
		T::BlockNumber::from(10u32),
		T::BlockNumber::from(10u32),
		W3G,
		USDT,
		10 * USDT_DECIMALS,
		));
		System::<T>::set_block_number(T::BlockNumber::from(10u32));
		assert_ok!(Farming::<T>::staking(
				RawOrigin::Signed(bob.clone()).into(),
		0,
		10 * W3G_DECIMALS,
		));
	}: _(RawOrigin::Signed(alice),bob,0)

	impl_benchmark_test_suite!(Farming, crate::mock::new_test_ext(), crate::mock::Test);
}
