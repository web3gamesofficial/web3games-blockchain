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

#[test]
fn create_token_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));
		assert_eq!(TokenMulti::exists(1), true);
		assert_eq!(TokenMulti::uri(1, 1), b"https://web3games.com/1".to_vec());
		assert_eq!(TokenMulti::owner_or_approved(1, &ALICE, &ALICE), true);
	})
}

#[test]
fn create_token_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));
		assert_noop!(
			TokenMulti::create_token(Origin::signed(ALICE), 1, b"https://web3games.com/".to_vec()),
			Error::<Test>::InvalidId
		);
	})
}

#[test]
fn mint_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));

		assert_ok!(TokenMulti::mint(Origin::signed(ALICE), 1, ALICE, 1, 100));
		assert_ok!(TokenMulti::mint(Origin::signed(ALICE), 1, BOB, 1, 100));

		assert_eq!(TokenMulti::balance_of(1, (1, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (1, BOB)), 100);
	})
}

#[test]
fn mint_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			TokenMulti::mint(Origin::signed(ALICE), 1, ALICE, 1, 100),
			Error::<Test>::InvalidId
		);
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));

		assert_ok!(TokenMulti::mint(Origin::signed(ALICE), 1, ALICE, 1, 100));
		assert_noop!(
			TokenMulti::mint(Origin::signed(BOB), 1, BOB, 1, 100),
			Error::<Test>::NoPermission
		);
	})
}

#[test]
fn batch_mint_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));
		assert_ok!(TokenMulti::mint_batch(
			Origin::signed(ALICE),
			1,
			ALICE,
			vec![1, 2, 3, 4, 5],
			vec![100u128; 5]
		));
		assert_eq!(TokenMulti::balance_of(1, (1, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (2, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (3, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (4, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (5, ALICE)), 100);
	})
}

#[test]
fn batch_mint_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			TokenMulti::mint_batch(
				Origin::signed(ALICE),
				1,
				ALICE,
				vec![1, 2, 3, 4, 5],
				vec![100u128; 5]
			),
			Error::<Test>::InvalidId
		);
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));
		assert_noop!(
			TokenMulti::mint_batch(
				Origin::signed(BOB),
				1,
				ALICE,
				vec![1, 2, 3, 4, 5],
				vec![100u128; 5]
			),
			Error::<Test>::NoPermission
		);
		assert_noop!(
			TokenMulti::mint_batch(
				Origin::signed(ALICE),
				1,
				ALICE,
				vec![1, 2, 3, 4, 5],
				vec![100u128; 4]
			),
			Error::<Test>::LengthMismatch
		);
	})
}

#[test]
fn burn_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));
		assert_ok!(TokenMulti::mint(Origin::signed(ALICE), 1, ALICE, 1, 100));
		assert_ok!(TokenMulti::burn(Origin::signed(ALICE), 1, 1, 5));
		assert_eq!(TokenMulti::balance_of(1, (1, ALICE)), 95);
	})
}

#[test]
fn burn_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(TokenMulti::burn(Origin::signed(ALICE), 1, 1, 5), Error::<Test>::Unknown);
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));
		assert_noop!(TokenMulti::burn(Origin::signed(BOB), 1, 1, 5), Error::<Test>::NumOverflow);
	})
}

#[test]
fn batch_burn_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));
		assert_ok!(TokenMulti::mint_batch(
			Origin::signed(ALICE),
			1,
			ALICE,
			vec![1, 2, 3, 4, 5],
			vec![100u128; 5]
		));
		assert_eq!(TokenMulti::balance_of(1, (1, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (2, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (3, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (4, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (5, ALICE)), 100);
		assert_ok!(TokenMulti::burn_batch(
			Origin::signed(1),
			1,
			vec![1, 2, 3, 4, 5],
			vec![5u128; 5]
		));
		assert_eq!(TokenMulti::balance_of(1, (1, ALICE)), 95);
		assert_eq!(TokenMulti::balance_of(1, (2, ALICE)), 95);
		assert_eq!(TokenMulti::balance_of(1, (3, ALICE)), 95);
		assert_eq!(TokenMulti::balance_of(1, (4, ALICE)), 95);
		assert_eq!(TokenMulti::balance_of(1, (5, ALICE)), 95);
	})
}

#[test]
fn set_approval_for_all_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));
		assert_ok!(TokenMulti::mint(Origin::signed(ALICE), 1, ALICE, 1, 100));
		assert_ok!(TokenMulti::set_approval_for_all(Origin::signed(ALICE), 1, BOB, true));
		assert_eq!(TokenMulti::owner_or_approved(1, &BOB, &ALICE), true);
	})
}

#[test]
fn transfer_from_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			2,
			b"https://web3games.com/".to_vec()
		));
		assert_ok!(TokenMulti::mint(Origin::signed(ALICE), 1, ALICE, 1, 100));
		assert_ok!(TokenMulti::mint(Origin::signed(ALICE), 1, ALICE, 2, 100));
		assert_eq!(TokenMulti::balance_of(1, (1, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (2, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (1, BOB)), 0);
		assert_eq!(TokenMulti::balance_of(1, (2, BOB)), 0);

		assert_ok!(TokenMulti::set_approval_for_all(Origin::signed(ALICE), 1, BOB, true));

		assert_ok!(TokenMulti::transfer_from(Origin::signed(ALICE), 1, ALICE, BOB, 1, 50));
		assert_eq!(TokenMulti::balance_of(1, (1, ALICE)), 50);
		assert_eq!(TokenMulti::balance_of(1, (1, BOB)), 50);

		assert_ok!(TokenMulti::transfer_from(Origin::signed(BOB), 1, ALICE, BOB, 1, 20));
		assert_eq!(TokenMulti::balance_of(1, (1, ALICE)), 30);
		assert_eq!(TokenMulti::balance_of(1, (1, BOB)), 70);

		assert_ok!(TokenMulti::transfer_from(Origin::signed(BOB), 1, ALICE, BOB, 2, 50));
		assert_eq!(TokenMulti::balance_of(1, (2, ALICE)), 50);
		assert_eq!(TokenMulti::balance_of(1, (2, BOB)), 50);
	})
}

#[test]
fn transfer_from_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			2,
			b"https://web3games.com/".to_vec()
		));
		assert_ok!(TokenMulti::mint(Origin::signed(ALICE), 1, ALICE, 1, 100));
		assert_ok!(TokenMulti::mint(Origin::signed(ALICE), 1, ALICE, 2, 100));

		assert_noop!(
			TokenMulti::transfer_from(Origin::signed(BOB), 1, ALICE, BOB, 1, 50),
			Error::<Test>::NotOwnerOrApproved
		);
		assert_noop!(
			TokenMulti::transfer_from(Origin::signed(ALICE), 1, ALICE, BOB, 1, 101),
			Error::<Test>::InsufficientTokens
		);
	})
}

#[test]
fn batch_transfer_from_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenMulti::create_token(
			Origin::signed(ALICE),
			1,
			b"https://web3games.com/".to_vec()
		));
		assert_ok!(TokenMulti::mint_batch(
			Origin::signed(ALICE),
			1,
			ALICE,
			vec![1, 2, 3],
			vec![100u128; 3]
		));
		assert_eq!(TokenMulti::balance_of(1, (1, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (2, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (3, ALICE)), 100);
		assert_eq!(TokenMulti::balance_of(1, (1, BOB)), 0);
		assert_eq!(TokenMulti::balance_of(1, (2, BOB)), 0);
		assert_eq!(TokenMulti::balance_of(1, (3, BOB)), 0);

		assert_ok!(TokenMulti::batch_transfer_from(
			Origin::signed(ALICE),
			1,
			ALICE,
			BOB,
			vec![1, 2, 3],
			vec![50u128; 3]
		));

		assert_eq!(TokenMulti::balance_of(1, (1, ALICE)), 50);
		assert_eq!(TokenMulti::balance_of(1, (2, ALICE)), 50);
		assert_eq!(TokenMulti::balance_of(1, (3, ALICE)), 50);
		assert_eq!(TokenMulti::balance_of(1, (1, BOB)), 50);
		assert_eq!(TokenMulti::balance_of(1, (2, BOB)), 50);
		assert_eq!(TokenMulti::balance_of(1, (3, BOB)), 50);
	})
}
