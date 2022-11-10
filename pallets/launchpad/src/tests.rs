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

const W3G: u128 = 1;
const USDT: u128 = 2;

const USDT_DECIMALS: u128 = 1_000_000;
const W3G_DECIMALS: u128 = 1_000_000_000_000_000_000;

fn create_tokens() {
	assert_ok!(TokenFungible::create_token(
		Origin::signed(ALICE),
		W3G,
		b"W3G1".to_vec(),
		b"W3G1".to_vec(),
		18
	));
	assert_ok!(TokenFungible::create_token(
		Origin::signed(ALICE),
		USDT,
		b"W3G2".to_vec(),
		b"W3G2".to_vec(),
		6
	));
}
fn set_balance() {
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), W3G, ALICE, 100 * W3G_DECIMALS));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), USDT, BOB, 100 * USDT_DECIMALS));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), USDT, CHARLIE, 100 * USDT_DECIMALS));
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
			W3G,
			USDT,
			10 * W3G_DECIMALS,
			1 * USDT_DECIMALS
		));

		let escrow_account = Launchpad::escrow_account_id(0);
		assert_eq!(TokenFungible::balance_of(W3G, ALICE), 100 * W3G_DECIMALS - 10 * W3G_DECIMALS);
		assert_eq!(TokenFungible::balance_of(W3G, escrow_account), 10 * W3G_DECIMALS);
		assert_eq!(
			Launchpad::pools(0).unwrap(),
			Pool {
				owner: ALICE,
				escrow_account,
				sale_start: 1,
				sale_end: 1 + 10,
				sale_token_id: W3G,
				buy_token_id: USDT,
				token_price: 1 * USDT_DECIMALS,
				total_sale_amount: 10 * W3G_DECIMALS,
				raise_amount: 10 * W3G_DECIMALS,
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
			W3G,
			USDT,
			10 * W3G_DECIMALS,
			1 * USDT_DECIMALS
		));

		assert_ok!(Launchpad::buy_token(Origin::signed(BOB), 0, 1));

		let escrow_account = Launchpad::escrow_account_id(0);
		assert_eq!(TokenFungible::balance_of(USDT, BOB), 100 * USDT_DECIMALS - 1 * USDT_DECIMALS);
		assert_eq!(TokenFungible::balance_of(USDT, escrow_account), 1 * USDT_DECIMALS);
		assert_eq!(
			Launchpad::account_pool_id_locked((BOB, 0)).unwrap(),
			ClaimInfo { balance: 1 * W3G_DECIMALS, is_claimed: false }
		);
		assert_eq!(Launchpad::pools(0).unwrap().raise_amount, 10 * W3G_DECIMALS - 1 * W3G_DECIMALS);

		assert_noop!(Launchpad::buy_token(Origin::signed(BOB), 1, 1), Error::<Test>::PoolNotFound);
		assert_ok!(Launchpad::buy_token(Origin::signed(BOB), 0, 1));

		assert_eq!(
			TokenFungible::balance_of(USDT, BOB),
			100 * USDT_DECIMALS - 1 * USDT_DECIMALS - 1 * USDT_DECIMALS
		);
		assert_eq!(TokenFungible::balance_of(USDT, escrow_account), 2 * USDT_DECIMALS);
		assert_eq!(
			Launchpad::pools(0).unwrap().raise_amount,
			10 * W3G_DECIMALS - 1 * W3G_DECIMALS - 1 * W3G_DECIMALS
		);

		run_to_block(12);
		assert_noop!(Launchpad::buy_token(Origin::signed(BOB), 0, 1), Error::<Test>::OutOfSaleTime);
	})
}

#[test]
fn claim_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		set_balance();
		assert_ok!(Launchpad::create_pool(
			Origin::signed(ALICE),
			1,
			10,
			W3G,
			USDT,
			10 * W3G_DECIMALS,
			1 * USDT_DECIMALS
		));

		assert_ok!(Launchpad::buy_token(Origin::signed(BOB), 0, 1));

		let escrow_account = Launchpad::escrow_account_id(0);
		assert_eq!(
			Launchpad::account_pool_id_locked((BOB, 0)).unwrap(),
			ClaimInfo { balance: 1 * W3G_DECIMALS, is_claimed: false }
		);
		assert_eq!(Launchpad::pools(0).unwrap().raise_amount, 10 * W3G_DECIMALS - 1 * W3G_DECIMALS);
		assert_ok!(Launchpad::buy_token(Origin::signed(BOB), 0, 1));
		assert_ok!(Launchpad::buy_token(Origin::signed(CHARLIE), 0, 3));

		assert_noop!(
			Launchpad::owner_claim(Origin::signed(ALICE), 0),
			Error::<Test>::ClaimNotStart
		);

		run_to_block(12);
		assert_noop!(Launchpad::owner_claim(Origin::signed(BOB), 0), Error::<Test>::NotOwner);
		assert_ok!(Launchpad::claim(Origin::signed(BOB), 0));
		assert_ok!(Launchpad::claim(Origin::signed(CHARLIE), 0));

		assert_eq!(TokenFungible::balance_of(W3G, BOB), 2 * W3G_DECIMALS);
		assert_eq!(TokenFungible::balance_of(W3G, CHARLIE), 3 * W3G_DECIMALS);
		assert_eq!(TokenFungible::balance_of(W3G, escrow_account), 5 * W3G_DECIMALS);

		assert_eq!(TokenFungible::balance_of(USDT, BOB), 100 * USDT_DECIMALS - 2 * USDT_DECIMALS);
		assert_eq!(
			TokenFungible::balance_of(USDT, CHARLIE),
			100 * USDT_DECIMALS - 3 * USDT_DECIMALS
		);
		assert_eq!(TokenFungible::balance_of(USDT, escrow_account), 5 * USDT_DECIMALS);

		assert_ok!(Launchpad::owner_claim(Origin::signed(ALICE), 0));
		assert_eq!(TokenFungible::balance_of(USDT, escrow_account), 0);
		assert_eq!(TokenFungible::balance_of(USDT, ALICE), 5 * USDT_DECIMALS);
		assert_eq!(TokenFungible::balance_of(W3G, ALICE), (100 - 10 + 5) * W3G_DECIMALS);
	})
}
