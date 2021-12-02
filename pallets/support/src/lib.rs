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

#![cfg_attr(not(feature = "std"), no_std)]

use primitives::{TokenId, TokenIndex};
use sp_std::prelude::*;

pub trait NonFungibleMetadata {
	type NonFungibleTokenId;

	fn token_name(id: Self::NonFungibleTokenId) -> Vec<u8>;
	fn token_symbol(id: Self::NonFungibleTokenId) -> Vec<u8>;
	fn token_uri(id: Self::NonFungibleTokenId) -> Vec<u8>;
}

pub trait NonFungibleEnumerable<AccountId> {
	type NonFungibleTokenId;

	fn total_supply(id: Self::NonFungibleTokenId) -> u32;
	fn token_by_index(id: Self::NonFungibleTokenId, index: TokenIndex) -> TokenId;
	fn token_of_owner_by_index(
		id: Self::NonFungibleTokenId,
		owner: AccountId,
		index: TokenIndex,
	) -> TokenId;
}
