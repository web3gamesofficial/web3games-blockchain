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

use crate::{CREATE_SELECTOR, NFT_PRECOMPILE_ADDRESS_PREFIX};
use fp_evm::PrecompileOutput;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, PrecompileHandle};
use pallet_support::{
	AccountMapping, NonFungibleEnumerable, NonFungibleMetadata, TokenIdConversion,
};
use precompile_utils::prelude::*;
use primitives::{TokenId, TokenIndex};
use sp_core::{H160, U256};
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
// pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

pub type NonFungibleTokenIdOf<Runtime> =
	<Runtime as pallet_token_non_fungible::Config>::NonFungibleTokenId;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	BalanceOf = "balanceOf(address)",
	OwnerOf = "ownerOf(uint256)",
	TransferFrom = "transferFrom(address,address,uint256)",
	Mint = "mint(address,uint256)",
	Burn = "burn(uint256)",
	Name = "name()",
	Symbol = "symbol()",
	TokenURI = "tokenURI(uint256)",
	TotalSupply = "totalSupply()",
	TokenOfOwnerByIndex = "tokenOfOwnerByIndex(address,uint256)",
	TokenByIndex = "tokenByIndex(uint256)",
}

pub struct NonFungibleTokenExtension<Runtime>(PhantomData<Runtime>);

impl<Runtime> TokenIdConversion<NonFungibleTokenIdOf<Runtime>>
	for NonFungibleTokenExtension<Runtime>
where
	Runtime: pallet_token_non_fungible::Config + pallet_evm::Config,
	<Runtime as pallet_token_non_fungible::Config>::NonFungibleTokenId: From<u128> + Into<u128>,
{
	fn try_from_address(address: H160) -> Option<NonFungibleTokenIdOf<Runtime>> {
		let mut data = [0u8; 4];
		let prefix = &address.to_fixed_bytes()[0..4];
		let id = &address.to_fixed_bytes()[16..20];
		if prefix == NFT_PRECOMPILE_ADDRESS_PREFIX {
			data.copy_from_slice(id);
			let non_fungible_token_id: NonFungibleTokenIdOf<Runtime> =
				u32::from_be_bytes(data).into();
			Some(non_fungible_token_id)
		} else {
			None
		}
	}

	fn into_address(id: NonFungibleTokenIdOf<Runtime>) -> H160 {
		let id: u128 = id.into();
		let mut data = [0u8; 20];
		data[0..4].copy_from_slice(NFT_PRECOMPILE_ADDRESS_PREFIX);
		data[4..20].copy_from_slice(&id.to_be_bytes());
		H160::from_slice(&data)
	}
}

impl<Runtime> fp_evm::Precompile for NonFungibleTokenExtension<Runtime>
where
	Runtime: pallet_token_non_fungible::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_non_fungible::Call<Runtime>>,
	<Runtime as pallet_token_non_fungible::Config>::NonFungibleTokenId: From<u128> + Into<u128>,
	<Runtime as pallet_token_non_fungible::Config>::TokenId: From<u128> + Into<u128>,
	Runtime: AccountMapping<Runtime::AccountId>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let address = handle.code_address();
		let input = handle.input();
		if let Some(non_fungible_token_id) = Self::try_from_address(address) {
			if pallet_token_non_fungible::Pallet::<Runtime>::exists(non_fungible_token_id) {
				let selector = handle.read_selector()?;
				handle.check_function_modifier(match selector {
					Action::Name |
					Action::Symbol |
					Action::OwnerOf |
					Action::TotalSupply |
					Action::TokenURI |
					Action::TokenOfOwnerByIndex |
					Action::TokenByIndex |
					Action::BalanceOf => FunctionModifier::View,
					Action::TransferFrom | Action::Mint | Action::Burn =>
						FunctionModifier::NonPayable,
				})?;
				let (origin, call) = match selector {
					// storage getters
					Action::Name => return Self::name(non_fungible_token_id, handle),
					Action::Symbol => return Self::symbol(non_fungible_token_id, handle),
					Action::TokenURI => return Self::token_uri(non_fungible_token_id, handle),
					Action::TotalSupply => return Self::total_supply(non_fungible_token_id, handle),
					Action::TokenByIndex =>
						return Self::token_by_index(non_fungible_token_id, handle),
					Action::TokenOfOwnerByIndex =>
						return Self::token_of_owner_by_index(non_fungible_token_id, handle),
					Action::BalanceOf => return Self::balance_of(non_fungible_token_id, handle),
					Action::OwnerOf => return Self::owner_of(non_fungible_token_id, handle),
					// call methods (dispatchable)
					Action::TransferFrom => Self::transfer_from(non_fungible_token_id, handle)?,
					Action::Mint => Self::mint(non_fungible_token_id, handle)?,
					Action::Burn => Self::burn(non_fungible_token_id, handle)?,
				};
				// Dispatch call (if enough gas).
				RuntimeHelper::<Runtime>::try_dispatch(handle, origin, call)?;
			} else {
				if &input[0..4] == CREATE_SELECTOR {
					Self::create(handle)?;
				}
			}
		}
		Ok(succeed([]))
	}
}

impl<Runtime> NonFungibleTokenExtension<Runtime> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime> NonFungibleTokenExtension<Runtime>
where
	Runtime: pallet_token_non_fungible::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_non_fungible::Call<Runtime>>,
	<Runtime as pallet_token_non_fungible::Config>::NonFungibleTokenId: From<u128> + Into<u128>,
	<Runtime as pallet_token_non_fungible::Config>::TokenId: From<u128> + Into<u128>,
	Runtime: AccountMapping<Runtime::AccountId>,
{
	fn create(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_token_non_fungible::Call<Runtime>,
	)> {
		handle.record_log_costs_manual(3, 32)?;

		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(4)?;

		let id = input.read::<u128>()?.into();
		let name: Vec<u8> = input.read::<Bytes>()?.into();
		let symbol: Vec<u8> = input.read::<Bytes>()?.into();
		let base_uri: Vec<u8> = input.read::<Bytes>()?.into();

		let origin: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Dispatch call (if enough gas).
		let call =
			pallet_token_non_fungible::Call::<Runtime>::create_token { id, name, symbol, base_uri };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn balance_of(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(1)?;

		let owner: H160 = input.read::<Address>()?.into();

		let balance: U256 = {
			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			pallet_token_non_fungible::Pallet::<Runtime>::balance_of(id, owner).into()
		};

		Ok(succeed(EvmDataWriter::new().write(balance).build()))
	}

	fn owner_of(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(1)?;

		let token_id: Runtime::TokenId = input.read::<TokenId>()?.into();

		let owner_account_id: Runtime::AccountId =
			pallet_token_non_fungible::Pallet::<Runtime>::owner_of(id, token_id).unwrap();

		let owner = Runtime::into_evm_address(owner_account_id);

		Ok(succeed(EvmDataWriter::new().write::<Address>(owner.into()).build()))
	}

	fn transfer_from(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_token_non_fungible::Call<Runtime>,
	)> {
		handle.record_log_costs_manual(3, 32)?;

		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(3)?;
		let from: H160 = input.read::<Address>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let token_id = input.read::<TokenId>()?;

		let origin: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(handle.context().caller);
		let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
		let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
		let token_id: Runtime::TokenId = token_id.into();

		// Dispatch call (if enough gas).
		let call =
			pallet_token_non_fungible::Call::<Runtime>::transfer_from { id, from, to, token_id };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn mint(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_token_non_fungible::Call<Runtime>,
	)> {
		handle.record_log_costs_manual(3, 32)?;

		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(2)?;

		let to: H160 = input.read::<Address>()?.into();
		let token_id = input.read::<TokenId>()?;

		let origin: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(handle.context().caller);
		let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
		let token_id: Runtime::TokenId = token_id.into();

		// Dispatch call (if enough gas).
		let call = pallet_token_non_fungible::Call::<Runtime>::mint { id, to, token_id };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn burn(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_token_non_fungible::Call<Runtime>,
	)> {
		handle.record_log_costs_manual(3, 32)?;

		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(1)?;

		let token_id = input.read::<TokenId>()?;

		let origin: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(handle.context().caller);
		let token_id: Runtime::TokenId = token_id.into();

		// Dispatch call (if enough gas).
		let call = pallet_token_non_fungible::Call::<Runtime>::burn { id, token_id };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn name(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let name: Vec<u8> = pallet_token_non_fungible::Pallet::<Runtime>::token_name(id);

		Ok(succeed(EvmDataWriter::new().write(name).build()))
	}

	fn symbol(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let symbol: Vec<u8> = pallet_token_non_fungible::Pallet::<Runtime>::token_symbol(id);

		Ok(succeed(EvmDataWriter::new().write(symbol).build()))
	}

	fn token_uri(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(1)?;

		let token_id: Runtime::TokenId = input.read::<TokenId>()?.into();

		let token_uri: Vec<u8> =
			pallet_token_non_fungible::Pallet::<Runtime>::token_uri(id, token_id);

		Ok(succeed(EvmDataWriter::new().write(token_uri).build()))
	}

	fn total_supply(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let balance: u32 = pallet_token_non_fungible::Pallet::<Runtime>::total_supply(id);

		Ok(succeed(EvmDataWriter::new().write(balance).build()))
	}

	fn token_by_index(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(1)?;

		let token_index = input.read::<TokenIndex>()?.into();

		let token_id: TokenId =
			pallet_token_non_fungible::Pallet::<Runtime>::token_by_index(id, token_index).into();

		Ok(succeed(EvmDataWriter::new().write(token_id).build()))
	}

	fn token_of_owner_by_index(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(2)?;

		let owner: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		let token_index = input.read::<TokenIndex>()?.into();

		let token_id: TokenId =
			pallet_token_non_fungible::Pallet::<Runtime>::token_of_owner_by_index(
				id,
				owner,
				token_index,
			)
			.into();

		Ok(succeed(EvmDataWriter::new().write(token_id).build()))
	}
}
