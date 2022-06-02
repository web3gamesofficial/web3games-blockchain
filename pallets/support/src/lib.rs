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

use primitives::TokenIndex;
use sp_core::H160;
use sp_std::prelude::*;

/// This trait ensure we can convert EVM Address to FungibleTokenId,
/// NonFungibleTokenId, or MultiTokenId.
/// We will require each mod to have this trait implemented
pub trait TokenIdConversion<A> {
	/// Try to convert an evm address into token ID. Might not succeed.
	fn try_from_address(address: H160) -> Option<A>;
	/// Convert into an evm address. This is infallible.
	fn into_address(id: A) -> H160;
}

pub trait AccountMapping<A> {
	/// Convert account ID into an evm address.
	fn into_evm_address(account: A) -> H160;
}

pub trait FungibleMetadata {
	type FungibleTokenId;

	fn token_name(id: Self::FungibleTokenId) -> Vec<u8>;
	fn token_symbol(id: Self::FungibleTokenId) -> Vec<u8>;
	fn token_decimals(id: Self::FungibleTokenId) -> u8;
}

pub trait NonFungibleMetadata {
	type NonFungibleTokenId;
	type TokenId;

	fn token_name(id: Self::NonFungibleTokenId) -> Vec<u8>;
	fn token_symbol(id: Self::NonFungibleTokenId) -> Vec<u8>;
	fn token_uri(id: Self::NonFungibleTokenId, token_id: Self::TokenId) -> Vec<u8>;
}

pub trait NonFungibleEnumerable<AccountId> {
	type NonFungibleTokenId;
	type TokenId;

	fn total_supply(id: Self::NonFungibleTokenId) -> u32;
	fn token_by_index(id: Self::NonFungibleTokenId, index: TokenIndex) -> Self::TokenId;
	fn token_of_owner_by_index(
		id: Self::NonFungibleTokenId,
		owner: AccountId,
		index: TokenIndex,
	) -> Self::TokenId;
}

pub trait MultiMetadata {
	type MultiTokenId;
	type TokenId;

	fn uri(id: Self::MultiTokenId, token_id: Self::TokenId) -> Vec<u8>;
}
