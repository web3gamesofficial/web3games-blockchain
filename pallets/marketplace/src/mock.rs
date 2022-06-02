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

use crate as pallet_marketplace;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU16, ConstU64},
	PalletId,
};
use primitives::Balance;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Percent,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type AccountId = u64;
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
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		TokenNonFungible: pallet_token_non_fungible::{Pallet, Call, Storage, Event<T>},
		TokenMulti: pallet_token_multi::{Pallet, Call, Storage, Event<T>},
		Marketplace: pallet_marketplace::{Pallet, Call, Storage, Event<T>},
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<1>;
	type WeightInfo = ();
}

parameter_types! {
	pub const TokenNonFungiblePalletId: PalletId = PalletId(*b"w3g/tnfp");
	pub const TokenMultiPalletId: PalletId = PalletId(*b"w3g/tmpi");
	pub const StringLimit: u32 = 50;
	pub const CreateTokenDeposit: Balance = 500 * MILLICENTS;
	pub const CreateCollectionDeposit: Balance = 500 * MILLICENTS;
}

impl pallet_token_non_fungible::Config for Test {
	type Event = Event;
	type PalletId = TokenNonFungiblePalletId;
	type NonFungibleTokenId = u32;
	type TokenId = u32;
	type StringLimit = StringLimit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type Currency = Balances;
}

impl pallet_token_multi::Config for Test {
	type Event = Event;
	type PalletId = TokenMultiPalletId;
	type MultiTokenId = u32;
	type TokenId = u32;
	type StringLimit = StringLimit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type Currency = Balances;
}

parameter_types! {
	pub const MarketplacePalletId: PalletId = PalletId(*b"w3g/mpct");
	pub const FeesCollectorShareCut: Percent = Percent::from_percent(2);
	pub const TreasuryAccount: AccountId = 10;
}

impl pallet_marketplace::Config for Test {
	type Event = Event;
	type Time = Timestamp;
	type PalletId = MarketplacePalletId;
	type Currency = Balances;
	type FeesCollectorShareCut = FeesCollectorShareCut;
	type FeesCollector = TreasuryAccount;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(0, 100 * DOLLARS),
			(1, 100 * DOLLARS),
			(2, 100 * DOLLARS),
			(3, 100 * DOLLARS),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	// Timestamp::set_timestamp(50);
	t.into()
}
