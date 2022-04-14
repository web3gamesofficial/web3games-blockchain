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

use crate::RuntimeHelper;
use codec::Encode;
use frame_support::dispatch::Dispatchable;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

pub struct NonFungibleTokenExtension;

impl<C> ChainExtension<C> for NonFungibleTokenExtension
where
	C: pallet_contracts::Config + pallet_token_non_fungible::Config,
	<<C as pallet_contracts::Config>::Call as Dispatchable>::Origin: From<Option<C::AccountId>>,
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

				let (id, name, symbol, base_uri): (
					<C as pallet_token_non_fungible::Config>::NonFungibleTokenId,
					Vec<u8>,
					Vec<u8>,
					Vec<u8>,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_non_fungible::Call::<E::T>::create_token {
						id,
						name,
						symbol,
						base_uri,
					},
				)?;
				env.charge_weight(used_weight)?;
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

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_non_fungible::Call::<E::T>::approve { id, to, token_id },
				)?;
				env.charge_weight(used_weight)?;
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

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_non_fungible::Call::<E::T>::set_approve_for_all {
						id,
						operator,
						approved,
					},
				)?;
				env.charge_weight(used_weight)?;
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

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_non_fungible::Call::<E::T>::transfer_from {
						id,
						from,
						to,
						token_id,
					},
				)?;
				env.charge_weight(used_weight)?;
			},
			// mint
			65605 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let (id, from, to, token_id): (
					<E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as SysConfig>::AccountId,
					<E::T as pallet_token_non_fungible::Config>::TokenId,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_non_fungible::Call::<E::T>::transfer_from {
						id,
						from,
						to,
						token_id,
					},
				)?;
				env.charge_weight(used_weight)?;
			},
			// burn
			65606 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, token_id): (
					<E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId,
					<E::T as pallet_token_non_fungible::Config>::TokenId,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_non_fungible::Call::<E::T>::burn { id, token_id },
				)?;
				env.charge_weight(used_weight)?;
			},
			// exists
			65607 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId =
					env.read_as()?;

				let exists: bool = pallet_token_non_fungible::Pallet::<E::T>::exists(id);

				let output = exists.encode();

				env.write(&output, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call exists"))?;
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

				let output = token_exists.encode();

				env.write(&output, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call token_exists")
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
