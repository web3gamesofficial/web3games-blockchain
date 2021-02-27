#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};

use frame_support::debug;
use frame_support::traits::Randomness;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::{DispatchError, AccountId32, RuntimeDebug};
use sp_std::{prelude::*, convert::TryFrom};

// use super::Runtime;
// pub trait Config: frame_system::Config + pallet_contracts::Config + pallet_poe::Config {}
pub trait Config: pallet_contracts::Config + pallet_erc1155::Config {
	type Randomness: Randomness<Self::Hash>;
}

/// Result that returns a [`DispatchError`] on error.
pub type Result<T> = sp_std::result::Result<T, DispatchError>;

pub fn to_account_id(account: &[u8]) -> AccountId32 {
	AccountId32::try_from(account).unwrap()
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct BalanceOf<AccountId, TokenId> {
	owner: AccountId,
	id: TokenId,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct TransferFromInput<AccountId, TokenId, TokenBalance> {
	from: AccountId,
	to: AccountId,
	id: TokenId,
	amount: TokenBalance,
}

/// chain extension of contract
pub struct SgcChainExtension;

impl<C: Config> ChainExtension<C> for SgcChainExtension {
	fn call<E>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			1001 => {
				debug::info!("run 1001");
				let mut env = env.buf_in_buf_out();
				let random_slice = <E::T as Config>::Randomness::random_seed().encode();
				// let random_slice = random_seed.encode();
				debug::native::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);
				env.write(&random_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call random"))?;
			}
			1002 => {
				debug::info!("run 1002");
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				debug::info!("caller: {:?}", caller);

				let input: BalanceOf<
					<E::T as SysConfig>::AccountId,
					<E::T as pallet_erc1155::Config>::TokenId,
				 > = env.read_as()?;

				let balance: u128 = pallet_erc1155::Module::<E::T>::balance_of(&input.owner, &input.id).into();
				debug::info!("balance: {:?}", balance);

				let balance_slice = balance.to_be_bytes();
				debug::info!("balance_slice: {:?}", balance_slice);

				debug::native::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);

				env.write(&balance_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call create collection"))?;
			}
			1003 => {
				debug::info!("run 1003");
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				debug::info!("caller: {:?}", caller);

				let in_len = env.in_len();
				debug::info!("in_len: {}", in_len);

				let mut buffer = vec![0u8; in_len as usize];

				env.read_into(&mut &mut buffer[..])?;
				debug::info!("buffer: {:?}", buffer);

				let input: TransferFromInput<
					<E::T as SysConfig>::AccountId,
					<E::T as pallet_erc1155::Config>::TokenId,
					<E::T as pallet_erc1155::Config>::TokenBalance,
				> = env.read_as()?;
				debug::info!("input: {:?}", input);

				let weight = 100_000;
				env.charge_weight(weight)?;

				pallet_erc1155::Module::<E::T>::do_transfer_from(&input.from, &input.to, &input.id, input.amount)?;
			}

			_ => {
				debug::error!("call an unregistered `func_id`, func_id:{:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			}
		}
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
