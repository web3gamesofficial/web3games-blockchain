// This file is part of Web3Games.

// Copyright (C) 2021 Web3Games https://web3games.org
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
use frame_support::{dispatch::GetDispatchInfo, weights::extract_actual_weight};
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
			4224 => {
				log::info!("func id 4224");

				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				log::info!("caller {:?}", caller);

				let in_len = env.in_len();
				log::debug!("in_len {}", in_len);

				let uri: Vec<u8> = env.read_as_unbounded(in_len)?;
				log::info!("uri {:?}", uri);

				env.charge_weight(10000)?;

				let id = pallet_token_multi::Pallet::<E::T>::do_create_token(&caller, uri)?;

				let id_slice = id.encode();
				log::info!("id slice {:?}", id_slice);

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create token")
				})?;
			}
			// transfer_from
			4226 => {
				log::info!("func id 4226");
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_multi::Config>::MultiTokenId = env.read_as()?;
				let from: <E::T as SysConfig>::AccountId = env.read_as()?;
				let to: <E::T as SysConfig>::AccountId = env.read_as()?;
				let token_id: TokenId = env.read_as()?;
				let amount: Balance = env.read_as()?;

				let call =
					<E::T as pallet_contracts::Config>::Call::from(pallet_token_multi::Call::<
						E::T,
					>::transfer_from {
						id,
						from,
						to,
						token_id,
						amount,
					});

				let dispatch_info = call.get_dispatch_info();
				let charged = env.charge_weight(dispatch_info.weight)?;
				let result = env.ext().call_runtime(call);
				let actual_weight = extract_actual_weight(&result, &dispatch_info);
				env.adjust_weight(charged, actual_weight);

				match result {
					Ok(_) => {}
					Err(_) => return Err(DispatchError::Other("Call runtime returned error")),
				}
			}
			// mint
			4228 => {
				log::info!("func id 4228");
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_multi::Config>::MultiTokenId = env.read_as()?;
				let to: <E::T as SysConfig>::AccountId = env.read_as()?;
				let token_id: TokenId = env.read_as()?;
				let amount: Balance = env.read_as()?;

				let call =
					<E::T as pallet_contracts::Config>::Call::from(pallet_token_multi::Call::<
						E::T,
					>::mint {
						id,
						to,
						token_id,
						amount,
					});

				let dispatch_info = call.get_dispatch_info();
				let charged = env.charge_weight(dispatch_info.weight)?;
				let result = env.ext().call_runtime(call);
				let actual_weight = extract_actual_weight(&result, &dispatch_info);
				env.adjust_weight(charged, actual_weight);

				match result {
					Ok(_) => {}
					Err(_) => return Err(DispatchError::Other("Call runtime returned error")),
				}
			}
			// balance_of
			4232 => {
				log::info!("func id 4232");
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
