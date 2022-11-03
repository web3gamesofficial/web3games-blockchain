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

const INITIAL_BALANCE: u128 = 100_000_000_000_000_000_000;

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
		18
	));
	assert_ok!(TokenFungible::create_token(
		Origin::signed(ALICE),
		TOKENC,
		b"W3G2".to_vec(),
		b"W3G2".to_vec(),
		18
	));
}
fn set_balance() {
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), TOKENA, BOB, INITIAL_BALANCE));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), TOKENA, CHARLIE, INITIAL_BALANCE));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), TOKENB, ALICE, INITIAL_BALANCE));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), TOKENC, ALICE, INITIAL_BALANCE));
}

#[test]
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		set_balance();
		assert_ok!(Farming::set_admin(Origin::root(), ALICE));
		assert_ok!(Farming::create_pool(
			Origin::signed(ALICE),
			1,
			10,
			TOKENA,
			TOKENB,
			10_000_000_000_000_000_000,
		));

		let escrow_account = Farming::escrow_account_id(0);
		assert_eq!(
			TokenFungible::balance_of(TOKENB, ALICE),
			INITIAL_BALANCE - 10_000_000_000_000_000_000
		);
		assert_eq!(TokenFungible::balance_of(TOKENB, escrow_account), 10_000_000_000_000_000_000);
		assert_eq!(
			Farming::pools(0).unwrap(),
			Pool {
				escrow_account,
				start_at: 1,
				end_time: 1 + 10,
				locked_token_id: TOKENA,
				award_token_id: TOKENB,
				total_locked: 0,
				total_award: 10_000_000_000_000_000_000,
			}
		);
		assert_eq!(Farming::next_pool_id(), 1);
		assert_eq!(System::block_number(), 1);
	})
}

#[test]
fn staking_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		set_balance();
		assert_ok!(Farming::set_admin(Origin::root(), ALICE));
		assert_ok!(Farming::create_pool(
			Origin::signed(ALICE),
			1,
			10,
			TOKENA,
			TOKENB,
			10_000_000_000_000_000_000,
		));
		assert_ok!(Farming::staking(Origin::signed(BOB), 0, 10_000_000_000_000_000_000));
		let escrow_account = Farming::escrow_account_id(0);
		assert_eq!(
			TokenFungible::balance_of(TOKENA, BOB),
			INITIAL_BALANCE - 10_000_000_000_000_000_000
		);
		assert_eq!(TokenFungible::balance_of(TOKENA, escrow_account), 10_000_000_000_000_000_000);
		assert_eq!(
			Farming::account_pool_id_locked((BOB, 0)).unwrap(),
			StakingInfo { staking_balance: 10_000_000_000_000_000_000, is_claimed: false }
		);
		assert_eq!(Farming::pools(0).unwrap().total_locked, 10_000_000_000_000_000_000);

		assert_noop!(
			Farming::staking(Origin::signed(BOB), 1, 10_000_000_000_000_000_000),
			Error::<Test>::PoolNotFound
		);
		assert_ok!(Farming::staking(Origin::signed(BOB), 0, 10_000_000_000_000_000_000));

		assert_eq!(
			TokenFungible::balance_of(TOKENA, BOB),
			INITIAL_BALANCE - 10_000_000_000_000_000_000 - 10_000_000_000_000_000_000
		);
		assert_eq!(TokenFungible::balance_of(TOKENA, escrow_account), 20_000_000_000_000_000_000);
		assert_eq!(Farming::pools(0).unwrap().total_locked, 20_000_000_000_000_000_000);

		run_to_block(12);
		assert_noop!(
			Farming::staking(Origin::signed(BOB), 0, 10_000_000_000_000_000_000),
			Error::<Test>::StakingTimeout
		);
	})
}

#[test]
fn claim_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		set_balance();
		assert_ok!(Farming::set_admin(Origin::root(), ALICE));
		assert_ok!(Farming::create_pool(
			Origin::signed(ALICE),
			1,
			10,
			TOKENA,
			TOKENB,
			10_000_000_000_000_000_000,
		));
		assert_ok!(Farming::staking(Origin::signed(BOB), 0, 10_000_000_000_000_000_000));
		assert_ok!(Farming::staking(Origin::signed(CHARLIE), 0, 10_000_000_000_000_000_000));

		assert_noop!(Farming::claim(Origin::signed(BOB), 1), Error::<Test>::PoolNotFound);
		assert_noop!(Farming::claim(Origin::signed(BOB), 0), Error::<Test>::ClaimNotStart);

		run_to_block(12);
		assert_noop!(Farming::claim(Origin::signed(ALICE), 0), Error::<Test>::NotStaking);

		assert_ok!(Farming::claim(Origin::signed(BOB), 0));

		let escrow_account = Farming::escrow_account_id(0);
		assert_eq!(TokenFungible::balance_of(TOKENA, BOB), INITIAL_BALANCE);
		assert_eq!(TokenFungible::balance_of(TOKENB, BOB), 5_000_000_000_000_000_000);
		assert_eq!(TokenFungible::balance_of(TOKENA, escrow_account), 10_000_000_000_000_000_000);
		assert_eq!(TokenFungible::balance_of(TOKENB, escrow_account), 5_000_000_000_000_000_000);
		assert_eq!(
			Farming::account_pool_id_locked((BOB, 0)).unwrap(),
			StakingInfo { staking_balance: 10_000_000_000_000_000_000, is_claimed: true }
		);
		assert_eq!(Farming::pools(0).unwrap().total_locked, 20_000_000_000_000_000_000);

		assert_ok!(Farming::claim(Origin::signed(CHARLIE), 0));
		assert_eq!(TokenFungible::balance_of(TOKENA, CHARLIE), INITIAL_BALANCE);
		assert_eq!(TokenFungible::balance_of(TOKENB, CHARLIE), 5_000_000_000_000_000_000);
		assert_eq!(TokenFungible::balance_of(TOKENA, escrow_account), 0);
		assert_eq!(TokenFungible::balance_of(TOKENB, escrow_account), 0);
		assert_eq!(
			Farming::account_pool_id_locked((CHARLIE, 0)).unwrap(),
			StakingInfo { staking_balance: 10_000_000_000_000_000_000, is_claimed: true }
		);

		assert_noop!(Farming::claim(Origin::signed(BOB), 0), Error::<Test>::AlreadyClaim);
		assert_noop!(Farming::claim(Origin::signed(CHARLIE), 0), Error::<Test>::AlreadyClaim);
	})
}

#[test]
fn force_claim_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		set_balance();
		assert_ok!(Farming::set_admin(Origin::root(), ALICE));
		assert_ok!(Farming::create_pool(
			Origin::signed(ALICE),
			1,
			10,
			TOKENA,
			TOKENB,
			10_000_000_000_000_000_000,
		));
		assert_ok!(Farming::staking(Origin::signed(BOB), 0, 10_000_000_000_000_000_000));
		assert_ok!(Farming::staking(Origin::signed(CHARLIE), 0, 10_000_000_000_000_000_000));

		assert_noop!(Farming::force_claim(Origin::signed(BOB), BOB, 0), Error::<Test>::NoPermisson);
		assert_ok!(Farming::set_admin(Origin::root(), ALICE));

		assert_ok!(Farming::force_claim(Origin::signed(ALICE), BOB, 0));

		let escrow_account = Farming::escrow_account_id(0);
		assert_eq!(TokenFungible::balance_of(TOKENA, BOB), INITIAL_BALANCE);
		assert_eq!(TokenFungible::balance_of(TOKENA, escrow_account), 10_000_000_000_000_000_000);

		assert_eq!(Farming::account_pool_id_locked((BOB, 0)), None);
		assert_eq!(Farming::pools(0).unwrap().total_locked, 10_000_000_000_000_000_000);

		run_to_block(12);

		assert_noop!(Farming::claim(Origin::signed(BOB), 0), Error::<Test>::NotStaking);
	})
}
