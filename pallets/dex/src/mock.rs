use super::*;
use crate as pallet_dex;
use sp_core::H256;
use sp_runtime::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
use frame_support::{
    parameter_types, construct_runtime,
    traits::{OnInitialize, OnFinalize, GenesisBuild},
};
use orml_traits::parameter_type_with_key;
use primitives::{CurrencyId, Balance, TokenSymbol, Amount, BlockNumber};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const MILLICENTS: Balance = 10_000_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: Balance = 100 * CENTS;

pub const SGC: CurrencyId = CurrencyId::Token(TokenSymbol::SGC);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = ();
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
}

parameter_type_with_key! {
    pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
        Zero::zero()
    };
}

impl orml_tokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = ();
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = SGC;
}

pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;

impl orml_currencies::Config for Test {
    type Event = Event;
    type MultiCurrency = Tokens;
    type NativeCurrency = AdaptedBasicCurrency;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

parameter_types! {
    pub const CreateInstanceDeposit: Balance = 1;
    pub const CreateExchangeDeposit: Balance = 1;
    pub const CreateCollectionDeposit: Balance = 1;
    pub const CreateCurrencyInstanceDeposit: Balance = 1;
}

impl token::Config for Test {
    type Event = Event;
    type CreateInstanceDeposit = CreateInstanceDeposit;
    type Currency = Balances;
    type TokenId = u64;
    type InstanceId = u64;
}

parameter_types! {
    pub const CurrencyTokenModuleId: ModuleId = ModuleId(*b"sgc/curr");
    pub const DexModuleId: ModuleId = ModuleId(*b"sgc/dexm");
}

impl currency_token::Config for Test {
    type Event = Event;
    type ModuleId = CurrencyTokenModuleId;
    type Currency = Currencies;
    type CreateCurrencyInstanceDeposit = CreateCurrencyInstanceDeposit;
    type GetNativeCurrencyId = GetNativeCurrencyId;
}

impl Config for Test {
    type Event = Event;
    type ModuleId = DexModuleId;
    type CreateExchangeDeposit = CreateExchangeDeposit;
    type Currency = Balances;
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
        Currencies: orml_currencies::{Pallet, Call, Event<T>},
        Dex: pallet_dex::{Pallet, Call, Storage, Event<T>},
        Token: token::{Pallet, Call, Storage, Event<T>},
        CurrencyToken: currency_token::{Pallet, Call, Storage, Event<T>},
    }
);

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    pallet_balances::GenesisConfig::<Test>{
        balances: vec![(1, 100 * DOLLARS), (2, 100 * DOLLARS)],
    }.assimilate_storage(&mut t).unwrap();
    currency_token::GenesisConfig::<Test>{
        instance: (1, [].to_vec())
    }.assimilate_storage(&mut t).unwrap();
    t.into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Dex::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        Dex::on_initialize(System::block_number());
    }
}
