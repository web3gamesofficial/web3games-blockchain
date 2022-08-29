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

use crate::{FT_PRECOMPILE_ADDRESS_PREFIX, TOKEN_FUNGIBLE_CREATE_SELECTOR};
use fp_evm::{PrecompileHandle, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, PrecompileSet};
use pallet_support::{FungibleMetadata, TokenIdConversion};
use precompile_utils::prelude::*;
use primitives::Balance;
use sp_core::H160;
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};
use log;

pub type FungibleTokenIdOf<Runtime> = <Runtime as pallet_token_fungible::Config>::FungibleTokenId;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	CreatePool = "create_pool(uint256,uint256)",
	AddLiquidity = "add_liquidity(uint256,uin256,uint256,uint256,uint256,address)",
	RemoveLiquidity = "remove_liquidity(uint256,uin256,uint256,uint256,uint256,address)",
	SwapExactTokensForTokens = "swap_exact_tokens_for_tokens(uint256,uint256,uint256,[uint256,uint256],address)",
	SwapTokensForExactTokens = "swap_tokens_for_exact_tokens(uint256,uint256,uint256,[uint256,uint256],address)",
}

pub struct ExchangeExtension<Runtime>(PhantomData<Runtime>);

impl<Runtime> PrecompileSet for ExchangeExtension<Runtime>
where
	Runtime: pallet_evm::Config + pallet_exchange::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_exchange::Call<Runtime>>,
	<Runtime as pallet_token_fungible::Config>::FungibleTokenId: From<u128> + Into<u128>,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		let address = handle.code_address();
		let input = handle.input();
		log::debug!(target: "exchange", "log: {:?}", address);
		let result = {
			let selector = match handle.read_selector() {
				Ok(selector) => selector,
				Err(e) => return Some(Err(e)),
			};
			log::debug!(target: "exchange", "selector --- log: {:?}", address);
			if let Err(err) = handle.check_function_modifier(match selector {
				Action::CreatePool |
				Action::AddLiquidity |
				Action::RemoveLiquidity |
				Action::SwapExactTokensForTokens |
				Action::SwapTokensForExactTokens => FunctionModifier::NonPayable,
			}) {
				return Some(Err(err))
			}
			log::debug!(target: "exchange", "match --- log: {:?}", address);
			match selector {
				Action::CreatePool => Self::create_pool(handle),
				Action::AddLiquidity => Self::add_liquidity(handle),
				Action::RemoveLiquidity => Self::remove_liquidity(handle),
				Action::SwapExactTokensForTokens => Self::swap_exact_tokens_for_tokens(handle),
				Action::SwapTokensForExactTokens => Self::swap_tokens_for_exact_tokens(handle),
			}
		};
		Some(result)
	}
	fn is_precompile(&self, address: H160) -> bool {
		true
	}
}

impl<Runtime> ExchangeExtension<Runtime> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime> ExchangeExtension<Runtime>
where
	Runtime: pallet_evm::Config + pallet_exchange::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_exchange::Call<Runtime>>,
	<Runtime as pallet_token_fungible::Config>::FungibleTokenId: From<u128> + Into<u128>,
{
	fn create_pool(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(2)?;
		log::debug!(target: "exchange", "log: {:?}", "create_pool");
		let token_a: FungibleTokenIdOf<Runtime> = input.read::<u128>()?.into();
		let token_b: FungibleTokenIdOf<Runtime> = input.read::<u128>()?.into();
		log::debug!(target: "exchange", "log: {:?}", token_a);
		log::debug!(target: "exchange", "log: {:?}", token_b);
		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_exchange::Call::<Runtime>::create_pool { token_a,token_b  },
			)?;
		}
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn add_liquidity(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn remove_liquidity(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn swap_exact_tokens_for_tokens(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn swap_tokens_for_exact_tokens(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
}
