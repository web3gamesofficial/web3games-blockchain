//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::Pallet as TokenMulti;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;

const SEED: u32 = 0;

benchmarks! {
	create_token {
		let alice: T::AccountId = account("alice", 0, SEED);
	}: _(RawOrigin::Signed(alice), 1u32.into(), vec![0u8; 20])

	mint {
		let alice: T::AccountId = account("alice", 0, SEED);

		let _ = TokenMulti::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 20]);
	}: _(RawOrigin::Signed(alice.clone()), 1u32.into(), alice.clone(), 1u32.into(), 10u128)

	mint_batch {
		let alice: T::AccountId = account("alice", 0, SEED);

		let _ = TokenMulti::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 20]);
	}: _(RawOrigin::Signed(alice.clone()), 1u32.into(), alice.clone(), vec![1u32.into(), 2u32.into(), 3u32.into(), 4u32.into(), 5u32.into()], vec![10u128; 5])

	set_approval_for_all {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);

		let _ = TokenMulti::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 20]);
		let _ = TokenMulti::<T>::mint(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), alice.clone(), 1u32.into(), 10u128);
	}: _(RawOrigin::Signed(alice), 1u32.into(), bob, true)

	burn {
		let alice: T::AccountId = account("alice", 0, SEED);

		let _ = TokenMulti::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 20]);
		let _ = TokenMulti::<T>::mint(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), alice.clone(), 1u32.into(), 10u128);
	}: _(RawOrigin::Signed(alice), 1u32.into(), 1u32.into(), 5u128)

	burn_batch {
		let alice: T::AccountId = account("alice", 0, SEED);

		let _ = TokenMulti::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 20]);
		let _ = TokenMulti::<T>::mint_batch(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())),1u32.into(), alice.clone(), vec![1u32.into(), 2u32.into(), 3u32.into(), 4u32.into(), 5u32.into()], vec![10u128; 5]);
	}: _(RawOrigin::Signed(alice), 1u32.into(), vec![1u32.into(), 2u32.into(), 3u32.into(), 4u32.into(), 5u32.into()], vec![5u128; 5])

	transfer_from {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);

		let _ = TokenMulti::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 20]);
		let _ = TokenMulti::<T>::mint(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), alice.clone(), 1u32.into(), 10u128);
	}: _(RawOrigin::Signed(alice.clone()), 1u32.into(), alice.clone(), bob.clone(), 1u32.into(), 5u128)

	batch_transfer_from {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);

		let _ = TokenMulti::<T>::create_token(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), vec![0u8; 20]);
		let _ = TokenMulti::<T>::mint_batch(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice.clone())), 1u32.into(), alice.clone(), vec![1u32.into(), 2u32.into(), 3u32.into(), 4u32.into(), 5u32.into()], vec![10u128; 5]);
	}: _(RawOrigin::Signed(alice.clone()), 1u32.into(), alice.clone(), bob.clone(), vec![1u32.into(), 2u32.into(), 3u32.into(), 4u32.into(), 5u32.into()], vec![5u128; 5])
}

impl_benchmark_test_suite!(TokenMulti, crate::mock::new_test_ext(), crate::mock::Test,);
