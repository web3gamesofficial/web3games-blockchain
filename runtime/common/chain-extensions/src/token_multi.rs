use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Randomness;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, SysConfig, UncheckedFrom,
};
use primitives::{Balance, TokenId};
use sp_runtime::{DispatchError, RuntimeDebug};
use sp_std::{marker::PhantomData, vec, vec::Vec};

pub struct TokenExtension;

impl<C: pallet_contracts::Config + pallet_token_multi::Config> ChainExtension<C>
	for TokenExtension
{
	fn call<E>(func_id: u32, mut env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			2048 => {
				// fn balance_of(
				//     token_account: &T::AccountId,
				//     account: &T::AccountId,
				//     id: TokenId
				// ) -> Balance;
				let mut env = env.buf_in_buf_out();

				let token_account: <E::T as SysConfig>::AccountId = env.read_as()?;
				let account: <E::T as SysConfig>::AccountId = env.read_as()?;
				let id: TokenId = env.read_as()?;

				let balance: Balance =
					pallet_token_multi::Pallet::<E::T>::balance_of(&token_account, &account, id);

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
				//     uri: Vec<u8>,
				// )
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let in_len = env.in_len();
				log::debug!("in_len {}", in_len);

				let uri: Vec<u8> = vec![];

				let weight = 100_000;
				env.charge_weight(weight)?;

				pallet_token_multi::Pallet::<E::T>::do_create_token(&caller, uri)?;
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
