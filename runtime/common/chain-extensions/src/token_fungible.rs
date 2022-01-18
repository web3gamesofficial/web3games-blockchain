// This file is part of Web3Games.

// Copyright (C) 2021-2022 Web3Games https://web3games.org
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use codec::Encode;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, SysConfig, UncheckedFrom,
};
use primitives::Balance;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

pub struct FungibleTokenExtension;

impl<C> ChainExtension<C> for FungibleTokenExtension
where
	C: pallet_contracts::Config + pallet_token_fungible::Config,
	<C as pallet_contracts::Config>::Call: From<pallet_token_fungible::Call<C>>,
{
	fn call<E>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			// create_token
			65537 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let (name, symbol, decimals): (Vec<u8>, Vec<u8>, u8) =
					env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_fungible::Pallet::<E::T>::do_create_token(
					&caller, name, symbol, decimals,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create token")
				})?;
			},

			// approve
			65538 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				let (id, spender, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					<E::T as SysConfig>::AccountId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_fungible::Pallet::<E::T>::do_approve(
					id, &caller, &spender, amount,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call approve"))?;
			},

			// transfer
			65539 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				let (id, recipient, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					<E::T as SysConfig>::AccountId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_fungible::Pallet::<E::T>::do_transfer(
					id, &caller, &recipient, amount,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call do_mint"))?;
			},

			// transfer_from
			65540 => {
				let mut env = env.buf_in_buf_out();

				let (id, sender, recipient, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as SysConfig>::AccountId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_fungible::Pallet::<E::T>::do_transfer(
					id, &sender, &recipient, amount,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call do_mint"))?;
			},

			// mint
			65541 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				let (id, account, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					<E::T as SysConfig>::AccountId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id =
					pallet_token_fungible::Pallet::<E::T>::do_mint(id, &caller, account, amount)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call do_mint"))?;
			},

			// burn
			65542 => {
				let mut env = env.buf_in_buf_out();

				let (id, account, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					<E::T as SysConfig>::AccountId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_fungible::Pallet::<E::T>::do_burn(id, &account, amount)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call do_mint"))?;
			},

			// exists
			65543 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_fungible::Config>::FungibleTokenId = env.read_as()?;

				let exists: bool = pallet_token_fungible::Pallet::<E::T>::exists(id);

				let exists_slice = exists.encode();

				log::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);

				env.write(&exists_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create collection")
				})?;
			},

			// total_supply
			65544 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_fungible::Config>::FungibleTokenId = env.read_as()?;

				let exists: Balance = pallet_token_fungible::Pallet::<E::T>::total_supply(id);

				let exists_slice = exists.encode();

				log::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);

				env.write(&exists_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create collection")
				})?;
			},
			_ => {
				log::error!("call an unregistered `func_id`, func_id:{:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			},
		}
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
