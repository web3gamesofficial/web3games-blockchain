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
use primitives::{Balance, TokenId};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

pub struct MultiTokenExtension;

impl<C> ChainExtension<C> for MultiTokenExtension
where
	C: pallet_contracts::Config + pallet_token_multi::Config,
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

				let uri: Vec<u8> = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_multi::Pallet::<E::T>::do_create_token(&caller, uri)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create token")
				})?;
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
				env.charge_weight(10000)?;

				let id = pallet_token_multi::Pallet::<E::T>::do_set_approval_for_all(
					&caller, id, &operator, approved,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call set_approval_for_all")
				})?;
			},

			// transfer_from
			65667 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, from, to, token_id, amount): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as SysConfig>::AccountId,
					TokenId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_multi::Pallet::<E::T>::do_transfer_from(
					&caller, id, &from, &to, token_id, amount,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call transfer_from")
				})?;
			},

			//batch_transfer_from
			65668 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, from, to, token_ids, amounts): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as SysConfig>::AccountId,
					Vec<TokenId>,
					Vec<Balance>,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_multi::Pallet::<E::T>::do_batch_transfer_from(
					&caller, id, &from, &to, token_ids, amounts,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call batch_transfer_from")
				})?;
			},

			//mint
			65669 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, to, token_id, amount): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					<E::T as SysConfig>::AccountId,
					TokenId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_multi::Pallet::<E::T>::do_mint(
					&caller, id, &to, token_id, amount,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call mint"))?;
			},

			//mint_batch
			65670 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, to, token_ids, amounts): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					<E::T as SysConfig>::AccountId,
					Vec<TokenId>,
					Vec<Balance>,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_multi::Pallet::<E::T>::do_batch_mint(
					&caller, id, &to, token_ids, amounts,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call mint_batch")
				})?;
			},

			//burn
			65671 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, token_id, amount): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					TokenId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id =
					pallet_token_multi::Pallet::<E::T>::do_burn(&caller, id, token_id, amount)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call burn"))?;
			},

			//burn_batch
			65672 => {
				let mut env = env.buf_in_buf_out();

				let caller = env.ext().caller().clone();

				let (id, token_ids, amounts): (
					<E::T as pallet_token_multi::Config>::MultiTokenId,
					Vec<TokenId>,
					Vec<Balance>,
				) = env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_multi::Pallet::<E::T>::do_batch_burn(
					&caller, id, token_ids, amounts,
				)?;

				let id_slice = id.encode();

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call burn_batch")
				})?;
			},

			// exists
			65673 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_multi::Config>::MultiTokenId = env.read_as()?;

				let exists: bool = pallet_token_multi::Pallet::<E::T>::exists(id);

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

			// balance_of_batch
			65674 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_multi::Config>::MultiTokenId = env.read_as()?;

				let in_len = env.in_len();

				let token_ids: Vec<TokenId> = env.read_as_unbounded(in_len)?;

				let in_len = env.in_len();

				let accounts: Vec<<E::T as SysConfig>::AccountId> =
					env.read_as_unbounded(in_len)?;

				let balance_of_batch =
					pallet_token_multi::Pallet::<E::T>::balance_of_batch(id, &accounts, token_ids);

				let balance_of_batch_slice = balance_of_batch.encode();

				log::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);

				env.write(&balance_of_batch_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create collection")
				})?;
			},
			// balance_of
			65675 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_multi::Config>::MultiTokenId = env.read_as()?;
				let account: <E::T as SysConfig>::AccountId = env.read_as()?;
				let token_id: TokenId = env.read_as()?;

				let balance: Balance =
					pallet_token_multi::Pallet::<E::T>::balance_of(id, (token_id, &account));

				let balance_slice = balance.encode();

				log::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);

				env.write(&balance_slice, false, None).map_err(|_| {
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
