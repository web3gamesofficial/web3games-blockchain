#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as WrapCurrency;
use frame_benchmarking::{account, benchmarks};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use pallet_balances::Pallet as Balances;
use sp_runtime::traits::StaticLookup;

const W3G_DECIMALS: u128 = 1_000_000_000_000_000_000;

pub fn lookup_of_account<T: Config>(
	who: T::AccountId,
) -> <<T as frame_system::Config>::Lookup as StaticLookup>::Source {
	<T as frame_system::Config>::Lookup::unlookup(who)
}

benchmarks! {
	where_clause {
		where
			T: Config + pallet_balances::Config<Balance = u128>
	}
	deposit {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		// assert_ok!(TokenFungible::<T>::create_token(
		// 		RawOrigin::Signed(WrapCurrency::<T>::account_id()).into(),
		// 		<T as web3games_token_fungible::Config>::FungibleTokenId::unique_saturated_from(W3G),
		// 		b"W3G".to_vec(),
		// 		b"W3G".to_vec(),
		// 		18
		// ));
		assert_ok!(Balances::<T>::set_balance(
				RawOrigin::Root.into(),
		lookup_of_account::<T>(alice.clone()),
		100 * W3G_DECIMALS,
		0,
		));

	}: _(RawOrigin::Signed(alice), 10 * W3G_DECIMALS)

	withdraw {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		// assert_ok!(TokenFungible::<T>::create_token(
		// 		RawOrigin::Signed(WrapCurrency::<T>::account_id()).into(),
		// 		<T as web3games_token_fungible::Config>::FungibleTokenId::unique_saturated_from(W3G),
		// 		b"W3G".to_vec(),
		// 		b"W3G".to_vec(),
		// 		18
		// ));
		assert_ok!(Balances::<T>::set_balance(
				RawOrigin::Root.into(),
		lookup_of_account::<T>(alice.clone()),
		100 * W3G_DECIMALS,
		0,
		));
		assert_ok!(WrapCurrency::<T>::deposit(
				RawOrigin::Signed(alice.clone()).into(),
		10 * W3G_DECIMALS,
		));
	}: _(RawOrigin::Signed(alice), 5 * W3G_DECIMALS)

	impl_benchmark_test_suite!(WrapCurrency, crate::mock::new_test_ext(), crate::mock::Test);
}
