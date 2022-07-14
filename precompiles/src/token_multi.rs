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

use crate::{CREATE_SELECTOR, MT_PRECOMPILE_ADDRESS_PREFIX};
use fp_evm::{PrecompileHandle, PrecompileOutput, PrecompileSet};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use pallet_support::{MultiMetadata, TokenIdConversion};
use precompile_utils::prelude::*;
use primitives::{Balance, TokenId};
use sp_core::H160;
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

pub type MultiTokenIdOf<Runtime> = <Runtime as pallet_token_multi::Config>::MultiTokenId;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	BalanceOf = "balanceOf(address,uint256)",
	BalanceOfBatch = "balanceOfBatch(address[],uint256[])",
	TransferFrom = "transferFrom(address,address,uint256,uint256)",
	BatchTransferFrom = "batchTransferFrom(address,address,uint256[],uint256[])",
	Mint = "mint(address,uint256,uint256)",
	MintBatch = "mintBatch(address,uint256[],uint256[])",
	Burn = "burn(uint256,uint256)",
	BurnBatch = "burnBatch(uint256[],uint256[])",
	URI = "uri(uint256)",
}
pub struct MultiTokenExtension<Runtime>(PhantomData<Runtime>);

impl<Runtime> TokenIdConversion<MultiTokenIdOf<Runtime>> for MultiTokenExtension<Runtime>
where
	Runtime: pallet_token_multi::Config + pallet_evm::Config,
	<Runtime as pallet_token_multi::Config>::MultiTokenId: From<u128> + Into<u128>,
{
	fn try_from_address(address: H160) -> Option<MultiTokenIdOf<Runtime>> {
		let mut data = [0u8; 4];
		let prefix = &address.to_fixed_bytes()[0..4];
		let id = &address.to_fixed_bytes()[16..20];
		if prefix == MT_PRECOMPILE_ADDRESS_PREFIX {
			data.copy_from_slice(id);
			let multi_token_id: MultiTokenIdOf<Runtime> = u32::from_be_bytes(data).into();
			Some(multi_token_id)
		} else {
			None
		}
	}

	fn into_address(id: MultiTokenIdOf<Runtime>) -> H160 {
		let id: u128 = id.into();
		let mut data = [0u8; 20];
		data[0..4].copy_from_slice(MT_PRECOMPILE_ADDRESS_PREFIX);
		data[4..20].copy_from_slice(&id.to_be_bytes());
		H160::from_slice(&data)
	}
}

impl<Runtime> PrecompileSet for MultiTokenExtension<Runtime>
where
	Runtime: pallet_token_multi::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_multi::Call<Runtime>>,
	<Runtime as pallet_token_multi::Config>::MultiTokenId: From<u128> + Into<u128>,
	<Runtime as pallet_token_multi::Config>::TokenId: From<u128> + Into<u128>,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		let address = handle.code_address();
		let input = handle.input();
		if let Some(multi_token_id) = Self::try_from_address(address) {
			if pallet_token_multi::Pallet::<Runtime>::exists(multi_token_id) {
				let result = {
					let selector = match handle.read_selector() {
						Ok(selector) => selector,
						Err(e) => return Some(Err(e)),
					};
					if let Err(err) = handle.check_function_modifier(match selector {
						Action::URI | Action::BalanceOfBatch | Action::BalanceOf =>
							FunctionModifier::View,
						Action::TransferFrom |
						Action::BatchTransferFrom |
						Action::Mint |
						Action::MintBatch |
						Action::Burn |
						Action::BurnBatch => FunctionModifier::NonPayable,
					}) {
						return Some(Err(err))
					}
					match selector {
						// storage getters
						Action::BalanceOf => Self::balance_of(multi_token_id, handle),
						Action::BalanceOfBatch => Self::balance_of_batch(multi_token_id, handle),
						Action::URI => Self::uri(multi_token_id, handle),
						// runtime methods (dispatchable)
						Action::TransferFrom => Self::transfer_from(multi_token_id, handle),
						Action::BatchTransferFrom =>
							Self::batch_transfer_from(multi_token_id, handle),
						Action::Mint => Self::mint(multi_token_id, handle),
						Action::MintBatch => Self::mint_batch(multi_token_id, handle),
						Action::Burn => Self::burn(multi_token_id, handle),
						Action::BurnBatch => Self::burn_batch(multi_token_id, handle),
					}
				};
				return Some(result)
			} else {
				if &input[0..4] == CREATE_SELECTOR {
					let result = Self::create(handle);
					return Some(result)
				}
			}
		}
		None
	}
	fn is_precompile(&self, address: H160) -> bool {
		if let Some(multi_token_id) = Self::try_from_address(address) {
			pallet_token_multi::Pallet::<Runtime>::exists(multi_token_id)
		} else {
			false
		}
	}
}

impl<Runtime> MultiTokenExtension<Runtime> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime> MultiTokenExtension<Runtime>
where
	Runtime: pallet_token_multi::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_multi::Call<Runtime>>,
	<Runtime as pallet_token_multi::Config>::MultiTokenId: From<u128> + Into<u128>,
	<Runtime as pallet_token_multi::Config>::TokenId: From<u128> + Into<u128>,
{
	fn create(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(2)?;

		let id = input.read::<u128>()?.into();
		let uri: Vec<u8> = input.read::<Bytes>()?.into();

		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_token_multi::Call::<Runtime>::create_token { id, uri },
			)?;
		}
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn balance_of(
		id: MultiTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(2)?;

		let account: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let token_id: Runtime::TokenId = input.read::<TokenId>()?.into();

		let balance: Balance =
			pallet_token_multi::Pallet::<Runtime>::balance_of(id, (token_id, &account));

		Ok(succeed(EvmDataWriter::new().write(balance).build()))
	}

	fn balance_of_batch(
		id: MultiTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(2)?;

		let accounts: Vec<Runtime::AccountId> = input
			.read::<Vec<Address>>()?
			.iter()
			.map(|&a| Runtime::AddressMapping::into_account_id(a.0))
			.collect();
		let token_ids: Vec<Runtime::TokenId> = input
			.read::<Vec<TokenId>>()?
			.iter()
			.map(|&a| Runtime::TokenId::from(a))
			.collect();

		let balances: Vec<Balance> =
			pallet_token_multi::Pallet::<Runtime>::balance_of_batch(id, &accounts, token_ids)
				.unwrap();

		Ok(succeed(EvmDataWriter::new().write::<Vec<Balance>>(balances.as_slice().into()).build()))
	}

	fn transfer_from(
		id: MultiTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(4)?;

		let from: H160 = input.read::<Address>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let token_id: Runtime::TokenId = input.read::<TokenId>()?.into();
		let amount = input.read::<Balance>()?;

		{
			// Build call with origin.
			let origin: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_token_multi::Call::<Runtime>::transfer_from {
					id,
					from,
					to,
					token_id,
					amount,
				},
			)?;
		}
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn batch_transfer_from(
		id: MultiTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(4)?;

		let from: H160 = input.read::<Address>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let token_ids: Vec<Runtime::TokenId> = input
			.read::<Vec<TokenId>>()?
			.iter()
			.map(|&a| Runtime::TokenId::from(a))
			.collect();
		let amounts = input.read::<Vec<Balance>>()?;

		{
			// Build call with origin.
			let origin: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_token_multi::Call::<Runtime>::batch_transfer_from {
					id,
					from,
					to,
					token_ids,
					amounts,
				},
			)?;
		}
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn mint(
		id: MultiTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_log_costs_manual(3, 32)?;

		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(3)?;

		let to: H160 = input.read::<Address>()?.into();
		let token_id: Runtime::TokenId = input.read::<TokenId>()?.into();
		let amount = input.read::<Balance>()?;

		{
			// Build call with origin.
			let origin: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_token_multi::Call::<Runtime>::mint { id, to, token_id, amount },
			)?;
		}
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn mint_batch(
		id: MultiTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(3)?;

		let to: H160 = input.read::<Address>()?.into();
		let token_ids: Vec<Runtime::TokenId> = input
			.read::<Vec<TokenId>>()?
			.iter()
			.map(|&a| Runtime::TokenId::from(a))
			.collect();
		let amounts = input.read::<Vec<Balance>>()?;

		{
			// Build call with origin.
			let origin: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_token_multi::Call::<Runtime>::mint_batch { id, to, token_ids, amounts },
			)?;
		}
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn burn(
		id: MultiTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(2)?;

		let token_id: Runtime::TokenId = input.read::<TokenId>()?.into();
		let amount = input.read::<Balance>()?;

		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_token_multi::Call::<Runtime>::burn { id, token_id, amount },
			)?;
		}
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn burn_batch(
		id: MultiTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(2)?;

		let token_ids: Vec<Runtime::TokenId> = input
			.read::<Vec<TokenId>>()?
			.iter()
			.map(|&a| Runtime::TokenId::from(a))
			.collect();
		let amounts = input.read::<Vec<Balance>>()?;

		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_token_multi::Call::<Runtime>::burn_batch { id, token_ids, amounts },
			)?;
		}
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn uri(
		id: MultiTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(1)?;

		let token_id: Runtime::TokenId = input.read::<TokenId>()?.into();

		let uri = pallet_token_multi::Pallet::<Runtime>::uri(id, token_id);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write::<Bytes>(uri.as_slice().into()).build()))
	}
}
