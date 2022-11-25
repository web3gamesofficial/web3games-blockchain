//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as CallSwitchgear;
use frame_benchmarking::{account, benchmarks};
use frame_support::assert_ok;
use frame_system::RawOrigin;

benchmarks! {
	switchoff_transaction {
		let alice: T::AccountId = account("alice", 0, 0);
	}: _(RawOrigin::Root,
		b"Balances".to_vec(),
		b"transfer".to_vec()
	)
	verify {
		assert_eq!(CallSwitchgear::<T>::get_switchoff_transactions((b"Balances".to_vec(),b"transfer".to_vec())),Some(()));
	}

	switchon_transaction {
		let alice: T::AccountId = account("alice", 0, 0);
		assert_ok!(CallSwitchgear::<T>::switchoff_transaction(
				RawOrigin::Root.into(),
		b"Balances".to_vec(),
		b"transfer".to_vec()
		));
		assert_eq!(
				CallSwitchgear::<T>::get_switchoff_transactions((
						b"Balances".to_vec(),
				b"transfer".to_vec()
				)),
		Some(())
		);
	}: _(RawOrigin::Root,
	b"Balances".to_vec(),
	b"transfer".to_vec()
	)
	verify {
		assert_eq!(CallSwitchgear::<T>::get_switchoff_transactions((b"Balances".to_vec(),b"transfer".to_vec())),None);
	}


	impl_benchmark_test_suite!(CallSwitchgear, crate::mock::new_test_ext(), crate::mock::Test);
}
