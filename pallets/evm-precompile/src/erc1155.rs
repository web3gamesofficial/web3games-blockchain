use fp_evm::Precompile;
use evm::{ExitSucceed, ExitError, Context};
// use pallet_evm::{AddressMapping, GasWeightMapping};
use sp_std::{marker::PhantomData, prelude::*, result};
use crate::Config;

pub struct Erc1155Precompile<T: Config> {
    _marker: PhantomData<T>,
}


impl<T> Precompile for Erc1155Precompile<T>
where
    T: pallet_evm::Config + Config,
{
    fn execute(
        input: &[u8],
        _target_gas: Option<u64>,
        _context: &Context,
    ) -> result::Result<(ExitSucceed, Vec<u8>, u64), ExitError> {

        log::debug!(target: "evm", "input: {:?}", input);
        
        log::info!("input: {:?}", input);

        let owner = T::AccountId::default();
        let tao_id = T::TaoId::from(2u64);
        let token_id = T::TokenId::from(1u64);
        let balance = pallet_erc1155::Module::<T>::balance_of(&owner, tao_id, token_id);
        log::info!("balance: {:?}", balance);

        Ok((ExitSucceed::Returned, [].to_vec(), 0))

        // let input = Input::<Action, T::AccountId, AddressMapping<T::AccountId>>::new(input);

        // let action = input.action()?;

        // match action {
        // 	Action::QueryBalance => {
        //         log::debug!(target: "evm", "into QueryBalance");
        // 		Ok((ExitSucceed::Returned, [].to_vec(), 0))
        // 	}
        // 	Action::TransferFrom => {
        //         log::debug!(target: "evm", "into TransferFrom");
        //         Ok((ExitSucceed::Returned, [].to_vec(), 0))
        // 	}
        // }
    }
}


// enum Action {
//     QueryBalance,
//     TransferFrom,
// }

// impl TryFrom<u8> for Action {
//     type Error = ();

//     fn try_from(value: u8) -> Result<Self, Self::Error> {
//         match value {
//             0 => Ok(Action::QueryBalance),
//             1 => Ok(Action::TransferFrom),
//             _ => Err(()),
//         }
//     }
// }

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

// fn vec_u8_from_balance(b: u128) -> Vec<u8> {
// 	let mut be_bytes = [0u8; 32];
// 	U256::from(b).to_big_endian(&mut be_bytes[..]);
// 	be_bytes.to_vec()
// }
