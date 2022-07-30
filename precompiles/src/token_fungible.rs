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

pub type FungibleTokenIdOf<Runtime> = <Runtime as pallet_token_fungible::Config>::FungibleTokenId;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	Name = "name()",
	Symbol = "symbol()",
	Decimals = "decimals()",
	TotalSupply = "totalSupply()",
	BalanceOf = "balanceOf(address)",
	Allowance = "allowance(address,address)",
	Transfer = "transfer(address,uint256)",
	TransferFrom = "transferFrom(address,address,uint256)",
	Mint = "mint(address,uint256)",
	Burn = "burn(uint256)",
	Approve = "approve(address,uint256)",
}

pub struct FungibleTokenExtension<Runtime>(PhantomData<Runtime>);

impl<Runtime> TokenIdConversion<FungibleTokenIdOf<Runtime>> for FungibleTokenExtension<Runtime>
where
	Runtime: pallet_token_fungible::Config + pallet_evm::Config,
	<Runtime as pallet_token_fungible::Config>::FungibleTokenId: From<u128> + Into<u128>,
{
	fn try_from_address(address: H160) -> Option<FungibleTokenIdOf<Runtime>> {
		let mut data = [0u8; 4];
		let prefix = &address.to_fixed_bytes()[0..4];
		let id = &address.to_fixed_bytes()[16..20];
		if prefix == FT_PRECOMPILE_ADDRESS_PREFIX {
			data.copy_from_slice(id);
			let fungible_token_id: FungibleTokenIdOf<Runtime> = u32::from_be_bytes(data).into();
			Some(fungible_token_id)
		} else {
			None
		}
	}

	fn into_address(id: FungibleTokenIdOf<Runtime>) -> H160 {
		let id: u128 = id.into();
		let mut data = [0u8; 20];
		data[0..4].copy_from_slice(FT_PRECOMPILE_ADDRESS_PREFIX);
		data[4..20].copy_from_slice(&id.to_be_bytes());
		H160::from_slice(&data)
	}
}

impl<Runtime> PrecompileSet for FungibleTokenExtension<Runtime>
where
	Runtime: pallet_token_fungible::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_fungible::Call<Runtime>>,
	<Runtime as pallet_token_fungible::Config>::FungibleTokenId: From<u128> + Into<u128>,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		let address = handle.code_address();
		let input = handle.input();
		if let Some(fungible_token_id) = Self::try_from_address(address) {
			if pallet_token_fungible::Pallet::<Runtime>::exists(fungible_token_id) {
				let result = {
					let selector = match handle.read_selector() {
						Ok(selector) => selector,
						Err(e) => return Some(Err(e)),
					};
					if let Err(err) = handle.check_function_modifier(match selector {
						Action::Name |
						Action::Symbol |
						Action::Decimals |
						Action::TotalSupply |
						Action::Allowance |
						Action::BalanceOf => FunctionModifier::View,
						Action::Transfer |
						Action::TransferFrom |
						Action::Mint |
						Action::Burn |
						Action::Approve => FunctionModifier::NonPayable,
					}) {
						return Some(Err(err))
					}
					match selector {
						// XC20
						Action::TotalSupply => Self::total_supply(fungible_token_id, handle),
						Action::BalanceOf => Self::balance_of(fungible_token_id, handle),
						// Action::Allowance => Self::allowance(fungible_token_id, handle),
						// Action::Approve => Self::approve(fungible_token_id, handle),
						Action::Name => Self::name(fungible_token_id, handle),
						Action::Symbol => Self::symbol(fungible_token_id, handle),
						Action::Decimals => Self::decimals(fungible_token_id, handle),
						Action::Allowance => Self::allowance(fungible_token_id, handle),
						Action::Mint => Self::mint(fungible_token_id, handle),
						Action::Burn => Self::burn(fungible_token_id, handle),
						Action::Transfer => Self::transfer(fungible_token_id, handle),
						Action::TransferFrom => Self::transfer_from(fungible_token_id, handle),
						Action::Approve => Self::approve(fungible_token_id, handle),
					}
				};
				return Some(result)
			} else {
				if &input[0..4] == TOKEN_FUNGIBLE_CREATE_SELECTOR {
					let result = Self::create(fungible_token_id, handle);
					return Some(result)
				}
			}
		}
		None
	}
	fn is_precompile(&self, address: H160) -> bool {
		if let Some(fungible_token_id) = Self::try_from_address(address) {
			pallet_token_fungible::Pallet::<Runtime>::exists(fungible_token_id)
		} else {
			false
		}
	}
}

impl<Runtime> FungibleTokenExtension<Runtime> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime> FungibleTokenExtension<Runtime>
where
	Runtime: pallet_token_fungible::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_fungible::Call<Runtime>>,
	<Runtime as pallet_token_fungible::Config>::FungibleTokenId: From<u128> + Into<u128>,
{
	fn create(
		id: FungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		input.expect_arguments(3)?;
		let name: Vec<u8> = input.read::<Bytes>()?.into();
		let symbol: Vec<u8> = input.read::<Bytes>()?.into();
		let decimals = input.read::<u8>()?.into();
		{
			// Build call with origin.
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_token_fungible::Call::<Runtime>::create_token { id, name, symbol, decimals },
			)?;
		}
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn total_supply(
		id: FungibleTokenIdOf<Runtime>,
		_handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		// Fetch info.
		let amount: Balance = pallet_token_fungible::Pallet::<Runtime>::total_supply(id);

		Ok(succeed(EvmDataWriter::new().write(amount).build()))
	}

	fn balance_of(
		id: FungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let address = input.read::<Address>()?.0;

		let account: Runtime::AccountId = Runtime::AddressMapping::into_account_id(address);

		let balance: Balance = pallet_token_fungible::Pallet::<Runtime>::balance_of(id, account);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(balance).build()))
	}

	fn allowance(
		id: FungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(2)?;
		let owner = input.read::<Address>()?.0;
		let spender = input.read::<Address>()?.0;

		let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
		let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

		let balance: Balance =
			pallet_token_fungible::Pallet::<Runtime>::allowances(id, (owner, spender));

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(balance).build()))
	}

	fn approve(
		id: FungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let spender: H160 = input.read::<Address>()?.into();
		let amount = input.read::<Balance>()?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_token_fungible::Call::<Runtime>::approve { id, spender, amount },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn transfer(
		id: FungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let to: H160 = input.read::<Address>()?.into();
		let amount = input.read::<Balance>()?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_token_fungible::Call::<Runtime>::transfer { id, recipient: to, amount },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn transfer_from(
		id: FungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		let from: H160 = input.read::<Address>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let amount = input.read::<Balance>()?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_token_fungible::Call::<Runtime>::transfer_from {
					id,
					sender: from,
					recipient: to,
					amount,
				},
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn mint(
		id: FungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let to: H160 = input.read::<Address>()?.into();
		let amount = input.read::<Balance>()?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_token_fungible::Call::<Runtime>::mint { id, account: to, amount },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn burn(
		id: FungibleTokenIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(1)?;

		let amount = input.read::<Balance>()?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(caller).into(),
				pallet_token_fungible::Call::<Runtime>::burn { id, amount },
			)?;
		}

		// Return call information
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn name(
		id: FungibleTokenIdOf<Runtime>,
		_handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		// handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let name = pallet_token_fungible::Pallet::<Runtime>::token_name(id);
		// Build output.
		Ok(succeed(EvmDataWriter::new().write::<Bytes>(name.as_slice().into()).build()))
	}

	fn symbol(
		id: FungibleTokenIdOf<Runtime>,
		_handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let symbol = pallet_token_fungible::Pallet::<Runtime>::token_symbol(id);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write::<Bytes>(symbol.as_slice().into()).build()))
	}

	fn decimals(
		id: FungibleTokenIdOf<Runtime>,
		_handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let decimals: u8 = pallet_token_fungible::Pallet::<Runtime>::token_decimals(id);
		// Build output.
		Ok(succeed(EvmDataWriter::new().write(decimals).build()))
	}
}
