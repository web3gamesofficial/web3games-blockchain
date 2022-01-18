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

use crate as pallet_exchange_nft;
use frame_support::{construct_runtime, parameter_types, PalletId};
use frame_system as system;
use primitives::Balance;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const MILLICENTS: Balance = 10_000_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: Balance = 100 * CENTS;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		TokenFungible: pallet_token_fungible::{Pallet, Call, Storage, Event<T>},
		TokenMulti: pallet_token_multi::{Pallet, Call, Storage, Event<T>},
		ExchangeNft: pallet_exchange_nft::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

parameter_types! {
	pub const TokenFungiblePalletId: PalletId = PalletId(*b"w3g/tfpi");
	pub const TokenMultiPalletId: PalletId = PalletId(*b"w3g/tmpi");
	pub const StringLimit: u32 = 50;
	pub const CreateTokenDeposit: Balance = 500 * MILLICENTS;
}

impl pallet_token_fungible::Config for Test {
	type Event = Event;
	type PalletId = TokenFungiblePalletId;
	type FungibleTokenId = u32;
	type StringLimit = StringLimit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type Currency = Balances;
}

impl pallet_token_multi::Config for Test {
	type Event = Event;
	type PalletId = TokenMultiPalletId;
	type MultiTokenId = u32;
	type StringLimit = StringLimit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type Currency = Balances;
}

parameter_types! {
	pub const ExchangeNftPalletId: PalletId = PalletId(*b"w3g/exnp");
	pub const CreatePoolDeposit: Balance = 500 * MILLICENTS;
}

impl pallet_exchange_nft::Config for Test {
	type Event = Event;
	type PalletId = ExchangeNftPalletId;
	type NftPoolId = u32;
	type CreatePoolDeposit = CreatePoolDeposit;
	type Currency = Balances;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 100 * DOLLARS), (2, 100 * DOLLARS)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}
