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

//! Autogenerated weights for pallet_exchange
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-12-09, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// target/release/web3games-node
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_exchange
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --output=./pallets/exchange/src/weights.rs
// --template=./.maintain/w3g-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_exchange.
pub trait WeightInfo {
	fn create_pool() -> Weight;
	fn add_liquidity() -> Weight;
	fn remove_liquidity() -> Weight;
	fn swap_exact_tokens_for_tokens() -> Weight;
	fn swap_tokens_for_exact_tokens() -> Weight;
}

/// Weights for pallet_exchange using the Web3Games node and recommended hardware.
pub struct W3GWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for W3GWeight<T> {
	// Storage: TokenFungible Tokens (r:2 w:1)
	// Storage: Exchange Pools (r:1 w:1)
	// Storage: Exchange NextPoolId (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
	// Storage: Exchange LpTokenToToken (r:0 w:1)
	fn create_pool() -> Weight {
		(54_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: Exchange Pools (r:1 w:0)
	// Storage: Exchange Reserves (r:1 w:1)
	// Storage: TokenFungible Balances (r:6 w:6)
	// Storage: Exchange KLast (r:1 w:0)
	// Storage: Exchange FeeTo (r:1 w:0)
	// Storage: TokenFungible Tokens (r:1 w:1)
	fn add_liquidity() -> Weight {
		(79_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(11 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	// Storage: Exchange Pools (r:1 w:0)
	// Storage: TokenFungible Balances (r:6 w:6)
	// Storage: Exchange Reserves (r:1 w:1)
	// Storage: Exchange KLast (r:1 w:0)
	// Storage: Exchange FeeTo (r:1 w:0)
	// Storage: TokenFungible Tokens (r:1 w:1)
	fn remove_liquidity() -> Weight {
		(85_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(11 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	// Storage: Exchange Pools (r:1 w:0)
	// Storage: Exchange Reserves (r:1 w:1)
	// Storage: TokenFungible Balances (r:4 w:4)
	fn swap_exact_tokens_for_tokens() -> Weight {
		(54_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: Exchange Pools (r:1 w:0)
	// Storage: Exchange Reserves (r:1 w:1)
	// Storage: TokenFungible Balances (r:4 w:4)
	fn swap_tokens_for_exact_tokens() -> Weight {
		(54_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_pool() -> Weight {
		(54_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	fn add_liquidity() -> Weight {
		(79_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(11 as Weight))
			.saturating_add(RocksDbWeight::get().writes(8 as Weight))
	}
	fn remove_liquidity() -> Weight {
		(85_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(11 as Weight))
			.saturating_add(RocksDbWeight::get().writes(8 as Weight))
	}
	fn swap_exact_tokens_for_tokens() -> Weight {
		(54_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	fn swap_tokens_for_exact_tokens() -> Weight {
		(54_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
}
