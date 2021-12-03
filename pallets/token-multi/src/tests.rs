// This file is part of Web3Games.

// Copyright (C) 2021 Web3Games https://web3games.org
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

pub type TokenId = u32;

#[test]
fn test_create_token_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let uri: Vec<u8> = vec![1, 2, 3];
		assert_ok!(TokenMulti::create_token(origin, uri));
	})
}

#[test]
fn test_mint_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let uri: Vec<u8> = vec![1, 2, 3];
		assert_ok!(TokenMulti::create_token(origin.clone(), uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_id: TokenId = 1u32;
		let amount: Balance = 1u128;
		assert_ok!(TokenMulti::mint(origin, id, to, token_id, amount));
	})
}

#[test]
fn test_batch_mint_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let uri: Vec<u8> = vec![1, 2, 3];
		assert_ok!(TokenMulti::create_token(origin.clone(), uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_ids: Vec<TokenId> = vec![1, 2, 3];
		let amounts: Vec<Balance> = vec![1, 1, 1];
		assert_ok!(TokenMulti::mint_batch(origin, id, to, token_ids, amounts));
	})
}

///
#[test]
fn set_approval_for_all_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let uri: Vec<u8> = vec![1, 2, 3];
		assert_ok!(TokenMulti::create_token(origin.clone(), uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_id: TokenId = 1u32;
		let amount: Balance = 1u128;
		assert_ok!(TokenMulti::mint(origin.clone(), id.clone(), to, token_id.clone(), amount));
		let operator: u64 = 2;
		let approved: bool = true;
		assert_ok!(TokenMulti::set_approval_for_all(origin, id, operator, approved));
	})
}

#[test]
fn test_transfer_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let uri: Vec<u8> = vec![1, 2, 3];
		assert_ok!(TokenMulti::create_token(origin.clone(), uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_id: TokenId = 1u32;
		let amount: Balance = 1u128;
		assert_ok!(TokenMulti::mint(origin.clone(), id, to, token_id.clone(), amount.clone()));
		let to: u64 = 2;
		assert_ok!(TokenMulti::transfer(origin.clone(), id, to, token_id.clone(), amount.clone()));
	})
}

#[test]
fn test_batch_transfer_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let uri: Vec<u8> = vec![1, 2, 3];
		assert_ok!(TokenMulti::create_token(origin.clone(), uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_ids: Vec<TokenId> = vec![1, 2, 3];
		let amounts: Vec<Balance> = vec![1, 1, 1];
		assert_ok!(TokenMulti::mint_batch(
			origin.clone(),
			id,
			to,
			token_ids.clone(),
			amounts.clone()
		));
		let to: u64 = 2;
		assert_ok!(TokenMulti::batch_transfer(
			origin.clone(),
			id,
			to,
			token_ids.clone(),
			amounts.clone()
		));
	})
}

#[test]
fn test_transfer_from_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let uri: Vec<u8> = vec![1, 2, 3];
		assert_ok!(TokenMulti::create_token(origin.clone(), uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_id: TokenId = 1u32;
		let amount: Balance = 1u128;
		assert_ok!(TokenMulti::mint(origin.clone(), id, to, token_id.clone(), amount.clone()));
		let from: u64 = 1;
		let to: u64 = 2;
		assert_ok!(TokenMulti::transfer_from(
			origin.clone(),
			id,
			from,
			to,
			token_id.clone(),
			amount.clone()
		));
	})
}

#[test]
fn test_batch_transfer_from_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let uri: Vec<u8> = vec![1, 2, 3];
		assert_ok!(TokenMulti::create_token(origin.clone(), uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_ids: Vec<TokenId> = vec![1, 2, 3];
		let amounts: Vec<Balance> = vec![1, 1, 1];
		assert_ok!(TokenMulti::mint_batch(
			origin.clone(),
			id,
			to,
			token_ids.clone(),
			amounts.clone()
		));
		let from: u64 = 1;
		let to: u64 = 2;
		assert_ok!(TokenMulti::batch_transfer_from(
			origin.clone(),
			id,
			from,
			to,
			token_ids.clone(),
			amounts.clone()
		));
	})
}

#[test]
fn test_burn_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let uri: Vec<u8> = vec![1, 2, 3];
		assert_ok!(TokenMulti::create_token(origin.clone(), uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_id: TokenId = 1u32;
		let amount: Balance = 1u128;
		assert_ok!(TokenMulti::mint(origin.clone(), id, to, token_id.clone(), amount.clone()));
		assert_ok!(TokenMulti::burn(origin, id, token_id, amount));
	})
}

#[test]
fn test_batch_burn_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let uri: Vec<u8> = vec![1, 2, 3];
		assert_ok!(TokenMulti::create_token(origin.clone(), uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_ids: Vec<TokenId> = vec![1, 2, 3];
		let amounts: Vec<Balance> = vec![1, 1, 1];
		assert_ok!(TokenMulti::mint_batch(
			origin.clone(),
			id,
			to,
			token_ids.clone(),
			amounts.clone()
		));
		assert_ok!(TokenMulti::burn_batch(origin, id, token_ids, amounts));
	})
}
