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
use primitives::{Balance, BlockNumber};
use sp_core::H160;
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

#[generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	CreatePool = "create_pool(uint256,uint256,uint256,uint256,uint256,uint256)",
	BuyToken = "buy_token(uint256,uint256)",
	Claim = "claim(uint256)",
	OwnerClaim = "owner_claim(uint256)",
}

pub struct LaunchpadExtension<Runtime>(PhantomData<Runtime>);

impl<Runtime> PrecompileSet for LaunchpadExtension<Runtime>
where
	Runtime: pallet_launchpad::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_launchpad::Call<Runtime>>,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		let result = {
			let selector = match handle.read_selector() {
				Ok(selector) => selector,
				Err(e) => return Some(Err(e)),
			};
			if let Err(err) = handle.check_function_modifier(match selector {
				Action::CreatePool | Action::BuyToken | Action::OwnerClaim | Action::Claim =>
					FunctionModifier::NonPayable,
			}) {
				return Some(Err(err))
			}
			match selector {
				Action::CreatePool => Self::create_pool(handle),
				Action::BuyToken => Self::buy_token(handle),
				Action::OwnerClaim => Self::owner_claim(handle),
				Action::Claim => Self::claim(handle),
			}
		};
		Some(result)
	}
	fn is_precompile(&self, _address: H160) -> bool {
		true
	}
}

impl<Runtime> LaunchpadExtension<Runtime> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime> LaunchpadExtension<Runtime>
where
	Runtime: pallet_launchpad::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_launchpad::Call<Runtime>>,
{
	fn create_pool(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(6)?;
		let sale_start = input.read::<BlockNumber>()?.into();
		let sale_duration = input.read::<BlockNumber>()?.into();
		let sale_token_id = input.read::<u128>()?.into();
		let buy_token_id = input.read::<u128>()?.into();
		let total_sale_amount = input.read::<Balance>()?.into();
		let token_price = input.read::<Balance>()?.into();
		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_launchpad::Call::<Runtime>::create_pool {
					sale_start,
					sale_duration,
					sale_token_id,
					buy_token_id,
					total_sale_amount,
					token_price,
				},
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn buy_token(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
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
				pallet_launchpad::Call::<Runtime>::buy_token { pool_id, amount },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn owner_claim(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
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
				pallet_launchpad::Call::<Runtime>::owner_claim { pool_id },
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
				pallet_launchpad::Call::<Runtime>::claim { pool_id },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
}
