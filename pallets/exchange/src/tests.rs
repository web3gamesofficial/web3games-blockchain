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

const TOKENA: u128 = 1;
const TOKENB: u128 = 2;
const TOKENC: u128 = 3;

const INITIAL_BALANCE: u128 = 1_000_000_000_000_000_000;
const TOKENA_LIQUIDITY: u128 = 1000_000_000_000_000;
const TOKENB_LIQUIDITY: u128 = 2000_000_000_000_000;
const SWAP_VALUE: u128 = 1_000_000_000_000;
const MINIMUM_LIQUIDITY: u128 = 1000; // 10**3;

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
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), TOKENA, ALICE, INITIAL_BALANCE));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), TOKENB, ALICE, INITIAL_BALANCE));
	assert_ok!(TokenFungible::mint(Origin::signed(ALICE), TOKENC, ALICE, INITIAL_BALANCE));
}

#[test]
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		assert_ok!(Exchange::create_pool(Origin::signed(1), TOKENA, TOKENB));
		assert_eq!(Exchange::exists(TOKENA, TOKENB), true);
		assert_eq!(Exchange::exists(TOKENA, TOKENC), false);
		// assert_eq!(1, Exchange::generate_lp_token_id(1000000, TOKENB));
	})
}

#[test]
fn create_pool_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Exchange::create_pool(Origin::signed(ALICE), TOKENA, TOKENB),
			Error::<Test>::TokenAccountNotFound
		);

		create_tokens();
		assert_noop!(
			Exchange::create_pool(Origin::signed(ALICE), TOKENA, TOKENA),
			Error::<Test>::TokenRepeat
		);

		assert_ok!(Exchange::create_pool(Origin::signed(ALICE), TOKENA, TOKENB));
		assert_noop!(
			Exchange::create_pool(Origin::signed(ALICE), TOKENA, TOKENB),
			Error::<Test>::PoolAlreadyCreated
		);
	})
}

#[test]
fn add_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		assert_ok!(Exchange::create_pool(Origin::signed(ALICE), TOKENA, TOKENB));

		set_balance();

		let liquidity: u128 =
			Exchange::get_liquidity(TOKENA, TOKENA_LIQUIDITY, TOKENB, TOKENB_LIQUIDITY).unwrap();

		assert_ok!(Exchange::add_liquidity(
			Origin::signed(ALICE),
			TOKENA,
			TOKENB,
			TOKENA_LIQUIDITY,
			TOKENB_LIQUIDITY,
			0u128,
			0u128,
			ALICE,
			100
		));

		assert_eq!(TokenFungible::balance_of(1, ALICE), INITIAL_BALANCE - TOKENA_LIQUIDITY);
		assert_eq!(TokenFungible::balance_of(2, ALICE), INITIAL_BALANCE - TOKENB_LIQUIDITY);

		let lp_token: u128 = Exchange::generate_lp_token_id(TOKENA, TOKENB);
		let lp_token_account: u64 = Exchange::token_id_to_account_id(lp_token);

		assert_eq!(TokenFungible::balance_of(lp_token, ALICE), liquidity);
		assert_eq!(TokenFungible::balance_of(TOKENA, lp_token_account), TOKENA_LIQUIDITY);
		assert_eq!(TokenFungible::balance_of(TOKENB, lp_token_account), TOKENB_LIQUIDITY);
	})
}

#[test]
fn remove_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		assert_ok!(Exchange::create_pool(Origin::signed(ALICE), TOKENA, TOKENB));

		set_balance();

		let liquidity: u128 =
			Exchange::get_liquidity(TOKENA, TOKENA_LIQUIDITY, TOKENB, TOKENB_LIQUIDITY).unwrap();

		assert_ok!(Exchange::add_liquidity(
			Origin::signed(ALICE),
			TOKENA,
			TOKENB,
			TOKENA_LIQUIDITY,
			TOKENB_LIQUIDITY,
			0u128,
			0u128,
			ALICE,
			100
		));

		let lp_token: u128 = Exchange::generate_lp_token_id(TOKENA, TOKENB);
		let lp_token_account: u64 = Exchange::token_id_to_account_id(lp_token);

		let balance_0 = TokenFungible::balance_of(TOKENA, lp_token_account);
		let balance_1 = TokenFungible::balance_of(TOKENB, lp_token_account);
		let total_supply = TokenFungible::total_supply(lp_token);

		let amount_0 = (liquidity / 2) * balance_0 / total_supply;
		let amount_1 = (liquidity / 2) * balance_1 / total_supply;

		let remove_liquidity = liquidity / 2;

		assert_eq!(TokenFungible::balance_of(lp_token, ALICE), liquidity);

		assert_ok!(Exchange::remove_liquidity(
			Origin::signed(ALICE),
			TOKENA,
			TOKENB,
			remove_liquidity,
			0u128,
			0u128,
			ALICE,
			200
		));

		let burned_liquidity = TokenFungible::balance_of(lp_token, lp_token_account);
		assert_eq!(0, burned_liquidity);

		assert_eq!(
			TokenFungible::balance_of(TOKENA, ALICE),
			INITIAL_BALANCE - TOKENA_LIQUIDITY + amount_0
		);
		assert_eq!(
			TokenFungible::balance_of(TOKENB, ALICE),
			INITIAL_BALANCE - TOKENB_LIQUIDITY + amount_1
		);

		assert_eq!(TokenFungible::balance_of(lp_token, ALICE), liquidity - remove_liquidity);
		assert_eq!(
			TokenFungible::balance_of(TOKENA, lp_token_account),
			TOKENA_LIQUIDITY - amount_0
		);
		assert_eq!(
			TokenFungible::balance_of(TOKENB, lp_token_account),
			TOKENB_LIQUIDITY - amount_1
		);
	})
}

#[test]
fn swap_exact_tokens_for_tokens_should_work() {
	new_test_ext().execute_with(|| {
		create_tokens();
		assert_ok!(Exchange::create_pool(Origin::signed(ALICE), TOKENA, TOKENB));

		set_balance();

		assert_ok!(Exchange::add_liquidity(
			Origin::signed(ALICE),
			TOKENA,
			TOKENB,
			TOKENA_LIQUIDITY,
			TOKENB_LIQUIDITY,
			0u128,
			0u128,
			ALICE,
			100
		));

		let (reserve_in, reserve_out) = Exchange::get_reserves(TOKENA, TOKENB).unwrap();
		assert!(reserve_in == TOKENA_LIQUIDITY && reserve_out == TOKENB_LIQUIDITY);

		let amount_out_1 = Exchange::get_amount_out(SWAP_VALUE, reserve_in, reserve_out).unwrap();

		let amount_in_with_fee: U256 = U256::from(SWAP_VALUE).saturating_mul(U256::from(997u128));
		let numerator: U256 =
			U256::from(amount_in_with_fee).saturating_mul(U256::from(reserve_out));
		let denominator: U256 = (U256::from(reserve_in).saturating_mul(U256::from(1000u128)))
			.saturating_add(amount_in_with_fee);
		let amount_out_2 = numerator
			.checked_div(denominator)
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())
			.unwrap();

		assert_eq!(amount_out_1, amount_out_2);

		let path: Vec<u128> = vec![TOKENA, TOKENB];
		assert_ok!(Exchange::swap_exact_tokens_for_tokens(
			Origin::signed(ALICE),
			SWAP_VALUE,
			0,
			path.clone(),
			ALICE,
			100
		));

		let (reserve_in_2, reserve_out_2) = Exchange::get_reserves(TOKENA, TOKENB).unwrap();
		assert_eq!(reserve_in_2, TOKENA_LIQUIDITY + SWAP_VALUE);
		assert_eq!(reserve_out_2, TOKENB_LIQUIDITY - amount_out_2);

		assert_eq!(
			TokenFungible::balance_of(TOKENA, ALICE),
			INITIAL_BALANCE - TOKENA_LIQUIDITY - SWAP_VALUE
		);
		assert_eq!(
			TokenFungible::balance_of(TOKENB, ALICE),
			INITIAL_BALANCE - TOKENB_LIQUIDITY + amount_out_1
		);
	})
}

#[test]
fn swap_tokens_for_exact_tokens_should_works() {
	new_test_ext().execute_with(|| {
		create_tokens();
		assert_ok!(Exchange::create_pool(Origin::signed(ALICE), TOKENA, TOKENB));

		set_balance();

		assert_ok!(Exchange::add_liquidity(
			Origin::signed(ALICE),
			TOKENA,
			TOKENB,
			TOKENA_LIQUIDITY,
			TOKENB_LIQUIDITY,
			0u128,
			0u128,
			ALICE,
			100
		));

		let (reserve_in, reserve_out) = Exchange::get_reserves(TOKENA, TOKENB).unwrap();
		assert!(reserve_in == TOKENA_LIQUIDITY && reserve_out == TOKENB_LIQUIDITY);

		let amount_in_1 = Exchange::get_amount_in(SWAP_VALUE, reserve_in, reserve_out).unwrap();

		let numerator: U256 = U256::from(reserve_in)
			.saturating_mul(U256::from(SWAP_VALUE))
			.saturating_mul(U256::from(1000u128));
		let denominator: U256 = (U256::from(reserve_out).saturating_sub(U256::from(SWAP_VALUE)))
			.saturating_mul(U256::from(997u128));
		let amount_in_2 = numerator
			.checked_div(denominator)
			.and_then(|r| r.checked_add(U256::one()))
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())
			.unwrap();

		assert_eq!(amount_in_1, amount_in_2);

		let path: Vec<u128> = vec![TOKENA, TOKENB];
		assert_ok!(Exchange::swap_tokens_for_exact_tokens(
			Origin::signed(ALICE),
			SWAP_VALUE,
			u128::MAX,
			path.clone(),
			ALICE,
			100
		));

		let (reserve_in_2, reserve_out_2) = Exchange::get_reserves(TOKENA, TOKENB).unwrap();
		assert_eq!(reserve_in_2, TOKENA_LIQUIDITY + amount_in_1);
		assert_eq!(reserve_out_2, TOKENB_LIQUIDITY - SWAP_VALUE);

		assert_eq!(
			TokenFungible::balance_of(TOKENA, ALICE),
			INITIAL_BALANCE - TOKENA_LIQUIDITY - amount_in_1
		);
		assert_eq!(
			TokenFungible::balance_of(TOKENB, ALICE),
			INITIAL_BALANCE - TOKENB_LIQUIDITY + SWAP_VALUE
		);
	})
}
