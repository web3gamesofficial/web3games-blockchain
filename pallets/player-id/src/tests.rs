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
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_register_works() {
	new_test_ext().execute_with(|| {
		let player_id_vec = "tester".as_bytes().to_vec();
		assert_ok!(PlayerIdModule::register(Origin::signed(ALICE), player_id_vec, ALICE));
	});
}

// #[test]
// fn test_add_address_works() {
// 	new_test_ext().execute_with(|| {
// 		let player_id_vec = "tester".as_bytes().to_vec();
// 		assert_ok!(PlayerIdModule::register(Origin::signed(ALICE), player_id_vec.clone(), ALICE));

// 		let player_id = PlayerId::try_from(player_id_vec).unwrap();
// 		let eth_address = "6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b".as_bytes().to_vec();
// 		assert_ok!(PlayerIdModule::add_address(Origin::signed(ALICE), player_id, Chains::ETH, eth_address));
// 	});
// }
