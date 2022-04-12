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
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

pub struct NonFungibleTokenExtension;

impl<C> ChainExtension<C> for NonFungibleTokenExtension
where
	C: pallet_contracts::Config + pallet_token_non_fungible::Config,
	<C as pallet_contracts::Config>::Call: From<pallet_token_non_fungible::Call<C>>,
{
	fn call<E>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			// create_token
			65601 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let (non_fungible_token_id, name, symbol, base_uri): (
					<C as pallet_token_non_fungible::Config>::NonFungibleTokenId,
					Vec<u8>,
					Vec<u8>,
					Vec<u8>,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_non_fungible::Pallet::<E::T>::do_create_token(
					&caller,
					non_fungible_token_id,
					name,
					symbol,
					base_uri,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create token")
				})?;
			},
			// approve
			65602 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, to, token_id): (
					<E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as pallet_token_non_fungible::Config>::TokenId,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_non_fungible::Pallet::<E::T>::do_approve(
					&caller, id, &to, token_id,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call approve"))?;
			},
			// set_approve_for_all
			65603 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, operator, approved): (
					<E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId,
					<E::T as SysConfig>::AccountId,
					bool,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_non_fungible::Pallet::<E::T>::do_set_approve_for_all(
					&caller, id, &operator, approved,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call set_approve_for_all")
				})?;
			},
			// transfer_from
			65604 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, from, to, token_id): (
					<E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as SysConfig>::AccountId,
					<E::T as pallet_token_non_fungible::Config>::TokenId,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_non_fungible::Pallet::<E::T>::do_transfer_from(
					&caller, id, &from, &to, token_id,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call set_approve_for_all")
				})?;
			},
			// mint
			65605 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, to, token_id): (
					<E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as pallet_token_non_fungible::Config>::TokenId,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id =
					pallet_token_non_fungible::Pallet::<E::T>::do_mint(&caller, id, &to, token_id)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call mint"))?;
			},
			// burn
			65606 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, token_id): (
					<E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId,
					<E::T as pallet_token_non_fungible::Config>::TokenId,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_non_fungible::Pallet::<E::T>::do_burn(&caller, id, token_id)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call burn"))?;
			},
			// exists
			65607 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId =
					env.read_as()?;

				let exists: bool = pallet_token_non_fungible::Pallet::<E::T>::exists(id);

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
			// token_exists
			65608 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId =
					env.read_as()?;
				let token_id: <E::T as pallet_token_non_fungible::Config>::TokenId =
					env.read_as()?;

				let token_exists: bool =
					pallet_token_non_fungible::Pallet::<E::T>::token_exists(id, token_id);

				let token_exists_slice = token_exists.encode();

				log::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);

				env.write(&token_exists_slice, false, None).map_err(|_| {
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
