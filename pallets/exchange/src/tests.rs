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
fn test_create_pool_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			2,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));
		assert_ok!(Exchange::create_pool(Origin::signed(1), 1, 2));
		assert_eq!(Exchange::get_pool((1, 2)), 0);
	})
}

#[test]
fn test_add_liquidity_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			2,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));

		assert_ok!(TokenFungible::mint(Origin::signed(1), 1, 1, 1000000u128,));

		assert_ok!(TokenFungible::mint(Origin::signed(1), 2, 1, 1000000u128,));

		assert_ok!(Exchange::create_pool(Origin::signed(1), 1, 2));
		assert_eq!(Exchange::get_pool((1, 2)), 0);

		assert_ok!(Exchange::add_liquidity(
			Origin::signed(1),
			0,
			10000u128,
			10000u128,
			0u128,
			0u128,
			1
		));
		assert_eq!(TokenFungible::balance_of(1, &Exchange::account_id()), 10000);
		assert_eq!(TokenFungible::balance_of(2, &Exchange::account_id()), 10000);
	})
}

#[test]
fn test_swap_exact_tokens_for_tokens_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			2,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));

		assert_ok!(TokenFungible::mint(Origin::signed(1), 1, 1, 1000000u128,));

		assert_ok!(TokenFungible::mint(Origin::signed(1), 2, 1, 1000000u128,));

		assert_ok!(Exchange::create_pool(Origin::signed(1), 1, 2));
		assert_eq!(Exchange::get_pool((1, 2)), 0);

		assert_ok!(Exchange::add_liquidity(
			Origin::signed(1),
			0,
			10000u128,
			10000u128,
			0u128,
			0u128,
			1
		));
		assert_eq!(TokenFungible::balance_of(1, &Exchange::account_id()), 10000);
		assert_eq!(TokenFungible::balance_of(2, &Exchange::account_id()), 10000);

		assert_ok!(Exchange::swap_exact_tokens_for_tokens(
			Origin::signed(1),
			0,
			1000u128,
			900u128,
			vec![1, 2],
			1
		));
	})
}

#[test]
fn test_swap_tokens_for_exact_tokens_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));
		assert_ok!(TokenFungible::create_token(
			Origin::signed(1),
			2,
			vec![0u8, 10],
			vec![0u8, 10],
			18
		));

		assert_ok!(TokenFungible::mint(Origin::signed(1), 1, 1, 1000000u128,));

		assert_ok!(TokenFungible::mint(Origin::signed(1), 2, 1, 1000000u128,));

		assert_ok!(Exchange::create_pool(Origin::signed(1), 1, 2));
		assert_eq!(Exchange::get_pool((1, 2)), 0);

		assert_ok!(Exchange::add_liquidity(
			Origin::signed(1),
			0,
			10000u128,
			10000u128,
			0u128,
			0u128,
			1
		));
		assert_eq!(TokenFungible::balance_of(1, &Exchange::account_id()), 10000);
		assert_eq!(TokenFungible::balance_of(2, &Exchange::account_id()), 10000);

		assert_ok!(Exchange::swap_tokens_for_exact_tokens(
			Origin::signed(1),
			0,
			1000u128,
			1100u128,
			vec![1, 2],
			1
		));
	})
}
