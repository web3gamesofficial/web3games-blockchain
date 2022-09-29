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
		fn get_amount_in_price(supply: Vec<u8>, path: Vec<Vec<u8>>) -> Option<Vec<Balance>>;
		fn get_amount_out_price(supply: Vec<u8>, path: Vec<Vec<u8>>) -> Option<Vec<Balance>>;
		fn get_estimate_lp_token(
			token_0: Vec<u8>,
			amount_0: Vec<u8>,
			token_1: Vec<u8>,
			amount_1: Vec<u8>,
		) -> Option<Balance>;
		fn get_estimate_out_token(supply: Vec<u8>,token_0:Vec<u8>,token_1:Vec<u8>)-> Option<Balance>;
		fn get_liquidity_to_tokens(lp_token_0:Vec<u8>,lp_balance:Vec<u8>)-> Option<(Balance,Balance)>;
	}
}
