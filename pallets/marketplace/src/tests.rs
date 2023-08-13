#![cfg(test)]

use super::*;
use crate::mock::*;
use frame_support::assert_ok;

const ALICE: u64 = 1;
const BOB: u64 = 2;

const W3G: u128 = 1_000_000_000_000_000_000;
const BLOCK: u64 = 1;

fn create_non_fungible_token() {
	assert_ok!(TokenNonFungible::create_token(
		Origin::signed(ALICE),
		1,
		b"W3G".to_vec(),
		b"W3G".to_vec(),
		b"https://web3games.com/".to_vec(),
	));
	assert_eq!(TokenNonFungible::exists(1), true);

	//mint 2 to ALICE
	assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 2));
	assert_eq!(TokenNonFungible::owner_of(1, 2), Some(ALICE));
}

#[test]
fn create_order_should_work() {
	new_test_ext().execute_with(|| {
		create_non_fungible_token();
		assert_ok!(Marketplace::create_order(
			Origin::signed(ALICE),
			Asset::NonFungibleToken(1, 2),
			100 * W3G,
			100 * BLOCK
		));
		assert_eq!(
			Marketplace::orders(Asset::NonFungibleToken(1, 2)),
			Some(Order {
				creater: ALICE,
				price: 100 * W3G,
				start: 1 * BLOCK,
				duration: 100 * BLOCK
			})
		);

		assert_ok!(
			Marketplace::cancel_order(Origin::signed(ALICE), Asset::NonFungibleToken(1, 2),)
		);
	})
}

#[test]
fn execute_order_should_work() {
	new_test_ext().execute_with(|| {
		create_non_fungible_token();
		assert_ok!(Marketplace::create_order(
			Origin::signed(ALICE),
			Asset::NonFungibleToken(1, 2),
			100 * W3G,
			100 * BLOCK
		));

		assert_ok!(Marketplace::execute_order(Origin::signed(BOB), Asset::NonFungibleToken(1, 2),));
	})
}
