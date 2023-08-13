// This file is part of Web3Games.
//
// Copyright (C) 2021-2022 Web3Games https://web3games.org
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

pub use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn wrap_currency_tests() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenFungible::create_token(
			Origin::signed(WrapCurrency::account_id()),
			0,
			b"W3G".to_vec(),
			b"W3G".to_vec(),
			18
		));
		assert_eq!(TokenFungible::exists(0), true);
		assert_ok!(WrapCurrency::deposit(Origin::signed(1), 10 * DOLLARS));
		assert_eq!(Balances::free_balance(1), 100 * DOLLARS - 10 * DOLLARS);
		assert_eq!(TokenFungible::balance_of(0, 1), 10 * DOLLARS);

		assert_ok!(WrapCurrency::withdraw(Origin::signed(1), 5 * DOLLARS));
		assert_eq!(Balances::free_balance(1), 100 * DOLLARS - 10 * DOLLARS + 5 * DOLLARS);
		assert_eq!(TokenFungible::balance_of(0, 1), 5 * DOLLARS);
	});
}
