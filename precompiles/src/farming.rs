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

use fp_evm::{PrecompileHandle, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, PrecompileSet};
use precompile_utils::prelude::*;
use sp_core::H160;
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

#[generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	Staking = "staking(uint256,uint256)",
	Claim = "claim(uint256)",
}

pub struct FarmingExtension<Runtime>(PhantomData<Runtime>);

impl<Runtime> PrecompileSet for FarmingExtension<Runtime>
where
	Runtime: pallet_farming::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_farming::Call<Runtime>>,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		let result = {
			let selector = match handle.read_selector() {
				Ok(selector) => selector,
				Err(e) => return Some(Err(e)),
			};
			if let Err(err) = handle.check_function_modifier(match selector {
				Action::Staking | Action::Claim => FunctionModifier::NonPayable,
			}) {
				return Some(Err(err))
			}
			match selector {
				Action::Staking => Self::staking(handle),
				Action::Claim => Self::claim(handle),
			}
		};
		Some(result)
	}
	fn is_precompile(&self, _address: H160) -> bool {
		true
	}
}

impl<Runtime> FarmingExtension<Runtime> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime> FarmingExtension<Runtime>
where
	Runtime: pallet_farming::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_farming::Call<Runtime>>,
{
	fn staking(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let pool_id = input.read::<u64>()?.into();
		let amount = input.read::<u128>()?.into();

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_farming::Call::<Runtime>::staking { pool_id, amount },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn claim(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(1)?;

		let pool_id = input.read::<u64>()?.into();

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_farming::Call::<Runtime>::claim { pool_id },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
}
