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
use primitives::Balance;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

pub struct MultiTokenExtension;

impl<C> ChainExtension<C> for MultiTokenExtension
where
	C: pallet_contracts::Config + pallet_token_multi::Config,
	<<C as pallet_contracts::Config>::Call as Dispatchable>::Origin: From<Option<C::AccountId>>,
	<C as pallet_contracts::Config>::Call: From<pallet_token_multi::Call<C>>,
{
	fn call<E>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			// create_token
			65665 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let (id, uri): (<C as pallet_token_multi::Config>::MultiTokenId, Vec<u8>) =
					env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_multi::Call::<E::T>::create_token { id, uri },
				)?;
				env.charge_weight(used_weight)?;
			},
			// set_approval_for_all
			65666 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let (id, operator, approved): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					<E::T as SysConfig>::AccountId,
					bool,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_multi::Call::<E::T>::set_approval_for_all {
						id,
						operator,
						approved,
					},
				)?;
				env.charge_weight(used_weight)?;
			},
			// transfer_from
			65667 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let (id, from, to, token_id, amount): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as SysConfig>::AccountId,
					<E::T as pallet_token_multi::Config>::TokenId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_multi::Call::<E::T>::transfer_from {
						id,
						from,
						to,
						token_id,
						amount,
					},
				)?;
				env.charge_weight(used_weight)?;
			},
			//batch_transfer_from
			65668 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let (id, from, to, token_ids, amounts): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as SysConfig>::AccountId,
					Vec<<E::T as pallet_token_multi::Config>::TokenId>,
					Vec<Balance>,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_multi::Call::<E::T>::batch_transfer_from {
						id,
						from,
						to,
						token_ids,
						amounts,
					},
				)?;
				env.charge_weight(used_weight)?;
			},
			//mint
			65669 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, to, token_id, amount): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as pallet_token_multi::Config>::TokenId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_multi::Call::<E::T>::mint { id, to, token_id, amount },
				)?;
				env.charge_weight(used_weight)?;
			},
			//mint_batch
			65670 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let (id, to, token_ids, amounts): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					<E::T as SysConfig>::AccountId,
					Vec<<E::T as pallet_token_multi::Config>::TokenId>,
					Vec<Balance>,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_multi::Call::<E::T>::mint_batch { id, to, token_ids, amounts },
				)?;
				env.charge_weight(used_weight)?;
			},
			//burn
			65671 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, token_id, amount): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					<E::T as pallet_token_multi::Config>::TokenId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_multi::Call::<E::T>::burn { id, token_id, amount },
				)?;
				env.charge_weight(used_weight)?;
			},
			//burn_batch
			65672 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, token_ids, amounts): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					Vec<<E::T as pallet_token_multi::Config>::TokenId>,
					Vec<Balance>,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_multi::Call::<E::T>::burn_batch { id, token_ids, amounts },
				)?;
				env.charge_weight(used_weight)?;
			},
			// exists
			65673 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_multi::Config>::MultiTokenId = env.read_as()?;

				let exists: bool = pallet_token_multi::Pallet::<E::T>::exists(id);

				let output = exists.encode();

				env.write(&output, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call exists"))?;
			},

			// balance_of_batch
			65674 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_multi::Config>::MultiTokenId = env.read_as()?;

				let in_len = env.in_len();

				let token_ids: Vec<<E::T as pallet_token_multi::Config>::TokenId> =
					env.read_as_unbounded(in_len)?;

				let in_len = env.in_len();

				let accounts: Vec<<E::T as SysConfig>::AccountId> =
					env.read_as_unbounded(in_len)?;

				let balance_of_batch =
					pallet_token_multi::Pallet::<E::T>::balance_of_batch(id, &accounts, token_ids);

				let output = balance_of_batch.encode();

				env.write(&output, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call balance_of_batch")
				})?;
			},
			// balance_of
			65675 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_multi::Config>::MultiTokenId = env.read_as()?;
				let account: <E::T as SysConfig>::AccountId = env.read_as()?;
				let token_id: <E::T as pallet_token_multi::Config>::TokenId = env.read_as()?;

				let balance: Balance =
					pallet_token_multi::Pallet::<E::T>::balance_of(id, (token_id, &account));

				let output = balance.encode();

				env.write(&output, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call balance_of")
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
