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
pub use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;

const TOKENA: u128 = 1;
const TOKENB: u128 = 2;
const TOKENC: u128 = 3;

const USDT: u128 = 1_000_000;
const W3G: u128 = 1_000_000_000_000_000_000;

fn create_tokens() {
	assert_ok!(TokenFungible::create_token(
		Origin::signed(ALICE),
		TOKENA,
		b"W3G1".to_vec(),
		b"W3G1".to_vec(),
		18
	));
	assert_ok!(TokenFungible::create_token(
		Origin::signed(ALICE),
		TOKENB,
		b"W3G2".to_vec(),
		b"W3G2".to_vec(),
		6
	));
}
fn set_balance() {
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), TOKENA, ALICE, 100 * W3G));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), TOKENA, BOB, 100 * W3G));

	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), TOKENB, ALICE, 100 * USDT));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), TOKENB, BOB, 100 * USDT));
}

#[test]
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		set_balance();
		assert_ok!(Launchpad::create_pool(
			Origin::signed(ALICE),
			1,
			10,
			TOKENA,
			TOKENB,
			10 * W3G,
			1 * USDT
		));

		let escrow_account = Launchpad::escrow_account_id(0);
		assert_eq!(
			TokenFungible::balance_of(TOKENA, ALICE),
			100 * W3G - 10 * W3G
		);
		assert_eq!(TokenFungible::balance_of(TOKENA, escrow_account), 10 * W3G);
		assert_eq!(
			Launchpad::pools(0).unwrap(),
			Pool {
				escrow_account,
				sale_start: 1,
				sale_end: 1 + 10,
				sale_token_id: TOKENA,
				buy_token_id: TOKENB,
				token_price: 1 * USDT,
				total_sale_amount: 10 * W3G,
				raise_amount: 10 * W3G,
			}
		);
		assert_eq!(Launchpad::next_pool_id(), 1);
	})
}

#[test]
fn buy_token_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		set_balance();
		assert_ok!(Launchpad::create_pool(
			Origin::signed(ALICE),
			1,
			10,
			TOKENA,
			TOKENB,
			10 * W3G,
			1 * USDT
		));

		assert_ok!(Launchpad::buy_token(Origin::signed(BOB), 0, 1));

		let escrow_account = Launchpad::escrow_account_id(0);
		assert_eq!(
			TokenFungible::balance_of(TOKENB, BOB),
			100 * USDT - 1 * USDT
		);
		assert_eq!(TokenFungible::balance_of(TOKENB, escrow_account), 1 * USDT);
		assert_eq!(
			Launchpad::account_pool_id_locked((BOB, 0)).unwrap(),
			ClaimInfo { balance: 1 * W3G, is_claimed: false }
		);
		assert_eq!(Launchpad::pools(0).unwrap().raise_amount, 10 * W3G - 1 * W3G);

		assert_noop!(
			Launchpad::buy_token(Origin::signed(BOB), 1, 1),
			Error::<Test>::PoolNotFound
		);
		assert_ok!(Launchpad::buy_token(Origin::signed(BOB), 0, 1));

		assert_eq!(
			TokenFungible::balance_of(TOKENB, BOB),
			100 * USDT - 1 * USDT - 1 * USDT
		);
		assert_eq!(TokenFungible::balance_of(TOKENB, escrow_account), 2 * USDT);
		assert_eq!(Launchpad::pools(0).unwrap().raise_amount, 10 * W3G - 1 * W3G - 1 * W3G);

		run_to_block(12);
		assert_noop!(
			Launchpad::buy_token(Origin::signed(BOB), 0, 1),
			Error::<Test>::OutOfSaleTime
		);
	})
}

// #[test]
// fn claim_should_work() {
// 	new_test_ext().execute_with(|| {
// 		create_tokens();
// 		set_balance();
// 		assert_ok!(Launchpad::set_admin(Origin::root(), ALICE));
// 		assert_ok!(Launchpad::create_pool(
// 			Origin::signed(ALICE),
// 			1,
// 			10,
// 			TOKENA,
// 			TOKENB,
// 			10_000_000_000_000_000_000,
// 		));
// 		assert_ok!(Launchpad::staking(Origin::signed(BOB), 0, 10_000_000_000_000_000_000));
// 		assert_ok!(Launchpad::staking(Origin::signed(CHARLIE), 0, 10_000_000_000_000_000_000));
//
// 		assert_noop!(Launchpad::claim(Origin::signed(BOB), 1), Error::<Test>::PoolNotFound);
// 		assert_noop!(Launchpad::claim(Origin::signed(BOB), 0), Error::<Test>::ClaimNotStart);
//
// 		run_to_block(12);
// 		assert_noop!(Launchpad::claim(Origin::signed(ALICE), 0), Error::<Test>::NotStaking);
//
// 		assert_ok!(Launchpad::claim(Origin::signed(BOB), 0));
//
// 		let escrow_account = Launchpad::escrow_account_id(0);
// 		assert_eq!(TokenFungible::balance_of(TOKENA, BOB), INITIAL_BALANCE);
// 		assert_eq!(TokenFungible::balance_of(TOKENB, BOB), 5_000_000_000_000_000_000);
// 		assert_eq!(TokenFungible::balance_of(TOKENA, escrow_account), 10_000_000_000_000_000_000);
// 		assert_eq!(TokenFungible::balance_of(TOKENB, escrow_account), 5_000_000_000_000_000_000);
// 		assert_eq!(
// 			Launchpad::account_pool_id_locked((BOB, 0)).unwrap(),
// 			StakingInfo { staking_balance: 10_000_000_000_000_000_000, is_claimed: true }
// 		);
// 		assert_eq!(Launchpad::pools(0).unwrap().total_locked, 20_000_000_000_000_000_000);
//
// 		assert_ok!(Launchpad::claim(Origin::signed(CHARLIE), 0));
// 		assert_eq!(TokenFungible::balance_of(TOKENA, CHARLIE), INITIAL_BALANCE);
// 		assert_eq!(TokenFungible::balance_of(TOKENB, CHARLIE), 5_000_000_000_000_000_000);
// 		assert_eq!(TokenFungible::balance_of(TOKENA, escrow_account), 0);
// 		assert_eq!(TokenFungible::balance_of(TOKENB, escrow_account), 0);
// 		assert_eq!(
// 			Launchpad::account_pool_id_locked((CHARLIE, 0)).unwrap(),
// 			StakingInfo { staking_balance: 10_000_000_000_000_000_000, is_claimed: true }
// 		);
//
// 		assert_noop!(Launchpad::claim(Origin::signed(BOB), 0), Error::<Test>::AlreadyClaim);
// 		assert_noop!(Launchpad::claim(Origin::signed(CHARLIE), 0), Error::<Test>::AlreadyClaim);
// 	})
// }
