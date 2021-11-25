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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::traits::Randomness;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;
use sp_std::marker::PhantomData;

mod token_multi;

pub use token_multi::MultiTokenExtension;

pub struct Web3GamesChainExtensions<C>(PhantomData<C>);

impl<C> ChainExtension<C> for Web3GamesChainExtensions<C>
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
			// 0x00-0x1000(0-4096): substrate pallet
			1024 => {
				let mut env = env.buf_in_buf_out();
				let random_slice =
					<E::T as pallet_contracts::Config>::Randomness::random_seed().0.encode();
				log::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);
				env.write(&random_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call random"))?;

				Ok(RetVal::Converging(0))
			}
			// 0x1000-0x1040: token-fungible

			// 0x1040-0x1080: token-non-fungible

			// 0x1080-0x10c0(4224-4288): token-multi
			id if id >= 4224 && id < 4288 => MultiTokenExtension::call(func_id, env),
			_ => {
				log::error!("call an unregistered `func_id`, func_id:{:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			}
		}
	}

	fn enabled() -> bool {
		true
	}
}
