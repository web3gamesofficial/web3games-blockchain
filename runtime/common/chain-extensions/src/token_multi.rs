use codec::Encode;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, SysConfig, UncheckedFrom,
};
use primitives::{Balance, TokenId};
use sp_runtime::DispatchError;
use sp_std::{vec, vec::Vec};

pub struct MultiTokenExtension;

impl<C: pallet_contracts::Config + pallet_token_multi::Config> ChainExtension<C>
	for MultiTokenExtension
{
	fn call<E>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			4224 => {
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
			4225 => {
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

				// pallet_token_multi::Pallet::<E::T>::do_create_token(&caller, uri)?;
				pallet_token_multi::Call::<E::T>::create_token(uri);
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
