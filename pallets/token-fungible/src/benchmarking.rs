//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as TokenFungible;
use codec::alloc::string::ToString;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use primitives::currency::CurrencyId;
use sp_runtime::traits::{Hash, UniqueSaturatedFrom};

const SEED: u32 = 0;

benchmarks! {
	create_token {
		let alice: T::AccountId = account("alice", 0, SEED);
	}: _(RawOrigin::Signed(alice), 1u128, "TestToken".to_string().into(), "TK".to_string().into(), 18)

	mint {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);

		let _ = TokenFungible::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u128, "TestToken".to_string().into(), "TK".to_string().into(), 18);
	}: _(RawOrigin::Signed(alice), 1u128, alice, 100_000_000_000_000u128)

	approve {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);

		let _ = TokenFungible::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u128, "TestToken".to_string().into(), "TK".to_string().into(), 18);
		let _ = TokenFungible::<T>::mint(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u128, alice, 100_000_000_000_000u128);
	}: _(RawOrigin::Signed(alice), 1u128, bob, 100_000_000_000u128)

	burn {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);

		let _ = TokenFungible::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u128, "TestToken".to_string().into(), "TK".to_string().into(), 18);
		let _ = TokenFungible::<T>::mint(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u128, alice, 100_000_000_000_000u128);
	}: _(RawOrigin::Signed(alice), 1u128, 100_000_000_000_000u128)

	transfer {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);

		let _ = TokenFungible::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u128, "TestToken".to_string().into(), "TK".to_string().into(), 18);
		let _ = TokenFungible::<T>::mint(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u128, alice, 100_000_000_000_000u128);
	}: _(RawOrigin::Signed(alice), 1u128, bob, 100_000_000_000u128)

	transfer_from {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);
		let charlie: T::AccountId = account("charlie", 0, SEED);

		let _ = TokenFungible::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u128, "TestToken".to_string().into(), "TK".to_string().into(), 18);
		let _ = TokenFungible::<T>::mint(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u128, alice, 100_000_000_000_000u128);
		let _ = TokenFungible::<T>::approve(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u128, charlie, 100_000_000_000_000u128);
	}: _(RawOrigin::Signed(charlie), 1u128, alice, bob, 100_000_000_000u128)
}

impl_benchmark_test_suite!(TokenFungible, crate::mock::new_test_ext(), crate::mock::Test,);
