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

use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn test_deposit_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(WrapCurrency::deposit(Origin::signed(1), 1000,));
		assert_eq!(TokenFungible::balance_of(0, 1), 1000);
	})
}

#[test]
fn test_withdraw_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(WrapCurrency::deposit(Origin::signed(1), 1000,));
		assert_eq!(TokenFungible::balance_of(0, 1), 1000);

		assert_ok!(WrapCurrency::withdraw(Origin::signed(1), 500,));
		assert_eq!(TokenFungible::balance_of(0, 1), 500);
	})
}
