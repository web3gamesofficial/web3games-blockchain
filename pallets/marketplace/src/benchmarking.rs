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

// Helper to assert last event.
fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

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
	}: _(RawOrigin::Root,alice.clone())
	verify {
		assert_eq!(Admin::<T>::get(), Some(alice));
	}

	set_service_fee_point {
		let alice: T::AccountId = account("alice", 0, 0);
		assert_ok!(Marketplace::<T>::set_admin(RawOrigin::Root.into(),alice.clone()));
	}: _(RawOrigin::Signed(alice),10u8)
	verify {
		assert_eq!(Point::<T>::get(), 10);
	}

	create_order {
		let alice: T::AccountId = account("alice", 0, 0);
		create_non_fungible_token::<T>();
	}: _(RawOrigin::Signed(alice.clone()),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK))
	verify {
		assert!(Orders::<T>::contains_key(Asset::NonFungibleToken(1, 2)));
		assert_last_event::<T>(Event::<T>::OrderCreated(alice.clone(), Asset::NonFungibleToken(1, 2), Order { creater: alice, price: BalanceOf::<T>::unique_saturated_from(100 * W3G), start: T::BlockNumber::from(1_u32), duration: T::BlockNumber::from(100 * BLOCK) }).into());
	}

	cancel_order {
		let alice: T::AccountId = account("alice", 0, 0);
		create_non_fungible_token::<T>();
		assert_ok!(Marketplace::<T>::create_order(RawOrigin::Signed(alice.clone()).into(),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK)));
	}: _(RawOrigin::Signed(alice.clone()),Asset::NonFungibleToken(1, 2))
	verify {
		assert!(!Orders::<T>::contains_key(Asset::NonFungibleToken(1, 2)));
		assert!(!Bids::<T>::contains_key(Asset::NonFungibleToken(1, 2)));
		assert_last_event::<T>(Event::<T>::OrderCancelled(alice, Asset::NonFungibleToken(1, 2)).into());
	}

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
	}: _(RawOrigin::Signed(bob.clone()),Asset::NonFungibleToken(1, 2))
	verify {
		assert!(!Bids::<T>::contains_key(Asset::NonFungibleToken(1, 2)));
		assert!(!Orders::<T>::contains_key(Asset::NonFungibleToken(1, 2)));
		assert_last_event::<T>(Event::<T>::OrderExecuted(bob, Asset::NonFungibleToken(1, 2), Order { creater: alice, price: BalanceOf::<T>::unique_saturated_from(100 * W3G), start: T::BlockNumber::from(0_u32), duration: T::BlockNumber::from(100 * BLOCK) }).into());
	}


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
	}: _(RawOrigin::Signed(bob.clone()),Asset::NonFungibleToken(1, 2),BalanceOf::<T>::unique_saturated_from(100 * W3G),T::BlockNumber::from(100 * BLOCK))
	verify {
		assert_eq!(Bids::<T>::get(Asset::NonFungibleToken(1, 2)), Some(Order { creater: bob.clone(), price: BalanceOf::<T>::unique_saturated_from(100 * W3G), start: T::BlockNumber::from(1_u32), duration: T::BlockNumber::from(100 * BLOCK) }));
		assert_last_event::<T>(Event::<T>::BidCreated(bob.clone(), Asset::NonFungibleToken(1, 2), Order { creater: bob, price: BalanceOf::<T>::unique_saturated_from(100 * W3G), start: T::BlockNumber::from(1_u32), duration: T::BlockNumber::from(100 * BLOCK) }).into());
	}

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
	}: _(RawOrigin::Signed(bob.clone()),Asset::NonFungibleToken(1, 2))
	verify {
		assert!(!Bids::<T>::contains_key(Asset::NonFungibleToken(1, 2)));
		assert_last_event::<T>(Event::<T>::BidCancelled(bob, Asset::NonFungibleToken(1, 2)).into());
	}

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
	}: _(RawOrigin::Signed(alice.clone()),Asset::NonFungibleToken(1, 2))
	verify {
		assert!(!Bids::<T>::contains_key(Asset::NonFungibleToken(1, 2)));
		assert!(!Orders::<T>::contains_key(Asset::NonFungibleToken(1, 2)));
		assert_last_event::<T>(Event::<T>::BidAccepted(alice, Asset::NonFungibleToken(1, 2), Order { creater: bob, price: BalanceOf::<T>::unique_saturated_from(100 * W3G), start: T::BlockNumber::from(0_u32), duration: T::BlockNumber::from(100 * BLOCK) }).into());
	}

	impl_benchmark_test_suite!(Marketplace, crate::mock::new_test_ext(), crate::mock::Test);
}
