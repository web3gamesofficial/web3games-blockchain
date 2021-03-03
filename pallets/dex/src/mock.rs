use crate::{Module, Trait};

use sp_core::H256;
use frame_support::{impl_outer_origin, impl_outer_event ,parameter_types, weights::Weight, traits::OnFinalize, traits::OnInitialize};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill, ModuleId,
};
use frame_system as system;

pub const MILLISECS_PER_BLOCK: u64 = 4000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

impl_outer_origin! {
    pub enum Origin for Test where system = frame_system {}
}

mod dex {
    // pub use crate::Event;
    pub use super::super::*;
}

impl_outer_event! {
    pub enum TestEvent for Test {
        system<T>,
        pallet_balances<T>,
        dex<T>,
        token<T>,
        currency<T>,
        tao<T>,
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);

    pub const ExistentialDeposit: u64 = 1;
}
impl system::Trait for Test {
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type SystemWeightInfo = ();
    type PalletInfo = ();
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type BaseCallFilter = ();
}
impl pallet_balances::Trait for Test {
    type Balance = u64;
    type MaxLocks = ();
    type Event = TestEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = system::Module<Test>;
    type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl token::Trait for Test {
    type Event = TestEvent;
    type TokenBalance = u128;
    type TokenId = u64;
}

impl currency::Trait for Test {
    type Event = TestEvent;
}

impl tao::Trait for Test {
    type Event = TestEvent;
    type TaoId = u64;
}

parameter_types! {
    pub const DexModuleId: ModuleId = ModuleId(*b"spr/dexm");
}
impl Trait for Test {
    type Event = TestEvent;
    type ModuleId = DexModuleId;
}

pub type Dex = Module<Test>;
pub type Currency = currency::Module<Test>;
pub type Token = token::Module<Test>;
pub type Tao = tao::Module<Test>;
pub type System = frame_system::Module<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 10000), (2, 11000), (3, 12000), (4, 13000), (5, 14000)],
    }
        .assimilate_storage(&mut t)
        .unwrap();
    let mut ext: sp_io::TestExternalities = t.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Dex::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Dex::on_initialize(System::block_number());
    }
}