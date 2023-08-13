//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as PlayerIdModule;
use frame_benchmarking::{account, benchmarks};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use hex_literal::hex;

benchmarks! {
	where_clause {
		where
			T::AccountId: AsRef<[u8]>
	}

	register {
	}: _(RawOrigin::Signed(account("alice", 0, 0)),"tester".as_bytes().to_vec(),account("alice", 0, 0))

	add_address {
		let player_id_vec = "tester".as_bytes().to_vec();
		let player_id: PlayerId = player_id_vec.clone().try_into().unwrap();
		assert_ok!(PlayerIdModule::<T>::register(RawOrigin::Signed(account("alice", 0, 0)).into(),player_id_vec,account("alice", 0, 0)));
		let eth_address = hex!["6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"];
	}: _(RawOrigin::Signed(account("alice", 0, 0)),player_id,Chains::ETH,eth_address.to_vec())

	remove_address {
		let player_id_vec = "tester".as_bytes().to_vec();
		let player_id: PlayerId = player_id_vec.clone().try_into().unwrap();
		assert_ok!(PlayerIdModule::<T>::register(RawOrigin::Signed(account("alice", 0, 0)).into(),player_id_vec,account("alice", 0, 0)));
		let eth_address = hex!["6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"];
		assert_ok!(PlayerIdModule::<T>::add_address(RawOrigin::Signed(account("alice", 0, 0)).into(),player_id.clone(),Chains::ETH,eth_address.clone().to_vec()));
	}: _(RawOrigin::Signed(account("alice", 0, 0)),player_id.clone(),Chains::ETH,Address::try_from(eth_address.to_vec()).unwrap())

	impl_benchmark_test_suite!(PlayerIdModule, crate::mock::new_test_ext(), crate::mock::Test);
}
