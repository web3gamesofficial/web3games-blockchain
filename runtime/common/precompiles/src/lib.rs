#![cfg_attr(not(feature = "std"), no_std)]

use codec::Decode;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{Context, Precompile, PrecompileResult, PrecompileSet};
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use sp_core::H160;
use sp_std::{marker::PhantomData, prelude::*};

pub mod token_multi;

pub use token_multi::MultiTokenExtension;

#[derive(Debug, Clone, Copy)]
pub struct Web3GamesPrecompiles<R>(PhantomData<R>);

impl<R> Web3GamesPrecompiles<R>
where
	R: pallet_evm::Config,
{
	pub fn new() -> Self {
		Self(Default::default())
	}
	pub fn used_addresses() -> sp_std::vec::Vec<H160> {
		sp_std::vec![1, 2, 3, 4, 5, 6, 7, 8, 1024, 1025, 1026]
			.into_iter()
			.map(|x| hash(x))
			.collect()
	}
}

impl<R> PrecompileSet for Web3GamesPrecompiles<R>
where
	R::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
	<R::Call as Dispatchable>::Origin: From<Option<R::AccountId>>,
	R: pallet_evm::Config + pallet_token_multi::Config,
	R::Call: From<pallet_token_multi::Call<R>>,
	<R as pallet_token_multi::Config>::MultiTokenId: Into<u32>,
{
	fn execute(
		&self,
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<PrecompileResult> {
		match address {
			// Ethereum precompiles
			a if a == hash(1) => Some(ECRecover::execute(input, target_gas, context, is_static)),
			a if a == hash(2) => Some(Sha256::execute(input, target_gas, context, is_static)),
			a if a == hash(3) => Some(Ripemd160::execute(input, target_gas, context, is_static)),
			a if a == hash(4) => Some(Identity::execute(input, target_gas, context, is_static)),
			a if a == hash(5) => Some(Modexp::execute(input, target_gas, context, is_static)),
			a if a == hash(6) => Some(Bn128Add::execute(input, target_gas, context, is_static)),
			a if a == hash(7) => Some(Bn128Mul::execute(input, target_gas, context, is_static)),
			a if a == hash(8) => Some(Bn128Pairing::execute(input, target_gas, context, is_static)),

			// Non-Web3Games specific nor Ethereum precompiles
			a if a == hash(1024) => {
				Some(Sha3FIPS256::execute(input, target_gas, context, is_static))
			}
			a if a == hash(1025) => {
				Some(Dispatch::<R>::execute(input, target_gas, context, is_static))
			}
			a if a == hash(1026) => {
				Some(ECRecoverPublicKey::execute(input, target_gas, context, is_static))
			}

			// Web3Games precompiles
			a if a == hash(2048) => {
				Some(MultiTokenExtension::<R>::execute(input, target_gas, context, is_static))
			}

			// Not support
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		Self::used_addresses().contains(&address)
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
