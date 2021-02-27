#![cfg_attr(not(feature = "std"), no_std)]

mod erc1155;

use sp_core::H160;
use evm::{ExitSucceed, ExitError, Context};
use pallet_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
use fp_evm::{PrecompileSet, Precompile};
use sp_std::{result, marker::PhantomData, prelude::*, str::FromStr};
pub use erc1155::Erc1155Precompile;

pub trait Config: pallet_evm::Config + pallet_erc1155::Config {}

// pub type SgcPrecompiles<Runtime> = (
// 	ECRecover,
// 	Sha256,
// 	Ripemd160,
// 	Identity,
// 	Erc1155<Runtime>,
// );

pub type EthereumPrecompiles = (
    ECRecover,
    Sha256,
    Ripemd160,
    Identity,
);

#[derive(Default)]
pub struct SgcPrecompiles<T: Config> {
    _marker: PhantomData<T>,
}

impl<T: Config> PrecompileSet for SgcPrecompiles<T> {
    fn execute(
        address: H160,
        input: &[u8],
        target_gas: Option<u64>,
        context: &Context,
    ) -> Option<result::Result<(ExitSucceed, Vec<u8>, u64), ExitError>> {
        EthereumPrecompiles::execute(address, input, target_gas, context).or_else(|| {
            let addr_erc1155 = H160::from_str("0000000000000000000000000000000000000401").unwrap();

            if address == addr_erc1155 {
                Some(Erc1155Precompile::<T>::execute(input, target_gas, context))
            } else {
                None
            }
        })
    }
}

// type PrecompiledCallable = fn(&[u8], Option<u64>, &Context)
// 	-> result::Result<(ExitSucceed, Vec<u8>, u64), ExitError>;

// fn get_precompiled_func_from_address(address: &H160) -> Option<PrecompiledCallable> {
// 	// ethereum precompiles
// 	let addr_ecrecover = H160::from_str("0000000000000000000000000000000000000001").unwrap();
// 	let addr_sha256 = H160::from_str("0000000000000000000000000000000000000002").unwrap();
// 	let addr_ripemd160 = H160::from_str("0000000000000000000000000000000000000003").unwrap();
// 	let addr_identity = H160::from_str("0000000000000000000000000000000000000004").unwrap();
// 	// sgc precompiles
// 	let addr_erc1155 = H160::from_str("0000000000000000000000000000000000000401").unwrap();

// 	let exec: Option<PrecompiledCallable> = match *address {
// 		addr_ecrecover => Some(ECRecover::execute),
// 		addr_sha256 => Some(Sha256::execute),
// 		addr_ripemd160 => Some(Ripemd160::execute),
// 		addr_identity => Some(Identity::execute),
// 		addr_erc1155 => Some(Erc1155::execute),
// 		_ => None,
// 	};
// 	exec
// }

// pub struct SgcPrecompiles;

// impl PrecompileSet for SgcPrecompiles {
// 	fn execute(
// 		address: H160,
// 		input: &[u8],
// 		target_gas: Option<u64>,
// 		context: &Context,
// 	) -> Option<result::Result<(ExitSucceed, Vec<u8>, u64), ExitError>> {

// 		match get_precompiled_func_from_address(&address) {
// 		   Some(func) => return Some(func(input, target_gas, context)),
// 		   _ => {},
// 		};
// 		None
// 	}
// }
