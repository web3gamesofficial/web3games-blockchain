//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Launchpad;
use frame_benchmarking::{account, benchmarks};
use frame_support::assert_ok;
use frame_system::{Pallet as System, RawOrigin};
use sp_runtime::traits::UniqueSaturatedFrom;
use web3games_token_fungible::Pallet as TokenFungible;

const W3G: u128 = 1;
const USDT: u128 = 2;
const W3G_DECIMALS: u128 = 1_000_000_000_000_000_000;
const USDT_DECIMALS: u128 = 1_000_000;

fn setup<T: Config>() -> DispatchResult {
	let alice: T::AccountId = account("alice", 0, 0);
	let bob: T::AccountId = account("bob", 0, 0);

	assert_ok!(TokenFungible::<T>::create_token(
		RawOrigin::Signed(alice.clone()).into(),
		<T as web3games_token_fungible::Config>::FungibleTokenId::unique_saturated_from(W3G),
		b"TestToken".to_vec(),
		b"TK".to_vec(),
		18
	));
	assert_ok!(TokenFungible::<T>::create_token(
		RawOrigin::Signed(alice.clone()).into(),
		<T as web3games_token_fungible::Config>::FungibleTokenId::unique_saturated_from(USDT),
		b"TestToken".to_vec(),
		b"TK".to_vec(),
		6
	));
	assert_ok!(TokenFungible::<T>::mint(
		RawOrigin::Signed(alice.clone()).into(),
		<T as web3games_token_fungible::Config>::FungibleTokenId::unique_saturated_from(USDT),
		bob.clone(),
		100 * USDT_DECIMALS,
	));
	assert_ok!(TokenFungible::<T>::mint(
		RawOrigin::Signed(alice.clone()).into(),
		<T as web3games_token_fungible::Config>::FungibleTokenId::unique_saturated_from(W3G),
		alice,
		100 * W3G_DECIMALS,
	));
	Ok(())
}

benchmarks! {
	create_pool {
		let alice: T::AccountId = account("alice", 0, 0);
		setup::<T>()?;
	}: _(RawOrigin::Signed(alice),T::BlockNumber::from(10u32),T::BlockNumber::from(10u32),W3G,USDT,10 * W3G_DECIMALS,1 * USDT_DECIMALS)

	buy_token {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		setup::<T>()?;
		assert_ok!(Launchpad::<T>::create_pool(
				RawOrigin::Signed(alice.clone()).into(),
				T::BlockNumber::from(10u32),
				T::BlockNumber::from(10u32),
				W3G,
				USDT,
				10 * W3G_DECIMALS,
				1 * USDT_DECIMALS,
		));
		System::<T>::set_block_number(T::BlockNumber::from(10u32));
	}: _(RawOrigin::Signed(bob),0,2)

	claim {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		setup::<T>()?;
		assert_ok!(Launchpad::<T>::create_pool(
				RawOrigin::Signed(alice.clone()).into(),
				T::BlockNumber::from(10u32),
				T::BlockNumber::from(10u32),
				W3G,
				USDT,
				10 * W3G_DECIMALS,
				1 * USDT_DECIMALS,
		));
		System::<T>::set_block_number(T::BlockNumber::from(10u32));
		assert_ok!(Launchpad::<T>::buy_token(
				RawOrigin::Signed(bob.clone()).into(),
				0,
				2
		));
		System::<T>::set_block_number(T::BlockNumber::from(21u32));
	}: _(RawOrigin::Signed(bob),0)

	owner_claim {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		setup::<T>()?;
		assert_ok!(Launchpad::<T>::create_pool(
				RawOrigin::Signed(alice.clone()).into(),
				T::BlockNumber::from(10u32),
				T::BlockNumber::from(10u32),
				W3G,
				USDT,
				10 * W3G_DECIMALS,
				1 * USDT_DECIMALS,
		));
		System::<T>::set_block_number(T::BlockNumber::from(21u32));
	}: _(RawOrigin::Signed(alice),0)

	impl_benchmark_test_suite!(Launchpad, crate::mock::new_test_ext(), crate::mock::Test);
}
