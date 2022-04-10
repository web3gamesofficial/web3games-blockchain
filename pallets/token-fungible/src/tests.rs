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
use frame_support::assert_ok;

#[test]
fn test_create_token_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));
	})
}

#[test]
fn test_mint_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(1), 1, 2, 100));
		assert_eq!(TokenFungible::balance_of(1, 2), 100);
	})
}

#[test]
fn test_approve_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(1), 1, 1, 100));
		assert_eq!(TokenFungible::balance_of(1, 1), 100);
		assert_ok!(TokenFungible::approve(Origin::signed(1), 1, 2, 50));
		assert_eq!(TokenFungible::allowances(1, (1, 2)), 50);
	})
}

#[test]
fn test_transfer_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(1), 1, 1, 100));
		assert_eq!(TokenFungible::balance_of(1, 1), 100);
		assert_ok!(TokenFungible::transfer(Origin::signed(1), 1, 2, 10));
		assert_eq!(TokenFungible::balance_of(1, 1), 90);
		assert_eq!(TokenFungible::balance_of(1, 2), 10);
	})
}

#[test]
fn test_transfer_from_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(1), 1, 1, 100));
		assert_eq!(TokenFungible::balance_of(1, 1), 100);
		assert_ok!(TokenFungible::approve(Origin::signed(1).clone(), 1, 2, 50));
		assert_eq!(TokenFungible::allowances(1, (1, 2)), 50);

		assert_ok!(TokenFungible::transfer_from(Origin::signed(2), 1, 1, 3, 20));
		assert_eq!(TokenFungible::allowances(1, (1, 2)), 30);
		assert_eq!(TokenFungible::balance_of(1, 3), 20);
	})
}

#[test]
fn test_burn_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));
		assert_ok!(TokenFungible::mint(Origin::signed(1), 1, 1, 100));
		assert_eq!(TokenFungible::balance_of(1, 1), 100);
		assert_ok!(TokenFungible::burn(Origin::signed(1), 1, 50));
		assert_eq!(TokenFungible::balance_of(1, 1), 50);
	})
}
