//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::Pallet as TokenNonFungible;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;

const SEED: u32 = 0;

benchmarks! {
	create_token {
		let alice: T::AccountId = account("alice", 0, SEED);
	}: _(RawOrigin::Signed(alice), 1u32.into(), vec![0u8; 10], vec![0u8; 10], vec![0u8; 20])

	mint {
		let alice: T::AccountId = account("alice", 0, SEED);
		// let bob: T::AccountId = account("bob", 0, SEED);

		let _ = TokenNonFungible::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 10], vec![0u8; 10], vec![0u8; 20]);
	}: _(RawOrigin::Signed(alice.clone()), 1u32.into(), alice.clone(), 1u32.into())

	burn {
		let alice: T::AccountId = account("alice", 0, SEED);

		let _ = TokenNonFungible::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 10], vec![0u8; 10], vec![0u8; 20]);
		let _ = TokenNonFungible::<T>::mint(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), alice.clone(), 1u32.into());
	}: _(RawOrigin::Signed(alice), 1u32.into(), 1u32.into())

	approve {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);

		let _ = TokenNonFungible::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 10], vec![0u8; 10], vec![0u8; 20]);
		let _ = TokenNonFungible::<T>::mint(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), alice.clone(), 1u32.into());
	}: _(RawOrigin::Signed(alice), 1u32.into(), bob, 1u32.into())

	set_approve_for_all {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);

		let _ = TokenNonFungible::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 10], vec![0u8; 10], vec![0u8; 20]);
		let _ = TokenNonFungible::<T>::mint(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), alice.clone(), 1u32.into());
	}: _(RawOrigin::Signed(alice), 1u32.into(), bob, true)

	transfer_from {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);

		let _ = TokenNonFungible::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 10], vec![0u8; 10], vec![0u8; 20]);
		let _ = TokenNonFungible::<T>::mint(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), alice.clone(), 1u32.into());
	}: _(RawOrigin::Signed(alice.clone()), 1u32.into(), alice.clone(), bob, 1u32.into())
}

impl_benchmark_test_suite!(TokenNonFungible, crate::mock::new_test_ext(), crate::mock::Test,);
