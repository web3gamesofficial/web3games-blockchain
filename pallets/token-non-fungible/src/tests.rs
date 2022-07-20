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
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		));
		assert_eq!(TokenNonFungible::exists(1), true);
		assert_eq!(TokenNonFungible::token_name(1), b"W3G".to_vec());
		assert_eq!(TokenNonFungible::token_symbol(1), b"W3G".to_vec());
		assert_eq!(TokenNonFungible::token_uri(1,0), b"https://web3games.com/0");
		assert_eq!(TokenNonFungible::total_supply(1), 0);
	})
}

#[test]
fn create_token_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		));
		assert_noop!(
			TokenNonFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		),
			Error::<Test>::InvalidId
		);
	})
}

#[test]
fn mint_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 1));
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, BOB, 100));
		assert_eq!(TokenNonFungible::balance_of(1, ALICE), 1);
		assert_eq!(TokenNonFungible::owner_of(1, 1), Some(ALICE));
		assert_eq!(TokenNonFungible::total_supply(1), 2);
		assert_eq!(TokenNonFungible::token_by_index(1,1), 100);


		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 2));
		assert_eq!(TokenNonFungible::balance_of(1, ALICE), 2);
		assert_eq!(TokenNonFungible::owner_of(1, 2), Some(ALICE));
		assert_eq!(TokenNonFungible::token_by_index(1,2), 2);
		assert_eq!(TokenNonFungible::total_supply(1), 3);
		assert_eq!(TokenNonFungible::token_of_owner_by_index(1,ALICE,0), 1);
		assert_eq!(TokenNonFungible::token_of_owner_by_index(1,ALICE,1), 2);

	})
}

#[test]
fn mint_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 1),Error::<Test>::InvalidId);
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		));
		assert_noop!(TokenNonFungible::mint(Origin::signed(BOB), 1, ALICE, 100),Error::<Test>::NoPermission);
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 2));
		assert_noop!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 2),Error::<Test>::TokenAlreadyMinted);
	})
}

#[test]
fn burn_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
		Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 1));
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, BOB, 2));
		assert_eq!(TokenNonFungible::balance_of(1, ALICE), 1);
		assert_eq!(TokenNonFungible::balance_of(1, BOB), 1);
		assert_eq!(TokenNonFungible::total_supply(1), 2);

		assert_ok!(TokenNonFungible::burn(Origin::signed(ALICE), 1, 1));
		assert_eq!(TokenNonFungible::balance_of(1, ALICE), 0);
		assert_eq!(TokenNonFungible::owner_of(1, 1), None);
		assert_eq!(TokenNonFungible::total_supply(1), 1);

		assert_ok!(TokenNonFungible::burn(Origin::signed(BOB), 1, 2));
		assert_eq!(TokenNonFungible::balance_of(1, BOB), 0);
		assert_eq!(TokenNonFungible::owner_of(1, 2), None);
		assert_eq!(TokenNonFungible::total_supply(1), 0);

	})
}

#[test]
fn burn_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(TokenNonFungible::burn(Origin::signed(ALICE), 1, 0),Error::<Test>::NotFound);

		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		));

		assert_noop!(TokenNonFungible::burn(Origin::signed(BOB), 1, 0),Error::<Test>::NotFound);

	})
}

#[test]
fn approve_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 0));

		assert_ok!(TokenNonFungible::approve(Origin::signed(ALICE), 1, BOB, 0));
	})
}

#[test]
fn approve_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(TokenNonFungible::approve(Origin::signed(ALICE), 1, BOB, 50),Error::<Test>::NotFound);
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 0));
		assert_noop!(TokenNonFungible::approve(Origin::signed(ALICE), 1, BOB, 1),Error::<Test>::NotFound);
		assert_noop!(TokenNonFungible::approve(Origin::signed(ALICE), 1, ALICE, 0),Error::<Test>::ApproveToCurrentOwner);

		assert_eq!(TokenNonFungible::total_supply(1), 1);
	})
}

#[test]
fn set_approve_for_all_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 1));
		assert_ok!(TokenNonFungible::set_approve_for_all(Origin::signed(ALICE), 1, BOB, true));
	})
}

#[test]
fn transfer_from_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 0));
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 1));
		assert_eq!(TokenNonFungible::balance_of(1, ALICE), 2);

		assert_ok!(TokenNonFungible::approve(Origin::signed(ALICE), 1, BOB, 0));
		assert_ok!(TokenNonFungible::set_approve_for_all(Origin::signed(ALICE), 1, CHARLIE, true));

		assert_ok!(TokenNonFungible::transfer_from(Origin::signed(BOB), 1, ALICE, CHARLIE, 0));
		assert_eq!(TokenNonFungible::balance_of(1, CHARLIE), 1);
		assert_eq!(TokenNonFungible::balance_of(1, ALICE), 1);

		assert_ok!(TokenNonFungible::transfer_from(Origin::signed(CHARLIE), 1, ALICE, CHARLIE, 1));
		assert_eq!(TokenNonFungible::balance_of(1, CHARLIE), 2);
		assert_eq!(TokenNonFungible::balance_of(1, ALICE), 0);
		assert_eq!(TokenNonFungible::total_supply(1), 2);
	})
}

#[test]
fn transfer_from_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(TokenNonFungible::transfer_from(Origin::signed(BOB), 1, ALICE, CHARLIE, 0),Error::<Test>::TokenNonExistent);

		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(ALICE),
			1,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			b"https://web3games.com/".to_vec(),
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 0));
		assert_ok!(TokenNonFungible::mint(Origin::signed(ALICE), 1, ALICE, 1));
		assert_eq!(TokenNonFungible::balance_of(1, ALICE), 2);

		assert_ok!(TokenNonFungible::approve(Origin::signed(ALICE), 1, BOB, 0));
		assert_noop!(TokenNonFungible::transfer_from(Origin::signed(BOB), 1, ALICE, BOB, 1),Error::<Test>::NotOwnerOrApproved);

		assert_noop!(TokenNonFungible::transfer_from(Origin::signed(BOB), 1, BOB, ALICE, 1),Error::<Test>::NotOwnerOrApproved);

		assert_eq!(TokenNonFungible::total_supply(1), 2);
	})
}
