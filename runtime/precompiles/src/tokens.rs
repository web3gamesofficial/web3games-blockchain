use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use fp_evm::Precompile;
// use pallet_evm::{AddressMapping, GasWeightMapping};
use crate::Config;
use sp_std::{marker::PhantomData, prelude::*, result};

pub struct TokensPrecompile<T: Config> {
    _marker: PhantomData<T>,
}

impl<T> Precompile for TokensPrecompile<T>
where
    T: pallet_evm::Config + Config,
{
    fn execute(
        input: &[u8],
        _target_gas: Option<u64>,
        _context: &Context,
    ) -> core::result::Result<PrecompileOutput, ExitError> {
        log::debug!(target: "evm", "input: {:?}", input);

        let owner = T::AccountId::default();
        let instance_id = T::InstanceId::from(2u64);
        let token_id = T::TokenId::from(1u64);
        let balance = pallet_tokens::Pallet::<T>::balance_of(&owner, instance_id, token_id);
        log::debug!(target: "evm", "balance: {:?}", balance);

        let cost: u64 = 0;

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost,
            output: Default::default(),
            logs: Default::default(),
        })

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
