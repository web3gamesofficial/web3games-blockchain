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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use primitives::Balance;
use sp_api::decl_runtime_apis;
use sp_std::vec::Vec;

decl_runtime_apis! {
	pub trait ExchangeRuntimeApi<AccountId> where
		AccountId: Codec,
	{
		fn get_amount_in_price(pool_id:u128,supply: Balance, path: Vec<u128>) -> Option<Balance>;
		fn get_amount_out_price(pool_id:u128,supply: Balance, path: Vec<u128>) -> Option<Balance>;
	}
}
