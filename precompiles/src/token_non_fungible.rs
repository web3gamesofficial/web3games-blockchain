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

use crate::{NFT_PRECOMPILE_ADDRESS_PREFIX, TOKEN_NON_FUNGIBLE_CREATE_SELECTOR};
use fp_evm::PrecompileOutput;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, PrecompileHandle, PrecompileSet};
use precompile_utils::prelude::*;
use primitives::{TokenId, TokenIndex};
use sp_core::{H160, U256};
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};
use web3games_support::{
	AccountMapping, NonFungibleEnumerable, NonFungibleMetadata, TokenIdConversion,
};

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
// pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

pub type NonFungibleTokenIdOf<Runtime> =
	<Runtime as web3games_token_non_fungible::Config>::NonFungibleTokenId;

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
	Approve = "approve(address,uint256)",
}

pub struct NonFungibleTokenExtension<Runtime>(PhantomData<Runtime>);

impl<Runtime> TokenIdConversion<NonFungibleTokenIdOf<Runtime>>
	for NonFungibleTokenExtension<Runtime>
where
	Runtime: web3games_token_non_fungible::Config + pallet_evm::Config,
	<Runtime as web3games_token_non_fungible::Config>::NonFungibleTokenId: From<u128> + Into<u128>,
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

impl<Runtime> PrecompileSet for NonFungibleTokenExtension<Runtime>
where
	Runtime: web3games_token_non_fungible::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<web3games_token_non_fungible::Call<Runtime>>,
	<Runtime as web3games_token_non_fungible::Config>::NonFungibleTokenId: From<u128> + Into<u128>,
	<Runtime as web3games_token_non_fungible::Config>::TokenId: From<u128> + Into<u128>,
	Runtime: AccountMapping<Runtime::AccountId>,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		let address = handle.code_address();
		let input = handle.input();
		if let Some(non_fungible_token_id) = Self::try_from_address(address) {
			if web3games_token_non_fungible::Pallet::<Runtime>::exists(non_fungible_token_id) {
				let result = {
					let selector = match handle.read_selector() {
						Ok(selector) => selector,
						Err(e) => return Some(Err(e)),
					};
					if let Err(err) = handle.check_function_modifier(match selector {
						Action::Name |
						Action::Symbol |
						Action::OwnerOf |
						Action::TotalSupply |
						Action::TokenURI |
						Action::TokenOfOwnerByIndex |
						Action::TokenByIndex |
						Action::BalanceOf => FunctionModifier::View,
						Action::TransferFrom | Action::Mint | Action::Burn | Action::Approve =>
							FunctionModifier::NonPayable,
					}) {
						return Some(Err(err))
					}
					match selector {
						// storage getters
						Action::Name => Self::name(non_fungible_token_id, handle),
						Action::Symbol => Self::symbol(non_fungible_token_id, handle),
						Action::TokenURI => Self::token_uri(non_fungible_token_id, handle),
						Action::TotalSupply => Self::total_supply(non_fungible_token_id, handle),
						Action::TokenByIndex => Self::token_by_index(non_fungible_token_id, handle),
						Action::TokenOfOwnerByIndex =>
							Self::token_of_owner_by_index(non_fungible_token_id, handle),
						Action::BalanceOf => Self::balance_of(non_fungible_token_id, handle),
						Action::OwnerOf => Self::owner_of(non_fungible_token_id, handle),
						// call methods (dispatchable)
						Action::TransferFrom => Self::transfer_from(non_fungible_token_id, handle),
						Action::Mint => Self::mint(non_fungible_token_id, handle),
						Action::Burn => Self::burn(non_fungible_token_id, handle),
						Action::Approve => Self::approve(non_fungible_token_id, handle),
					}
				};
				return Some(result)
			} else {
				if &input[0..4] == TOKEN_NON_FUNGIBLE_CREATE_SELECTOR {
					let result = Self::create(non_fungible_token_id, handle);
					return Some(result)
				}
			}
		}
		None
	}
	fn is_precompile(&self, address: H160) -> bool {
		if let Some(non_fungible_token_id) = Self::try_from_address(address) {
			web3games_token_non_fungible::Pallet::<Runtime>::exists(non_fungible_token_id)
		} else {
			false
		}
	}
}

impl<Runtime> NonFungibleTokenExtension<Runtime> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime> NonFungibleTokenExtension<Runtime>
where
	Runtime: web3games_token_non_fungible::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<web3games_token_non_fungible::Call<Runtime>>,
	<Runtime as web3games_token_non_fungible::Config>::NonFungibleTokenId: From<u128> + Into<u128>,
	<Runtime as web3games_token_non_fungible::Config>::TokenId: From<u128> + Into<u128>,
	Runtime: AccountMapping<Runtime::AccountId>,
{
	fn create(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(3)?;

		let name: Vec<u8> = input.read::<Bytes>()?.into();
		let symbol: Vec<u8> = input.read::<Bytes>()?.into();
		let base_uri: Vec<u8> = input.read::<Bytes>()?.into();

		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				web3games_token_non_fungible::Call::<Runtime>::create_token {
					id,
					name,
					symbol,
					base_uri,
				},
			)?;
		}
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn balance_of(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(1)?;

		let address = input.read::<Address>()?.0;
		let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(address);

		let balance: U256 =
			web3games_token_non_fungible::Pallet::<Runtime>::balance_of(id, owner).into();

		Ok(succeed(EvmDataWriter::new().write(balance).build()))
	}

	fn owner_of(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(1)?;

		let token_id: Runtime::TokenId = input.read::<TokenId>()?.into();

		let owner_account_id: Runtime::AccountId =
			web3games_token_non_fungible::Pallet::<Runtime>::owner_of(id, token_id).unwrap();
		let owner = Runtime::into_evm_address(owner_account_id);

		Ok(succeed(EvmDataWriter::new().write::<Address>(owner.into()).build()))
	}

	fn approve(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let spender: H160 = input.read::<Address>()?.into();
		let token_id = input.read::<TokenId>()?.into();

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				web3games_token_non_fungible::Call::<Runtime>::approve { id, to, token_id },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn transfer_from(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(3)?;
		let from: H160 = input.read::<Address>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let token_id = input.read::<TokenId>()?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			let token_id: Runtime::TokenId = token_id.into();

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				web3games_token_non_fungible::Call::<Runtime>::transfer_from {
					id,
					from,
					to,
					token_id,
				},
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn mint(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(2)?;

		let to: H160 = input.read::<Address>()?.into();
		let token_id = input.read::<TokenId>()?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			let token_id: Runtime::TokenId = token_id.into();

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				web3games_token_non_fungible::Call::<Runtime>::mint { id, to, token_id },
			)?;
		}
		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn burn(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(1)?;

		let token_id = input.read::<TokenId>()?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let token_id: Runtime::TokenId = token_id.into();
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				web3games_token_non_fungible::Call::<Runtime>::burn { id, token_id },
			)?;
		}
		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn name(
		id: NonFungibleTokenIdOf<Runtime>,
		_handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let name = web3games_token_non_fungible::Pallet::<Runtime>::token_name(id);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write::<Bytes>(name.as_slice().into()).build()))
	}

	fn symbol(
		id: NonFungibleTokenIdOf<Runtime>,
		_handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let symbol = web3games_token_non_fungible::Pallet::<Runtime>::token_symbol(id);
		// Build output.
		Ok(succeed(EvmDataWriter::new().write::<Bytes>(symbol.as_slice().into()).build()))
	}

	fn token_uri(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(1)?;

		let token_id: Runtime::TokenId = input.read::<TokenId>()?.into();

		let token_uri: Vec<u8> =
			web3games_token_non_fungible::Pallet::<Runtime>::token_uri(id, token_id);

		Ok(succeed(EvmDataWriter::new().write::<Bytes>(token_uri.as_slice().into()).build()))
	}

	fn total_supply(
		id: NonFungibleTokenIdOf<Runtime>,
		_handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let balance: u32 = web3games_token_non_fungible::Pallet::<Runtime>::total_supply(id);

		Ok(succeed(EvmDataWriter::new().write(balance).build()))
	}

	fn token_by_index(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(1)?;

		let token_index = input.read::<TokenIndex>()?.into();

		let token_id: TokenId =
			web3games_token_non_fungible::Pallet::<Runtime>::token_by_index(id, token_index).into();

		Ok(succeed(EvmDataWriter::new().write(token_id).build()))
	}

	fn token_of_owner_by_index(
		id: NonFungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(2)?;

		let owner: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		let token_index = input.read::<TokenIndex>()?.into();

		let token_id: TokenId =
			web3games_token_non_fungible::Pallet::<Runtime>::token_of_owner_by_index(
				id,
				owner,
				token_index,
			)
			.into();

		Ok(succeed(EvmDataWriter::new().write(token_id).build()))
	}
}
