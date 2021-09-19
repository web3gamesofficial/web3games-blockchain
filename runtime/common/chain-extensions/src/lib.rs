#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::traits::Randomness;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;
use sp_std::marker::PhantomData;

mod token_multi;

pub use token_multi::MultiTokenExtension;

pub struct Web3gamesExtensions<C>(PhantomData<C>);

impl<C: pallet_contracts::Config + pallet_token_multi::Config> ChainExtension<C>
	for Web3gamesExtensions<C>
{
	fn call<E>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			// 0x00-0x1000(0-4096): substrate pallet
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
			// 0x1000-0x1040: token-fungible

			// 0x1040-0x1080: token-non-fungible

			// 0x1080-0x10c0(4224-4288): token-multi
			id if id >= 4224 && id < 4288 => MultiTokenExtension::call(func_id, env),
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
