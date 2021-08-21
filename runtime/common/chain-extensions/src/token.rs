use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Randomness;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, SysConfig, UncheckedFrom,
};
use primitives::Balance;
use sp_runtime::{DispatchError, RuntimeDebug};
use sp_std::{marker::PhantomData, vec, vec::Vec};

pub struct TokenExtension;

impl<C: pallet_contracts::Config + pallet_token::Config> ChainExtension<C> for TokenExtension {
	fn call<E>(func_id: u32, mut env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			2048 => {
				// fn balance_of(
				//     owner: &T::AccountId,
				//     instance_id: T::InstanceId,
				//     token_id: T::TokenId
				// ) -> Balance;
				let mut env = env.buf_in_buf_out();

				let owner: <E::T as SysConfig>::AccountId = env.read_as()?;
				let instance_id: <E::T as pallet_token::Config>::InstanceId = env.read_as()?;
				let token_id: <E::T as pallet_token::Config>::TokenId = env.read_as()?;

				let balance: Balance =
					pallet_token::Pallet::<E::T>::balance_of(&owner, instance_id, token_id);

				let balance_slice = balance.encode();

				log::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);

				env.write(&balance_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create collection")
				})?;
			}
			2049 => {
				// do_create_token(
				//     who: &T::AccountId,
				//     instance_id: T::InstanceId,
				//     token_id: T::TokenId,
				//     is_nf: bool,
				//     uri: Vec<u8>,
				// )
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let in_len = env.in_len();
				log::debug!("in_len {}", in_len);

				// let mut buffer = vec![0u8; in_len as usize];
				// env.read_into(&mut &mut buffer[..])?;

				let instance_id: <E::T as pallet_token::Config>::InstanceId = env.read_as()?;
				let token_id: <E::T as pallet_token::Config>::TokenId = env.read_as()?;
				let is_nf: bool = env.read_as()?;
				// let uri: Vec<u8> = env.read_as()?;
				let uri: Vec<u8> = vec![];

				let weight = 100_000;
				env.charge_weight(weight)?;

				pallet_token::Pallet::<E::T>::do_create_token(
					&caller,
					instance_id,
					token_id,
					is_nf,
					uri,
				)?;
			}
			_ => {
				log::error!("call an unregistered `func_id`, func_id:{:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			}
		}
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
