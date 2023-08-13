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

#![cfg(test)]

use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

use super::*;

#[test]
fn switchoff_transaction_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			CallSwitchgear::switchoff_transaction(
				Origin::signed(5),
				b"Balances".to_vec(),
				b"transfer".to_vec()
			),
			BadOrigin
		);

		assert_eq!(
			CallSwitchgear::get_switchoff_transactions((
				b"Balances".to_vec(),
				b"transfer".to_vec()
			)),
			None
		);
		assert_ok!(CallSwitchgear::switchoff_transaction(
			Origin::root(),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		System::assert_last_event(Event::CallSwitchgear(crate::Event::TransactionSwitchedoff(
			b"Balances".to_vec(),
			b"transfer".to_vec(),
		)));
		assert_eq!(
			CallSwitchgear::get_switchoff_transactions((
				b"Balances".to_vec(),
				b"transfer".to_vec()
			)),
			Some(())
		);

		assert_noop!(
			CallSwitchgear::switchoff_transaction(
				Origin::root(),
				b"CallSwitchgear".to_vec(),
				b"switchoff_transaction".to_vec()
			),
			Error::<Test>::CannotSwitchOff
		);
		assert_noop!(
			CallSwitchgear::switchoff_transaction(
				Origin::root(),
				b"CallSwitchgear".to_vec(),
				b"some_other_call".to_vec()
			),
			Error::<Test>::CannotSwitchOff
		);
		assert_ok!(CallSwitchgear::switchoff_transaction(
			Origin::root(),
			b"OtherPallet".to_vec(),
			b"switchoff_transaction".to_vec()
		));
	});
}

#[test]
fn switchon_transaction_transaction_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(CallSwitchgear::switchoff_transaction(
			Origin::root(),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		assert_eq!(
			CallSwitchgear::get_switchoff_transactions((
				b"Balances".to_vec(),
				b"transfer".to_vec()
			)),
			Some(())
		);

		assert_noop!(
			CallSwitchgear::switchoff_transaction(
				Origin::signed(5),
				b"Balances".to_vec(),
				b"transfer".to_vec()
			),
			BadOrigin
		);

		assert_ok!(CallSwitchgear::switchon_transaction(
			Origin::root(),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		System::assert_last_event(Event::CallSwitchgear(crate::Event::TransactionSwitchedOn(
			b"Balances".to_vec(),
			b"transfer".to_vec(),
		)));
		assert_eq!(
			CallSwitchgear::get_switchoff_transactions((
				b"Balances".to_vec(),
				b"transfer".to_vec()
			)),
			None
		);

		assert_eq!(CallSwitchgear::get_overall_indicator(), false);

		assert_ok!(CallSwitchgear::switchoff_transaction(
			Origin::root(),
			b"All".to_vec(),
			b"transfer".to_vec()
		));

		assert_eq!(CallSwitchgear::get_overall_indicator(), true);

		assert_ok!(CallSwitchgear::switchon_transaction(
			Origin::root(),
			b"All".to_vec(),
			b"transfer".to_vec()
		));

		assert_eq!(CallSwitchgear::get_overall_indicator(), false);
	});
}
