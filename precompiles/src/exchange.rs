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

pub type FungibleTokenIdOf<Runtime> = <Runtime as pallet_token_fungible::Config>::FungibleTokenId;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	CreatePool = "create_pool(uint256,uint256)",
	AddLiquidity = "add_liquidity(uint256,uint256,uint256,uint256,uint256,uint256,address,uint256)",
	AddLiquidityW3G = "add_liquidity_w3g(uint256,uint256,uint256,uint256,uint256,address,uint256)",
	RemoveLiquidity = "remove_liquidity(uint256,uint256,uint256,uint256,uint256,address,uint256)",
	RemoveLiquidityW3G = "remove_liquidity_w3g(uint256,uint256,uint256,uint256,address,uint256)",
	SwapExactTokensForTokens =
		"swap_exact_tokens_for_tokens(uint256,uint256,uint256[],address,uint256)",
	SwapExactW3GForTokens = "swap_exact_w3g_for_tokens(uint256,uint256,uint256[],address,uint256)",
	SwapTokensForExactTokens =
		"swap_tokens_for_exact_tokens(uint256,uint256,uint256[],address,uint256)",
	SwapTokensForExactW3G = "swap_tokens_for_exact_w3g(uint256,uint256,uint256[],address,uint256)",
}

pub struct ExchangeExtension<Runtime>(PhantomData<Runtime>);

impl<Runtime> PrecompileSet for ExchangeExtension<Runtime>
where
	Runtime: pallet_evm::Config + pallet_exchange::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_exchange::Call<Runtime>>,
	<Runtime as pallet_token_fungible::Config>::FungibleTokenId: From<u128> + Into<u128>,
	<Runtime as pallet_exchange::Config>::PoolId: From<u128> + Into<u128>,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		let result = {
			let selector = match handle.read_selector() {
				Ok(selector) => selector,
				Err(e) => return Some(Err(e)),
			};
			if let Err(err) = handle.check_function_modifier(match selector {
				Action::CreatePool |
				Action::AddLiquidity |
				Action::AddLiquidityW3G |
				Action::RemoveLiquidity |
				Action::RemoveLiquidityW3G |
				Action::SwapExactTokensForTokens |
				Action::SwapExactW3GForTokens |
				Action::SwapTokensForExactTokens |
				Action::SwapTokensForExactW3G => FunctionModifier::NonPayable,
			}) {
				return Some(Err(err))
			}
			match selector {
				Action::CreatePool => Self::create_pool(handle),
				Action::AddLiquidity => Self::add_liquidity(handle),
				Action::AddLiquidityW3G => Self::add_liquidity_w3g(handle),
				Action::RemoveLiquidity => Self::remove_liquidity(handle),
				Action::RemoveLiquidityW3G => Self::remove_liquidity_w3g(handle),
				Action::SwapExactTokensForTokens => Self::swap_exact_tokens_for_tokens(handle),
				Action::SwapExactW3GForTokens => Self::swap_exact_w3g_for_tokens(handle),
				Action::SwapTokensForExactTokens => Self::swap_tokens_for_exact_tokens(handle),
				Action::SwapTokensForExactW3G => Self::swap_tokens_for_exact_w3g(handle),
			}
		};
		Some(result)
	}
	fn is_precompile(&self, _address: H160) -> bool {
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
	<Runtime as pallet_exchange::Config>::PoolId: From<u128> + Into<u128>,
{
	fn create_pool(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(2)?;
		let token_a: FungibleTokenIdOf<Runtime> = input.read::<u128>()?.into();
		let token_b: FungibleTokenIdOf<Runtime> = input.read::<u128>()?.into();
		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_exchange::Call::<Runtime>::create_pool { token_a, token_b },
			)?;
		}
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn add_liquidity(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(8)?;
		let token_a: FungibleTokenIdOf<Runtime> = input.read::<u128>()?.into();
		let token_b: FungibleTokenIdOf<Runtime> = input.read::<u128>()?.into();
		let amount_a_desired: Balance = input.read::<u128>()?.into();
		let amount_b_desired: Balance = input.read::<u128>()?.into();
		let amount_a_min: Balance = input.read::<u128>()?.into();
		let amount_b_min: Balance = input.read::<u128>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let deadline = input.read::<BlockNumber>()?.into();
		let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_exchange::Call::<Runtime>::add_liquidity {
					token_a,
					token_b,
					amount_a_desired,
					amount_b_desired,
					amount_a_min,
					amount_b_min,
					to,
					deadline,
				},
			)?;
		}

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn add_liquidity_w3g(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(7)?;
		let token: FungibleTokenIdOf<Runtime> = input.read::<u128>()?.into();
		let amount_w3g_desired: Balance = input.read::<u128>()?.into();
		let amount_desired: Balance = input.read::<u128>()?.into();
		let amount_w3g_min: Balance = input.read::<u128>()?.into();
		let amount_min: Balance = input.read::<u128>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let deadline = input.read::<BlockNumber>()?.into();
		let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_exchange::Call::<Runtime>::add_liquidity_w3g {
					token,
					amount_w3g_desired,
					amount_desired,
					amount_w3g_min,
					amount_min,
					to,
					deadline,
				},
			)?;
		}

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn remove_liquidity(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(7)?;
		let token_a: FungibleTokenIdOf<Runtime> = input.read::<u128>()?.into();
		let token_b: FungibleTokenIdOf<Runtime> = input.read::<u128>()?.into();
		let liquidity: Balance = input.read::<u128>()?.into();
		let amount_a_min: Balance = input.read::<u128>()?.into();
		let amount_b_min: Balance = input.read::<u128>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let deadline = input.read::<BlockNumber>()?.into();
		let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_exchange::Call::<Runtime>::remove_liquidity {
					token_a,
					token_b,
					liquidity,
					amount_a_min,
					amount_b_min,
					to,
					deadline,
				},
			)?;
		}

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn remove_liquidity_w3g(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(6)?;
		let token: FungibleTokenIdOf<Runtime> = input.read::<u128>()?.into();
		let liquidity: Balance = input.read::<u128>()?.into();
		let amount_w3g_min: Balance = input.read::<u128>()?.into();
		let amount_min: Balance = input.read::<u128>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let deadline = input.read::<BlockNumber>()?.into();
		let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_exchange::Call::<Runtime>::remove_liquidity_w3g {
					token,
					liquidity,
					amount_w3g_min,
					amount_min,
					to,
					deadline,
				},
			)?;
		}

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn swap_exact_tokens_for_tokens(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(5)?;
		let amount_in: Balance = input.read::<u128>()?.into();
		let amount_out_min: Balance = input.read::<u128>()?.into();
		let u128_path = input.read::<Vec<u128>>()?;
		let mut path: Vec<FungibleTokenIdOf<Runtime>> = vec![];
		for i in 0..u128_path.len() {
			path.push(u128_path[i].into())
		}
		let to: H160 = input.read::<Address>()?.into();
		let deadline = input.read::<BlockNumber>()?.into();
		let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_exchange::Call::<Runtime>::swap_exact_tokens_for_tokens {
					amount_in,
					amount_out_min,
					path,
					to,
					deadline,
				},
			)?;
		}

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn swap_exact_w3g_for_tokens(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(5)?;
		let amount_in_w3g: Balance = input.read::<u128>()?.into();
		let amount_out_min: Balance = input.read::<u128>()?.into();
		let u128_path = input.read::<Vec<u128>>()?;
		let mut path: Vec<FungibleTokenIdOf<Runtime>> = vec![];
		for i in 0..u128_path.len() {
			path.push(u128_path[i].into())
		}
		let to: H160 = input.read::<Address>()?.into();
		let deadline = input.read::<BlockNumber>()?.into();
		let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_exchange::Call::<Runtime>::swap_exact_w3g_for_tokens {
					amount_in_w3g,
					amount_out_min,
					path,
					to,
					deadline,
				},
			)?;
		}

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn swap_tokens_for_exact_tokens(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(5)?;
		let amount_out: Balance = input.read::<u128>()?.into();
		let amount_in_max: Balance = input.read::<u128>()?.into();
		let u128_path = input.read::<Vec<u128>>()?;
		let mut path: Vec<FungibleTokenIdOf<Runtime>> = vec![];
		for i in 0..u128_path.len() {
			path.push(u128_path[i].into())
		}
		let to: H160 = input.read::<Address>()?.into();
		let deadline = input.read::<BlockNumber>()?.into();
		let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_exchange::Call::<Runtime>::swap_tokens_for_exact_tokens {
					amount_out,
					amount_in_max,
					path,
					to,
					deadline,
				},
			)?;
		}

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
	fn swap_tokens_for_exact_w3g(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(5)?;
		let amount_out_w3g: Balance = input.read::<u128>()?.into();
		let amount_in_max: Balance = input.read::<u128>()?.into();
		let u128_path = input.read::<Vec<u128>>()?;
		let mut path: Vec<FungibleTokenIdOf<Runtime>> = vec![];
		for i in 0..u128_path.len() {
			path.push(u128_path[i].into())
		}
		let to: H160 = input.read::<Address>()?.into();
		let deadline = input.read::<BlockNumber>()?.into();
		let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_exchange::Call::<Runtime>::swap_tokens_for_exact_w3g {
					amount_out_w3g,
					amount_in_max,
					path,
					to,
					deadline,
				},
			)?;
		}

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
}
