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

//! Autogenerated weights for web3games_token_multi
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-12-22, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// target/release/web3games-node
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=web3games_token_multi
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --output=./pallets/token-multi/src/weights.rs
// --template=./.maintain/w3g-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for web3games_token_multi.
pub trait WeightInfo {
	fn create_token() -> Weight;
	fn mint() -> Weight;
	fn mint_batch() -> Weight;
	fn set_approval_for_all() -> Weight;
	fn burn() -> Weight;
	fn burn_batch() -> Weight;
	fn transfer_from() -> Weight;
	fn batch_transfer_from() -> Weight;
}

/// Weights for web3games_token_multi using the Web3Games node and recommended hardware.
pub struct W3GWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for W3GWeight<T> {
	// Storage: TokenMulti Tokens (r:1 w:1)
	fn create_token() -> Weight {
		(15_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: TokenMulti Tokens (r:1 w:1)
	// Storage: TokenMulti Balances (r:1 w:1)
	fn mint() -> Weight {
		(20_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: TokenMulti Tokens (r:1 w:1)
	// Storage: TokenMulti Balances (r:5 w:5)
	fn mint_batch() -> Weight {
		(38_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: TokenMulti Tokens (r:1 w:0)
	// Storage: TokenMulti OperatorApprovals (r:0 w:1)
	fn set_approval_for_all() -> Weight {
		(16_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: TokenMulti Tokens (r:1 w:1)
	// Storage: TokenMulti Balances (r:1 w:1)
	fn burn() -> Weight {
		(19_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: TokenMulti Tokens (r:1 w:1)
	// Storage: TokenMulti Balances (r:5 w:5)
	fn burn_batch() -> Weight {
		(41_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: TokenMulti Balances (r:2 w:2)
	fn transfer_from() -> Weight {
		(21_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: TokenMulti Balances (r:10 w:10)
	fn batch_transfer_from() -> Weight {
		(56_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().writes(10 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_token() -> Weight {
		(15_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn mint() -> Weight {
		(20_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn mint_batch() -> Weight {
		(38_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
	}
	fn set_approval_for_all() -> Weight {
		(16_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn burn() -> Weight {
		(19_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn burn_batch() -> Weight {
		(41_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
	}
	fn transfer_from() -> Weight {
		(21_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn batch_transfer_from() -> Weight {
		(56_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(10 as Weight))
			.saturating_add(RocksDbWeight::get().writes(10 as Weight))
	}
}
