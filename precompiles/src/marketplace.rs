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
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::UniqueSaturatedFrom,
};
use pallet_evm::{AddressMapping, PrecompileSet};
use pallet_marketplace::{Asset, BalanceOf};
use precompile_utils::prelude::*;
use primitives::BlockNumber;
use sp_core::H160;
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

#[generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	CreateOrder = "create_order(uint256,uint256,uint256,uint256,uint256)",
	CancelOrder = "cancel_order(uint256,uint256,uint256)",
	ExecuteOrder = "execute_order(uint256,uint256,uint256)",
	PlaceBid = "place_bid(uint256,uint256,uint256,uint256,uint256)",
	CancelBid = "cancel_bid(uint256,uint256,uint256)",
	AcceptBid = "accept_bid(uint256,uint256,uint256)",
}

pub struct MarketplaceExtension<Runtime>(PhantomData<Runtime>);

impl<Runtime> PrecompileSet for MarketplaceExtension<Runtime>
where
	Runtime: pallet_marketplace::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_marketplace::Call<Runtime>>,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		let result = {
			let selector = match handle.read_selector() {
				Ok(selector) => selector,
				Err(e) => return Some(Err(e)),
			};
			if let Err(err) = handle.check_function_modifier(match selector {
				Action::CreateOrder |
				Action::CancelOrder |
				Action::ExecuteOrder |
				Action::PlaceBid |
				Action::CancelBid |
				Action::AcceptBid => FunctionModifier::NonPayable,
			}) {
				return Some(Err(err))
			}
			match selector {
				Action::CreateOrder => Self::create_order(handle),
				Action::CancelOrder => Self::cancel_order(handle),
				Action::ExecuteOrder => Self::execute_order(handle),
				Action::PlaceBid => Self::place_bid(handle),
				Action::CancelBid => Self::cancel_bid(handle),
				Action::AcceptBid => Self::accept_bid(handle),
			}
		};
		Some(result)
	}
	fn is_precompile(&self, _address: H160) -> bool {
		true
	}
}

impl<Runtime> MarketplaceExtension<Runtime> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime> MarketplaceExtension<Runtime>
where
	Runtime: pallet_marketplace::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_marketplace::Call<Runtime>>,
{
	fn create_order(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(5)?;

		let group_id = input.read::<u128>()?.into();
		let token_id = input.read::<u128>()?.into();
		let asset_type = input.read::<u128>()?.into();
		let price: u128 = input.read::<u128>()?.into();
		let duration = input.read::<BlockNumber>()?;

		let asset = match asset_type {
			0 => Asset::NonFungibleToken(group_id, token_id),
			1 => Asset::MultiToken(group_id, token_id),
			_ => Asset::ErrorToken,
		};
		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_marketplace::Call::<Runtime>::create_order {
					asset,
					price: BalanceOf::<Runtime>::unique_saturated_from(price),
					duration:<Runtime as frame_system::pallet::Config>::BlockNumber::unique_saturated_from(duration)
				},
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn cancel_order(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		let group_id = input.read::<u128>()?.into();
		let token_id = input.read::<u128>()?.into();
		let asset_type = input.read::<u128>()?.into();

		let asset = match asset_type {
			0 => Asset::NonFungibleToken(group_id, token_id),
			1 => Asset::MultiToken(group_id, token_id),
			_ => Asset::ErrorToken,
		};

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_marketplace::Call::<Runtime>::cancel_order { asset },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn execute_order(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		let group_id = input.read::<u128>()?.into();
		let token_id = input.read::<u128>()?.into();
		let asset_type = input.read::<u128>()?.into();

		let asset = match asset_type {
			0 => Asset::NonFungibleToken(group_id, token_id),
			1 => Asset::MultiToken(group_id, token_id),
			_ => Asset::ErrorToken,
		};

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_marketplace::Call::<Runtime>::execute_order { asset },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn place_bid(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(5)?;

		let group_id = input.read::<u128>()?.into();
		let token_id = input.read::<u128>()?.into();
		let asset_type = input.read::<u128>()?.into();
		let price: u128 = input.read::<u128>()?.into();
		let duration = input.read::<BlockNumber>()?;

		let asset = match asset_type {
			0 => Asset::NonFungibleToken(group_id, token_id),
			1 => Asset::MultiToken(group_id, token_id),
			_ => Asset::ErrorToken,
		};
		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_marketplace::Call::<Runtime>::place_bid {
					asset,
					price: BalanceOf::<Runtime>::unique_saturated_from(price),
					duration:<Runtime as frame_system::pallet::Config>::BlockNumber::unique_saturated_from(duration)
				},
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn cancel_bid(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		let group_id = input.read::<u128>()?.into();
		let token_id = input.read::<u128>()?.into();
		let asset_type = input.read::<u128>()?.into();

		let asset = match asset_type {
			0 => Asset::NonFungibleToken(group_id, token_id),
			1 => Asset::MultiToken(group_id, token_id),
			_ => Asset::ErrorToken,
		};

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_marketplace::Call::<Runtime>::cancel_bid { asset },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn accept_bid(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		let group_id = input.read::<u128>()?.into();
		let token_id = input.read::<u128>()?.into();
		let asset_type = input.read::<u128>()?.into();

		let asset = match asset_type {
			0 => Asset::NonFungibleToken(group_id, token_id),
			1 => Asset::MultiToken(group_id, token_id),
			_ => Asset::ErrorToken,
		};

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_marketplace::Call::<Runtime>::accept_bid { asset },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
}
