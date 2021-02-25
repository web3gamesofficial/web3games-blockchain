use frame_support::debug;
use fp_evm::Precompile;
use evm::{ExitSucceed, ExitError, Context};
use frame_support::{dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo}, weights::{Pays, DispatchClass}};
use pallet_evm::{AddressMapping, GasWeightMapping};
use sp_core::U256;
use sp_std::{convert::TryFrom, fmt::Debug, marker::PhantomData, prelude::*, result};


pub struct Erc1155<T: pallet_evm::Config> {
    _marker: PhantomData<T>,
}

// pub struct Erc1155<AccountId, AddressMapping, Dex>(PhantomData<(AccountId, AddressMapping, Dex)>);

enum Action {
    QueryBalance,
    TransferFrom,
}

impl TryFrom<u8> for Action {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Action::QueryBalance),
            1 => Ok(Action::TransferFrom),
            _ => Err(()),
        }
    }
}

// pub struct Input<'a, Action, AccountId, AddressMapping> {
// 	content: &'a [u8],
// 	_marker: PhantomData<(Action, AccountId, AddressMapping)>,
// }
// impl<'a, Action, AccountId, AddressMapping> Input<'a, Action, AccountId, AddressMapping> {
// 	pub fn new(content: &'a [u8]) -> Self {
// 		Self {
// 			content,
// 			_marker: PhantomData,
// 		}
// 	}
// }


impl<T> Precompile for Erc1155<T>
where
    T: pallet_erc1155::Config + pallet_evm::Config,
{
    fn execute(
        input: &[u8],
        _target_gas: Option<u64>,
        _context: &Context,
    ) -> result::Result<(ExitSucceed, Vec<u8>, u64), ExitError> {

        debug::debug!(target: "evm", "input: {:?}", input);
        
        debug::info!("input: {:?}", input);

        Ok((ExitSucceed::Returned, [].to_vec(), 0))

        // let input = Input::<Action, T::AccountId, AddressMapping<T::AccountId>>::new(input);

        // let action = input.action()?;

        // match action {
        // 	Action::QueryBalance => {
        //         debug::debug!(target: "evm", "into QueryBalance");
        // 		Ok((ExitSucceed::Returned, [].to_vec(), 0))
        // 	}
        // 	Action::TransferFrom => {
        //         debug::debug!(target: "evm", "into TransferFrom");
        //         Ok((ExitSucceed::Returned, [].to_vec(), 0))
        // 	}
        // }
    }
}
