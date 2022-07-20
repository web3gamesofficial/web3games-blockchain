// This file is part of Web3Games.

// Copyright (C) 2021-2022 Web3Games https://web3games.org
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;

#[test]
fn create_token_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));
		assert_eq!(TokenFungible::exists(1), true);
		assert_eq!(TokenFungible::token_name(1), b"W3G".to_vec());
		assert_eq!(TokenFungible::token_symbol(1), b"W3G".to_vec());
		assert_eq!(TokenFungible::token_decimals(1), 18);
		assert_eq!(TokenFungible::total_supply(1), 0);
	})
}

#[test]
fn create_token_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));
		assert_noop!(
			TokenFungible::create_token(
				Origin::signed(ALICE),
				1,
				b"W3G".to_vec(),
				b"W3G".to_vec(),
				18
			),
			Error::<Test>::InvalidId
		);
	})
}

#[test]
fn mint_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(ALICE), 1, ALICE, 100));
		assert_eq!(TokenFungible::balance_of(1, ALICE), 100);
		assert_eq!(TokenFungible::total_supply(1), 100);
		assert_ok!(TokenFungible::mint(Origin::signed(ALICE), 1, ALICE, 100));
		assert_eq!(TokenFungible::balance_of(1, ALICE), 200);
		assert_eq!(TokenFungible::total_supply(1), 200);
	})
}

#[test]
fn mint_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			TokenFungible::mint(Origin::signed(ALICE), 1, ALICE, 100),
			Error::<Test>::InvalidId
		);
		assert_ok!(TokenFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));
		assert_noop!(
			TokenFungible::mint(Origin::signed(BOB), 1, ALICE, 100),
			Error::<Test>::NoPermission
		);
	})
}

#[test]
fn burn_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(ALICE), 1, ALICE, 100));
		assert_ok!(TokenFungible::mint(Origin::signed(ALICE), 1, BOB, 100));
		assert_eq!(TokenFungible::balance_of(1, ALICE), 100);
		assert_eq!(TokenFungible::balance_of(1, BOB), 100);
		assert_eq!(TokenFungible::total_supply(1), 200);

		assert_ok!(TokenFungible::burn(Origin::signed(ALICE), 1, 50));
		assert_ok!(TokenFungible::burn(Origin::signed(ALICE), 1, 0));
		assert_eq!(TokenFungible::balance_of(1, ALICE), 50);
		assert_eq!(TokenFungible::total_supply(1), 150);

		assert_ok!(TokenFungible::burn(Origin::signed(BOB), 1, 50));
		assert_eq!(TokenFungible::balance_of(1, BOB), 50);
		assert_eq!(TokenFungible::total_supply(1), 100);
	})
}

#[test]
fn burn_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(TokenFungible::burn(Origin::signed(ALICE), 1, 50), Error::<Test>::Unknown);

		assert_ok!(TokenFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));

		assert_noop!(TokenFungible::burn(Origin::signed(BOB), 1, 100), Error::<Test>::NumOverflow);
	})
}

#[test]
fn transfer_should_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(ALICE), 1, ALICE, 100));
		assert_eq!(TokenFungible::balance_of(1, ALICE), 100);

		assert_ok!(TokenFungible::transfer(Origin::signed(ALICE), 1, BOB, 10));
		assert_eq!(TokenFungible::balance_of(1, ALICE), 90);
		assert_eq!(TokenFungible::balance_of(1, BOB), 10);

		assert_eq!(TokenFungible::total_supply(1), 100);
	})
}

#[test]
fn approve_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(ALICE), 1, ALICE, 100));
		assert_eq!(TokenFungible::balance_of(1, ALICE), 100);
		assert_ok!(TokenFungible::approve(Origin::signed(ALICE), 1, BOB, 50));
		assert_eq!(TokenFungible::allowances(1, (ALICE, BOB)), 50);

		assert_eq!(TokenFungible::total_supply(1), 100);
	})
}

#[test]
fn approve_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			TokenFungible::approve(Origin::signed(ALICE), 1, BOB, 50),
			Error::<Test>::InsufficientAuthorizedTokens
		);
		assert_ok!(TokenFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(ALICE), 1, ALICE, 100));
		assert_noop!(
			TokenFungible::approve(Origin::signed(ALICE), 1, BOB, 200),
			Error::<Test>::InsufficientAuthorizedTokens
		);
		assert_noop!(
			TokenFungible::approve(Origin::signed(ALICE), 1, ALICE, 100),
			Error::<Test>::ApproveToCurrentOwner
		);

		assert_eq!(TokenFungible::total_supply(1), 100);
	})
}

#[test]
fn transfer_from_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(ALICE), 1, ALICE, 100));
		assert_eq!(TokenFungible::balance_of(1, ALICE), 100);
		assert_ok!(TokenFungible::approve(Origin::signed(ALICE), 1, BOB, 50));
		assert_eq!(TokenFungible::allowances(1, (ALICE, BOB)), 50);

		assert_ok!(TokenFungible::transfer_from(Origin::signed(BOB), 1, ALICE, CHARLIE, 20));
		assert_eq!(TokenFungible::allowances(1, (ALICE, BOB)), 30);
		assert_eq!(TokenFungible::balance_of(1, CHARLIE), 20);
		assert_eq!(TokenFungible::balance_of(1, ALICE), 80);

		assert_eq!(TokenFungible::total_supply(1), 100);
	})
}

#[test]
fn transfer_from_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			TokenFungible::transfer_from(Origin::signed(BOB), 1, ALICE, CHARLIE, 20),
			Error::<Test>::InsufficientAuthorizedTokens
		);
		assert_noop!(
			TokenFungible::transfer_from(Origin::signed(BOB), 1, ALICE, BOB, 20),
			Error::<Test>::ConfuseBehavior
		);

		assert_ok!(TokenFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(ALICE), 1, ALICE, 100));
		assert_eq!(TokenFungible::balance_of(1, ALICE), 100);

		assert_ok!(TokenFungible::approve(Origin::signed(ALICE), 1, BOB, 50));
		assert_eq!(TokenFungible::allowances(1, (ALICE, BOB)), 50);

		assert_noop!(
			TokenFungible::transfer_from(Origin::signed(BOB), 1, ALICE, BOB, 20),
			Error::<Test>::ConfuseBehavior
		);

		assert_noop!(
			TokenFungible::transfer_from(Origin::signed(BOB), 1, CHARLIE, ALICE, 20),
			Error::<Test>::InsufficientAuthorizedTokens
		);

		assert_eq!(TokenFungible::total_supply(1), 100);
	})
}
