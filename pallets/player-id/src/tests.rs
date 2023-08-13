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
use hex_literal::hex;

#[test]
fn test_register_works() {
	new_test_ext().execute_with(|| {
		let address: Address = PlayerIdModule::account_to_address(ALICE);
		let player_id_vec = "tester".as_bytes().to_vec();
		let player_id: PlayerId = player_id_vec.clone().try_into().unwrap();

		let addresses: BoundedVec<Address, ConstU32<10>> =
			BoundedVec::try_from(vec![address.clone()]).unwrap();

		assert_ok!(PlayerIdModule::register(Origin::signed(ALICE), player_id_vec, ALICE));

		assert_eq!(PlayerIdOf::<Test>::get(address.clone(), Chains::W3G).unwrap(), player_id);
		assert_eq!(Addresses::<Test>::get(player_id.clone(), Chains::W3G).unwrap(), addresses);

		let eth_address = hex!["6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"];
		assert_ok!(PlayerIdModule::add_address(
			Origin::signed(ALICE),
			player_id.clone(),
			Chains::ETH,
			eth_address.clone().to_vec()
		));

		assert_ok!(PlayerIdModule::remove_address(
			Origin::signed(ALICE),
			player_id,
			Chains::ETH,
			Address::try_from(eth_address.to_vec()).unwrap()
		));
	});
}
