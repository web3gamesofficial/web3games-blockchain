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
const USDC: u128 = 3;

const W3G_DECIMALS: u128 = 1_000_000_000_000_000_000;
const USDT_DECIMALS: u128 = 1_000_000;
const USDC_DECIMALS: u128 = 1_000_000;

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
		18
	));
	assert_ok!(TokenFungible::create_token(
		Origin::signed(ALICE),
		USDC,
		b"W3G2".to_vec(),
		b"W3G2".to_vec(),
		18
	));
}
fn set_balance() {
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), W3G, BOB, 100 * W3G_DECIMALS));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), W3G, CHARLIE, 100 * W3G_DECIMALS));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), USDT, ALICE, 100 * USDT_DECIMALS));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), USDC, ALICE, 100 * USDC_DECIMALS));
}

#[test]
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		set_balance();

		//set_admin to ALICE
		assert_ok!(Farming::set_admin(Origin::root(), ALICE));
		assert_ok!(Farming::create_pool(
			Origin::signed(ALICE),
			1,
			10,
			10,
			W3G,
			USDT,
			10 * USDT_DECIMALS,
		));

		//Check storage
		let escrow_account = Farming::escrow_account_id(0);
		assert_eq!(
			TokenFungible::balance_of(USDT, ALICE),
			100 * USDT_DECIMALS - 10 * USDT_DECIMALS
		);

		assert_eq!(TokenFungible::balance_of(USDT, escrow_account), 10 * USDT_DECIMALS);
		assert_eq!(
			Farming::pools(0).unwrap(),
			Pool {
				escrow_account,
				start_at: 1,
				staking_duration: 10,
				locked_duration: 10,
				locked_token_id: W3G,
				award_token_id: USDT,
				total_locked: 0,
				total_award: 10 * USDT_DECIMALS,
			}
		);
		assert_eq!(Farming::next_pool_id(), 1);
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
			10,
			10,
			10,
			W3G,
			USDT,
			10 * USDT_DECIMALS,
		));

		//StakingNotStart
		assert_noop!(
			Farming::staking(Origin::signed(BOB), 0, 10 * W3G_DECIMALS),
			Error::<Test>::StakingNotStart
		);

		//Run to start staking
		run_to_block(10);
		assert_ok!(Farming::staking(Origin::signed(BOB), 0, 10 * W3G_DECIMALS));
		let escrow_account = Farming::escrow_account_id(0);
		assert_eq!(TokenFungible::balance_of(W3G, BOB), 100 * W3G_DECIMALS - 10 * W3G_DECIMALS);
		assert_eq!(TokenFungible::balance_of(W3G, escrow_account), 10 * W3G_DECIMALS);
		assert_eq!(
			Farming::account_pool_id_locked((BOB, 0)).unwrap(),
			StakingInfo { staking_balance: 10 * W3G_DECIMALS, is_claimed: false }
		);
		assert_eq!(
			Farming::pools(0).unwrap(),
			Pool {
				escrow_account,
				start_at: 10,
				staking_duration: 10,
				locked_duration: 10,
				locked_token_id: W3G,
				award_token_id: USDT,
				total_locked: 10 * W3G_DECIMALS,
				total_award: 10 * USDT_DECIMALS,
			}
		);

		//PoolNotFound
		assert_noop!(
			Farming::staking(Origin::signed(BOB), 1, 10 * W3G_DECIMALS),
			Error::<Test>::PoolNotFound
		);

		//Secnod to staking
		assert_ok!(Farming::staking(Origin::signed(BOB), 0, 10 * W3G_DECIMALS));

		assert_eq!(TokenFungible::balance_of(W3G, BOB), 100 * W3G_DECIMALS - 20 * W3G_DECIMALS);
		assert_eq!(TokenFungible::balance_of(W3G, escrow_account), 20 * W3G_DECIMALS);
		assert_eq!(
			Farming::account_pool_id_locked((BOB, 0)).unwrap(),
			StakingInfo { staking_balance: 20 * W3G_DECIMALS, is_claimed: false }
		);
		assert_eq!(
			Farming::pools(0).unwrap(),
			Pool {
				escrow_account,
				start_at: 10,
				staking_duration: 10,
				locked_duration: 10,
				locked_token_id: W3G,
				award_token_id: USDT,
				total_locked: 20 * W3G_DECIMALS,
				total_award: 10 * USDT_DECIMALS,
			}
		);

		//CurrentLockedTime
		run_to_block(20);
		assert_noop!(
			Farming::staking(Origin::signed(BOB), 0, 10 * W3G_DECIMALS),
			Error::<Test>::CurrentLockedTime
		);
		//CurrentClaimTime
		run_to_block(30);
		assert_noop!(
			Farming::staking(Origin::signed(BOB), 0, 10 * W3G_DECIMALS),
			Error::<Test>::CurrentClaimTime
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
			10,
			10,
			10,
			W3G,
			USDT,
			10 * USDT_DECIMALS,
		));

		//StakingNotStart
		assert_noop!(Farming::claim(Origin::signed(BOB), 0), Error::<Test>::StakingNotStart);

		//Staking
		run_to_block(10);
		assert_ok!(Farming::staking(Origin::signed(BOB), 0, 2 * W3G_DECIMALS));
		assert_ok!(Farming::staking(Origin::signed(CHARLIE), 0, 3 * W3G_DECIMALS));
		assert_noop!(Farming::claim(Origin::signed(BOB), 0), Error::<Test>::CurrentStakingTime);

		//Locked
		run_to_block(20);
		assert_noop!(Farming::claim(Origin::signed(BOB), 1), Error::<Test>::PoolNotFound);
		assert_noop!(Farming::claim(Origin::signed(BOB), 0), Error::<Test>::CurrentLockedTime);

		//Claim
		run_to_block(30);
		assert_noop!(Farming::claim(Origin::signed(ALICE), 0), Error::<Test>::NotStaking);
		assert_ok!(Farming::claim(Origin::signed(BOB), 0));

		let escrow_account = Farming::escrow_account_id(0);
		assert_eq!(TokenFungible::balance_of(W3G, BOB), 100 * W3G_DECIMALS);
		assert_eq!(
			TokenFungible::balance_of(USDT, BOB),
			2 * W3G_DECIMALS * 10 * USDT_DECIMALS / (5 * W3G_DECIMALS)
		);
		assert_eq!(TokenFungible::balance_of(W3G, escrow_account), 3 * W3G_DECIMALS);
		assert_eq!(
			TokenFungible::balance_of(USDT, escrow_account),
			10 * USDT_DECIMALS - 2 * W3G_DECIMALS * 10 * USDT_DECIMALS / (5 * W3G_DECIMALS)
		);
		assert_eq!(
			Farming::account_pool_id_locked((BOB, 0)).unwrap(),
			StakingInfo { staking_balance: 2 * W3G_DECIMALS, is_claimed: true }
		);
		assert_eq!(
			Farming::pools(0).unwrap(),
			Pool {
				escrow_account,
				start_at: 10,
				staking_duration: 10,
				locked_duration: 10,
				locked_token_id: W3G,
				award_token_id: USDT,
				total_locked: 5 * W3G_DECIMALS,
				total_award: 10 * USDT_DECIMALS,
			}
		);

		assert_ok!(Farming::claim(Origin::signed(CHARLIE), 0));
		assert_eq!(TokenFungible::balance_of(W3G, CHARLIE), 100 * W3G_DECIMALS);
		assert_eq!(
			TokenFungible::balance_of(USDT, CHARLIE),
			3 * W3G_DECIMALS * 10 * USDT_DECIMALS / (5 * W3G_DECIMALS)
		);
		assert_eq!(TokenFungible::balance_of(W3G, escrow_account), 0);
		assert_eq!(TokenFungible::balance_of(USDT, escrow_account), 0);
		assert_eq!(
			Farming::account_pool_id_locked((CHARLIE, 0)).unwrap(),
			StakingInfo { staking_balance: 3 * W3G_DECIMALS, is_claimed: true }
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
			10,
			10,
			10,
			W3G,
			USDT,
			10 * USDT_DECIMALS,
		));

		run_to_block(10);
		assert_ok!(Farming::staking(Origin::signed(BOB), 0, 2 * W3G_DECIMALS));
		assert_ok!(Farming::staking(Origin::signed(CHARLIE), 0, 10 * W3G_DECIMALS));

		assert_noop!(Farming::force_claim(Origin::signed(BOB), BOB, 0), Error::<Test>::NoPermisson);
		assert_ok!(Farming::set_admin(Origin::root(), ALICE));

		//ignore status
		assert_ok!(Farming::force_claim(Origin::signed(ALICE), BOB, 0));

		let escrow_account = Farming::escrow_account_id(0);
		assert_eq!(TokenFungible::balance_of(W3G, BOB), 100 * W3G_DECIMALS);
		assert_eq!(TokenFungible::balance_of(W3G, escrow_account), 10 * W3G_DECIMALS);

		assert_eq!(Farming::account_pool_id_locked((BOB, 0)), None);
		assert_eq!(
			Farming::pools(0).unwrap(),
			Pool {
				escrow_account,
				start_at: 10,
				staking_duration: 10,
				locked_duration: 10,
				locked_token_id: W3G,
				award_token_id: USDT,
				total_locked: 10 * W3G_DECIMALS,
				total_award: 10 * USDT_DECIMALS,
			}
		);

		run_to_block(30);
		assert_noop!(Farming::claim(Origin::signed(BOB), 0), Error::<Test>::NotStaking);

		assert_ok!(Farming::force_claim(Origin::signed(ALICE), CHARLIE, 0));
		assert_eq!(Farming::account_pool_id_locked((CHARLIE, 0)), None);
		assert_eq!(TokenFungible::balance_of(W3G, CHARLIE), 100 * W3G_DECIMALS);
		assert_eq!(TokenFungible::balance_of(W3G, escrow_account), 0);

		assert_eq!(
			Farming::pools(0).unwrap(),
			Pool {
				escrow_account,
				start_at: 10,
				staking_duration: 10,
				locked_duration: 10,
				locked_token_id: W3G,
				award_token_id: USDT,
				total_locked: 0,
				total_award: 10 * USDT_DECIMALS,
			}
		);
	})
}
