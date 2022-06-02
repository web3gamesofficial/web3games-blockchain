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

use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn test_create_token_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
	})
}

#[test]
fn test_mint_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(1), 1, 2, 1));
	})
}

#[test]
fn test_approve_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(1), 1, 1, 1));
		assert_ok!(TokenNonFungible::approve(Origin::signed(1), 1, 2, 1));
	})
}

#[test]
fn test_set_approve_for_all_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(1), 1, 1, 1));
		assert_ok!(TokenNonFungible::set_approve_for_all(Origin::signed(1), 1, 2, true));
	})
}

#[test]
fn test_transfer_from_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(1), 1, 1, 1));
		assert_ok!(TokenNonFungible::transfer_from(Origin::signed(1), 1, 1, 2, 1));
	})
}

#[test]
fn test_burn_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(1), 1, 1, 1));
		assert_ok!(TokenNonFungible::burn(Origin::signed(1), 1, 1));
	})
}
