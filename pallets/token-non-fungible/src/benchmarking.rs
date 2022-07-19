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
}

impl_benchmark_test_suite!(TokenFungible, crate::mock::new_test_ext(), crate::mock::Test,);
