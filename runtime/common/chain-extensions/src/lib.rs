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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::{
	dispatch::Dispatchable,
	traits::Randomness,
	weights::{GetDispatchInfo, PostDispatchInfo, Weight},
};
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;
use sp_std::marker::PhantomData;

mod token_fungible;
mod token_multi;
mod token_non_fungible;

pub use token_fungible::FungibleTokenExtension;
pub use token_multi::MultiTokenExtension;
pub use token_non_fungible::NonFungibleTokenExtension;

#[derive(Clone, Copy, Debug)]
pub struct RuntimeHelper<Runtime>(PhantomData<Runtime>);

impl<Runtime> RuntimeHelper<Runtime>
where
	Runtime: pallet_contracts::Config,
	<Runtime as pallet_contracts::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
{
	/// Try to dispatch a Substrate call.
	/// Return an error if there are not enough gas, or if the call fails.
	/// If successful returns the used gas using the Runtime GasWeightMapping.
	pub fn try_dispatch<Call>(
		origin: <<Runtime as pallet_contracts::Config>::Call as Dispatchable>::Origin,
		call: Call,
	) -> Result<Weight>
	where
		<Runtime as pallet_contracts::Config>::Call: From<Call>,
	{
		let call = <Runtime as pallet_contracts::Config>::Call::from(call);
		let dispatch_info = call.get_dispatch_info();

		let used_weight = call
			.dispatch(origin)
			.map_err(|e| DispatchError::Other(e.error.into()))?
			.actual_weight;

		Ok(used_weight.unwrap_or(dispatch_info.weight))
	}
}

pub struct Web3GamesChainExtensions<C>(PhantomData<C>);

impl<C> ChainExtension<C> for Web3GamesChainExtensions<C>
where
	C: pallet_contracts::Config
		+ pallet_token_fungible::Config
		+ pallet_token_non_fungible::Config
		+ pallet_token_multi::Config,
	<<C as pallet_contracts::Config>::Call as Dispatchable>::Origin: From<Option<C::AccountId>>,
	<C as pallet_contracts::Config>::Call: From<pallet_token_fungible::Call<C>>,
	<C as pallet_contracts::Config>::Call: From<pallet_token_non_fungible::Call<C>>,
	<C as pallet_contracts::Config>::Call: From<pallet_token_multi::Call<C>>,
{
	fn call<E>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			// 0x00-0x10000(0-65536): substrate pallet
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
			},
			// 0x10001-0x10040(65537-65600): token-fungible
			id if id >= 65537 && id < 65600 => FungibleTokenExtension::call(func_id, env),
			// 0x10041-0x10080(65601-65664): token-non-fungible
			id if id >= 65601 && id < 65664 => NonFungibleTokenExtension::call(func_id, env),
			// 0x10081-0x100c1(65665-65729): token-multi
			id if id >= 65665 && id < 65729 => MultiTokenExtension::call(func_id, env),
			_ => {
				log::error!("call an unregistered `func_id`, func_id:{:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			},
		}
	}

	fn enabled() -> bool {
		true
	}
}
