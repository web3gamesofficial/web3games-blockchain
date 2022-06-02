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
use sp_std::prelude::*;

pub struct FungibleTokenExtension;

impl<C> ChainExtension<C> for FungibleTokenExtension
where
	C: pallet_contracts::Config + pallet_token_fungible::Config,
	<<C as pallet_contracts::Config>::Call as Dispatchable>::Origin: From<Option<C::AccountId>>,
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

				let (id, name, symbol, decimals): (
					<C as pallet_token_fungible::Config>::FungibleTokenId,
					Vec<u8>,
					Vec<u8>,
					u8,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_fungible::Call::<E::T>::create_token {
						id,
						name,
						symbol,
						decimals,
					},
				)?;
				env.charge_weight(used_weight)?;
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

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_fungible::Call::<E::T>::approve { id, spender, amount },
				)?;
				env.charge_weight(used_weight)?;
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

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_fungible::Call::<E::T>::transfer { id, recipient, amount },
				)?;
				env.charge_weight(used_weight)?;
			},
			// transfer_from
			65540 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let (id, sender, recipient, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as SysConfig>::AccountId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_fungible::Call::<E::T>::transfer_from {
						id,
						sender,
						recipient,
						amount,
					},
				)?;
				env.charge_weight(used_weight)?;
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

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_fungible::Call::<E::T>::mint { id, account, amount },
				)?;
				env.charge_weight(used_weight)?;
			},
			// burn
			65542 => {
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let (id, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;

				let used_weight = RuntimeHelper::<E::T>::try_dispatch(
					Some(caller).into(),
					pallet_token_fungible::Call::<E::T>::burn { id, amount },
				)?;
				env.charge_weight(used_weight)?;
			},
			// exists
			65543 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_fungible::Config>::FungibleTokenId = env.read_as()?;

				let exists: bool = pallet_token_fungible::Pallet::<E::T>::exists(id);

				let output = exists.encode();

				env.write(&output, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call exists"))?;
			},
			// total_supply
			65544 => {
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_fungible::Config>::FungibleTokenId = env.read_as()?;

				let total_supply: Balance = pallet_token_fungible::Pallet::<E::T>::total_supply(id);

				let output = total_supply.encode();

				env.write(&output, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call total_supply")
				})?;
			},
			// balance_of
			65545 => {
				let mut env = env.buf_in_buf_out();

				let (id, account): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					<E::T as SysConfig>::AccountId,
				) = env.read_as_unbounded(env.in_len())?;

				let balance: Balance =
					pallet_token_fungible::Pallet::<E::T>::balance_of(id, account);

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
