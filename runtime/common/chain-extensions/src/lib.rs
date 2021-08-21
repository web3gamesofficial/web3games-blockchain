#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Randomness;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::{DispatchError, RuntimeDebug};
use sp_std::{marker::PhantomData, vec::Vec};

mod tokens;

pub use tokens::TokensExtension;

pub struct Web3gamesExtensions<C>(PhantomData<C>);

impl<C: pallet_contracts::Config + pallet_token::Config> ChainExtension<C>
	for Web3gamesExtensions<C>
{
	fn call<E>(func_id: u32, mut env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			1024 => {
				let mut env = env.buf_in_buf_out();
				let random_slice = <E::T as pallet_contracts::Config>::Randomness::random_seed()
					.0
					.encode();
				log::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);
				env.write(&random_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call random"))?;

				Ok(RetVal::Converging(0))
			}
			// 0x800 - 0x880
			id if id >= 2048 && id < 2176 => TokensExtension::call(func_id, env),
			_ => {
				log::error!("call an unregistered `func_id`, func_id:{:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			}
		}
	}

	fn enabled() -> bool {
		true
	}
}
