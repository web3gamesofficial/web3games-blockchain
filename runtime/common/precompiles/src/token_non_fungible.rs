// This file is part of Web3Games.

// Copyright (C) 2021 Web3Games https://web3games.org
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
use fp_evm::{
	Context, ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileOutput,
	PrecompileResult,
};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use pallet_support::{
	AccountMapping, NonFungibleEnumerable, NonFungibleMetadata, TokenIdConversion,
};
use precompile_utils::{Address, Bytes, EvmDataReader, EvmDataWriter, Gasometer, RuntimeHelper};
use primitives::{TokenId, TokenIndex};
use sp_core::{H160, U256};
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

pub type NonFungibleTokenIdOf<Runtime> =
	<Runtime as pallet_token_non_fungible::Config>::NonFungibleTokenId;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
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
	<Runtime as pallet_token_non_fungible::Config>::NonFungibleTokenId: Into<u32>,
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
		let id: u32 = id.into();
		let mut data = [0u8; 20];
		data[0..4].copy_from_slice(NFT_PRECOMPILE_ADDRESS_PREFIX);
		data[4..20].copy_from_slice(&id.to_be_bytes());
		H160::from_slice(&data)
	}
}

impl<Runtime> Precompile for NonFungibleTokenExtension<Runtime>
where
	Runtime: pallet_token_non_fungible::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_non_fungible::Call<Runtime>>,
	<Runtime as pallet_token_non_fungible::Config>::NonFungibleTokenId: Into<u32>,
	Runtime: AccountMapping<Runtime::AccountId>,
{
	fn execute(
		input: &[u8], // reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		_is_static: bool,
	) -> PrecompileResult {
		if let Some(non_fungible_token_id) = Self::try_from_address(context.address) {
			if pallet_token_non_fungible::Pallet::<Runtime>::exists(non_fungible_token_id) {
				let (input, selector) = EvmDataReader::new_with_selector(input)?;

				let (origin, call) = match selector {
					// storage getters
					Action::Name => return Self::name(non_fungible_token_id, target_gas),
					Action::Symbol => return Self::symbol(non_fungible_token_id, target_gas),
					Action::TokenURI => {
						return Self::token_uri(non_fungible_token_id, input, target_gas)
					},
					Action::TotalSupply => {
						return Self::total_supply(non_fungible_token_id, target_gas)
					},
					Action::TokenByIndex => {
						return Self::token_by_index(non_fungible_token_id, input, target_gas)
					},
					Action::TokenOfOwnerByIndex => {
						return Self::token_of_owner_by_index(
							non_fungible_token_id,
							input,
							target_gas,
						)
					},
					Action::BalanceOf => {
						return Self::balance_of(non_fungible_token_id, input, target_gas)
					},
					Action::OwnerOf => {
						return Self::owner_of(non_fungible_token_id, input, target_gas)
					},
					// call methods (dispatchable)
					Action::TransferFrom => {
						Self::transfer_from(non_fungible_token_id, input, target_gas, context)?
					},
					Action::Mint => Self::mint(non_fungible_token_id, input, target_gas, context)?,
					Action::Burn => Self::burn(non_fungible_token_id, input, target_gas, context)?,
				};

				// initialize gasometer
				let mut gasometer = Gasometer::new(target_gas);
				// dispatch call (if enough gas).
				let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
					origin,
					call,
					gasometer.remaining_gas()?,
				)?;
				gasometer.record_cost(used_gas)?;

				return Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: gasometer.used_gas(),
					output: vec![],
					logs: vec![],
				});
			} else {
				// Action::Create = "create(bytes,bytes)"

				let selector = &input[0..4];
				if selector == CREATE_SELECTOR {
					let input = EvmDataReader::new(&input[4..]);
					return Self::create(input, target_gas, context);
				} else {
					return Err(PrecompileFailure::Error {
						exit_status: ExitError::Other("fungible token not exists".into()),
					});
				}
			}
		}

		Err(PrecompileFailure::Error {
			exit_status: ExitError::Other("fungible token precompile execution failed".into()),
		})
	}
}

impl<Runtime> NonFungibleTokenExtension<Runtime>
where
	Runtime: pallet_token_non_fungible::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_non_fungible::Call<Runtime>>,
	<Runtime as pallet_token_non_fungible::Config>::NonFungibleTokenId: Into<u32>,
	Runtime: AccountMapping<Runtime::AccountId>,
{
	fn create(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(3)?;

		let name: Vec<u8> = input.read::<Bytes>()?.into();
		let symbol: Vec<u8> = input.read::<Bytes>()?.into();
		let base_uri: Vec<u8> = input.read::<Bytes>()?.into();

		let caller: Runtime::AccountId = Runtime::AddressMapping::into_account_id(context.caller);

		let id: u32 = pallet_token_non_fungible::Pallet::<Runtime>::do_create_token(
			&caller, name, symbol, base_uri,
		)
		.map_err(|e| {
			let err_msg: &str = e.into();
			PrecompileFailure::Error { exit_status: ExitError::Other(err_msg.into()) }
		})?
		.into();

		let output = U256::from(id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: 0,
			output: EvmDataWriter::new().write(output).build(),
			logs: vec![],
		})
	}

	fn balance_of(
		id: NonFungibleTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(1)?;

		let account: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		let balance: u32 = pallet_token_non_fungible::Pallet::<Runtime>::balance_of(id, account);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(balance).build(),
			logs: vec![],
		})
	}

	fn owner_of(
		id: NonFungibleTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(1)?;

		let token_id = input.read::<TokenId>()?;

		let owner_account_id: Runtime::AccountId =
			pallet_token_non_fungible::Pallet::<Runtime>::owner_of(id, token_id);

		let owner = Runtime::into_evm_address(owner_account_id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Address>(owner.into()).build(),
			logs: vec![],
		})
	}

	fn transfer_from(
		id: NonFungibleTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<
		(<Runtime::Call as Dispatchable>::Origin, pallet_token_non_fungible::Call<Runtime>),
		PrecompileFailure,
	> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(3)?;

		let from: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let to: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let token_id = input.read::<TokenId>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		let call =
			pallet_token_non_fungible::Call::<Runtime>::transfer_from { id, from, to, token_id };

		Ok((Some(origin).into(), call))
	}

	fn mint(
		id: NonFungibleTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<
		(<Runtime::Call as Dispatchable>::Origin, pallet_token_non_fungible::Call<Runtime>),
		PrecompileFailure,
	> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(2)?;

		let to: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let token_id = input.read::<TokenId>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		let call = pallet_token_non_fungible::Call::<Runtime>::mint { id, to, token_id };

		Ok((Some(origin).into(), call))
	}

	fn burn(
		id: NonFungibleTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<
		(<Runtime::Call as Dispatchable>::Origin, pallet_token_non_fungible::Call<Runtime>),
		PrecompileFailure,
	> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(1)?;

		let token_id = input.read::<TokenId>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		let call = pallet_token_non_fungible::Call::<Runtime>::burn { id, token_id };

		Ok((Some(origin).into(), call))
	}

	fn name(
		id: NonFungibleTokenIdOf<Runtime>,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let name: Vec<u8> = pallet_token_non_fungible::Pallet::<Runtime>::token_name(id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Bytes>(name.as_slice().into()).build(),
			logs: vec![],
		})
	}

	fn symbol(
		id: NonFungibleTokenIdOf<Runtime>,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let symbol: Vec<u8> = pallet_token_non_fungible::Pallet::<Runtime>::token_symbol(id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Bytes>(symbol.as_slice().into()).build(),
			logs: vec![],
		})
	}

	fn token_uri(
		id: NonFungibleTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(1)?;

		let token_id = input.read::<TokenId>()?.into();

		let token_uri: Vec<u8> =
			pallet_token_non_fungible::Pallet::<Runtime>::token_uri(id, token_id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Bytes>(token_uri.as_slice().into()).build(),
			logs: vec![],
		})
	}

	fn total_supply(
		id: NonFungibleTokenIdOf<Runtime>,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let balance: u32 = pallet_token_non_fungible::Pallet::<Runtime>::total_supply(id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(balance).build(),
			logs: vec![],
		})
	}

	fn token_by_index(
		id: NonFungibleTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(1)?;

		let token_index = input.read::<TokenIndex>()?.into();

		let token_id: TokenId =
			pallet_token_non_fungible::Pallet::<Runtime>::token_by_index(id, token_index);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(token_id).build(),
			logs: vec![],
		})
	}

	fn token_of_owner_by_index(
		id: NonFungibleTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(2)?;

		let owner: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		let token_index = input.read::<TokenIndex>()?.into();

		let token_id: TokenId =
			pallet_token_non_fungible::Pallet::<Runtime>::token_of_owner_by_index(
				id,
				owner,
				token_index,
			);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(token_id).build(),
			logs: vec![],
		})
	}
}
