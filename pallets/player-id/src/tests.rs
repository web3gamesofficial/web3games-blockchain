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
fn test_register_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(PlayerIdModule::register(Origin::signed(ALICE), vec![0u8, 10], ALICE,));
	});
}

#[test]
fn test_add_address_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(PlayerIdModule::register(Origin::signed(ALICE), vec![0u8, 10], ALICE,));

		assert_ok!(PlayerIdModule::add_address(
			Origin::signed(ALICE),
			PlayerId::try_from(vec![0u8, 10]).unwrap(),
			Chains::ETH,
			vec![0u8, 20],
		));
	});
}

#[test]
fn test_remove_address_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(PlayerIdModule::register(Origin::signed(ALICE), vec![0u8, 10], ALICE,));

		assert_ok!(PlayerIdModule::add_address(
			Origin::signed(ALICE),
			PlayerId::try_from(vec![0u8, 10]).unwrap(),
			Chains::ETH,
			vec![0u8, 20],
		));

		assert_ok!(PlayerIdModule::remove_address(
			Origin::signed(ALICE),
			PlayerId::try_from(vec![0u8, 10]).unwrap(),
			Chains::ETH,
			Address::try_from(vec![0u8, 20]).unwrap(),
		));
	});
}
