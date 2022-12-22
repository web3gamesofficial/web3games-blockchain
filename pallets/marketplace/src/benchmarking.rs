//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Marketplace;
use frame_benchmarking::{account, benchmarks};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use pallet_balances::Pallet as Balances;
use sp_runtime::traits::{StaticLookup, UniqueSaturatedFrom};
use web3games_token_non_fungible::Pallet as TokenNonFungible;

const W3G: u128 = 1_000_000_000_000_000_000;
const BLOCK: u32 = 1;

type NonFungibleTokenIdOf<T> = <T as web3games_token_non_fungible::Config>::NonFungibleTokenId;
type TokenIdOf<T> = <T as web3games_token_non_fungible::Config>::TokenId;

pub fn lookup_of_account<T: Config>(
	who: T::AccountId,
) -> <<T as frame_system::Config>::Lookup as StaticLookup>::Source {
	<T as frame_system::Config>::Lookup::unlookup(who)
}

fn create_non_fungible_token<T: Config>() {
	assert_ok!(TokenNonFungible::<T>::create_token(
		RawOrigin::Signed(account("alice", 0, 0)).into(),
		NonFungibleTokenIdOf::<T>::unique_saturated_from(1u128),
		b"W3G".to_vec(),
		b"W3G".to_vec(),
		b"https://web3games.com/".to_vec(),
	));

	//mint 2 to ALICE
	assert_ok!(TokenNonFungible::<T>::mint(
		RawOrigin::Signed(account("alice", 0, 0)).into(),
		NonFungibleTokenIdOf::<T>::unique_saturated_from(1u128),
		account("alice", 0, 0),
		TokenIdOf::<T>::unique_saturated_from(2u128),
	));
}

benchmarks! {
	where_clause {
		where
			T: Config + pallet_balances::Config<Balance = u128>
	}

	set_admin {
		let alice: T::AccountId = account("alice", 0, 0);
	}: _(RawOrigin::Root,alice)

	set_service_fee_point {
		let alice: T::AccountId = account("alice", 0, 0);
		assert_ok!(Marketplace::<T>::set_admin(RawOrigin::Root.into(),alice.clone()));
	}: _(RawOrigin::Signed(alice),10u8)

	create_order {
		let alice: T::AccountId = account("alice", 0, 0);
		create_non_fungible_token::<T>();
	}: _(RawOrigin::Signed(alice),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK))

	cancel_order {
		let alice: T::AccountId = account("alice", 0, 0);
		create_non_fungible_token::<T>();
		assert_ok!(Marketplace::<T>::create_order(RawOrigin::Signed(alice.clone()).into(),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK)));
	}: _(RawOrigin::Signed(alice),Asset::NonFungibleToken(1, 2))

	execute_order {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		create_non_fungible_token::<T>();
		assert_ok!(Marketplace::<T>::create_order(RawOrigin::Signed(alice.clone()).into(),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK)));
		assert_ok!(Balances::<T>::set_balance(
				RawOrigin::Root.into(),
		lookup_of_account::<T>(bob.clone()),
		1000 * W3G,
		0,
		));
	}: _(RawOrigin::Signed(bob),Asset::NonFungibleToken(1, 2))

	place_bid {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		create_non_fungible_token::<T>();
		assert_ok!(Marketplace::<T>::create_order(RawOrigin::Signed(alice.clone()).into(),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK)));
		assert_ok!(Balances::<T>::set_balance(
				RawOrigin::Root.into(),
		lookup_of_account::<T>(bob.clone()),
		1000 * W3G,
		0,
		));
	}: _(RawOrigin::Signed(bob),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK))

	cancel_bid {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		create_non_fungible_token::<T>();
		assert_ok!(Marketplace::<T>::create_order(RawOrigin::Signed(alice.clone()).into(),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK)));
		assert_ok!(Balances::<T>::set_balance(
				RawOrigin::Root.into(),
		lookup_of_account::<T>(bob.clone()),
		1000 * W3G,
		0,
		));
		assert_ok!(Marketplace::<T>::place_bid(RawOrigin::Signed(bob.clone()).into(),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK)));
	}: _(RawOrigin::Signed(bob),Asset::NonFungibleToken(1, 2))

	accept_bid {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		create_non_fungible_token::<T>();
		assert_ok!(Marketplace::<T>::create_order(RawOrigin::Signed(alice.clone()).into(),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK)));
		assert_ok!(Balances::<T>::set_balance(
				RawOrigin::Root.into(),
		lookup_of_account::<T>(bob.clone()),
		1000 * W3G,
		0,
		));
		assert_ok!(Marketplace::<T>::place_bid(RawOrigin::Signed(bob.clone()).into(),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK)));
	}: _(RawOrigin::Signed(alice),Asset::NonFungibleToken(1, 2))

	impl_benchmark_test_suite!(Marketplace, crate::mock::new_test_ext(), crate::mock::Test);
}
