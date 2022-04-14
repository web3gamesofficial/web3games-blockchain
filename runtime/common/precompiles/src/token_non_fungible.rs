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
use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, PrecompileSet};
use pallet_support::{
	AccountMapping, NonFungibleEnumerable, NonFungibleMetadata, TokenIdConversion,
};
use precompile_utils::{
	keccak256, Address, Bytes, EvmDataReader, EvmDataWriter, EvmResult, FunctionModifier,
	Gasometer, LogsBuilder, RuntimeHelper,
};
use primitives::{Balance, TokenId, TokenIndex};
use sp_core::H160;
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

pub type NonFungibleTokenIdOf<Runtime> =
	<Runtime as pallet_token_non_fungible::Config>::NonFungibleTokenId;

#[precompile_utils::generate_function_selector]
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

impl<Runtime> PrecompileSet for NonFungibleTokenExtension<Runtime>
where
	Runtime: pallet_token_non_fungible::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_non_fungible::Call<Runtime>>,
	<Runtime as pallet_token_non_fungible::Config>::NonFungibleTokenId: From<u128> + Into<u128>,
	<Runtime as pallet_token_non_fungible::Config>::TokenId: From<u128> + Into<u128>,
	Runtime: AccountMapping<Runtime::AccountId>,
{
	fn execute(
		&self,
		address: H160,
		input: &[u8], // reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<EvmResult<PrecompileOutput>> {
		if let Some(non_fungible_token_id) = Self::try_from_address(address) {
			let mut gasometer = Gasometer::new(target_gas);
			let gasometer = &mut gasometer;
			if pallet_token_non_fungible::Pallet::<Runtime>::exists(non_fungible_token_id) {
				let result = {
					let (mut input, selector) =
						match EvmDataReader::new_with_selector(gasometer, input) {
							Ok((input, selector)) => (input, selector),
							Err(e) => return Some(Err(e)),
						};
					let input = &mut input;

					if let Err(err) = gasometer.check_function_modifier(
						context,
						is_static,
						match selector {
							Action::TransferFrom | Action::Mint | Action::Burn => {
								FunctionModifier::NonPayable
							},
							_ => FunctionModifier::View,
						},
					) {
						return Some(Err(err));
					}

					match selector {
						// storage getters
						Action::Name => Self::name(non_fungible_token_id, gasometer),
						Action::Symbol => Self::symbol(non_fungible_token_id, gasometer),
						Action::TokenURI => {
							Self::token_uri(non_fungible_token_id, input, gasometer)
						},
						Action::TotalSupply => Self::total_supply(non_fungible_token_id, gasometer),
						Action::TokenByIndex => {
							Self::token_by_index(non_fungible_token_id, input, gasometer)
						},
						Action::TokenOfOwnerByIndex => {
							Self::token_of_owner_by_index(non_fungible_token_id, input, gasometer)
						},
						Action::BalanceOf => {
							Self::balance_of(non_fungible_token_id, input, gasometer)
						},
						Action::OwnerOf => Self::owner_of(non_fungible_token_id, input, gasometer),
						// call methods (dispatchable)
						Action::TransferFrom => {
							Self::transfer_from(non_fungible_token_id, input, gasometer, context)
						},
						Action::Mint => {
							Self::mint(non_fungible_token_id, input, gasometer, context)
						},
						Action::Burn => {
							Self::burn(non_fungible_token_id, input, gasometer, context)
						},
					}
				};
				return Some(result);
			} else {
				if &input[0..4] == CREATE_SELECTOR {
					let mut input = EvmDataReader::new(&input[4..]);
					let result = Self::create(&mut input, gasometer, context);
					return Some(result);
				}
			}
		}

		None
	}

	fn is_precompile(&self, address: H160) -> bool {
		if let Some(non_fungible_token_id) = Self::try_from_address(address) {
			pallet_token_non_fungible::Pallet::<Runtime>::exists(non_fungible_token_id)
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
	Runtime: pallet_token_non_fungible::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_non_fungible::Call<Runtime>>,
	<Runtime as pallet_token_non_fungible::Config>::NonFungibleTokenId: From<u128> + Into<u128>,
	<Runtime as pallet_token_non_fungible::Config>::TokenId: From<u128> + Into<u128>,
	Runtime: AccountMapping<Runtime::AccountId>,
{
	fn create(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 4)?;

		let id = input.read::<u128>(gasometer)?.into();
		let name: Vec<u8> = input.read::<Bytes>(gasometer)?.into();
		let symbol: Vec<u8> = input.read::<Bytes>(gasometer)?.into();
		let base_uri: Vec<u8> = input.read::<Bytes>(gasometer)?.into();

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_non_fungible::Call::<Runtime>::create_token {
					id,
					name,
					symbol,
					base_uri,
				},
				gasometer,
			)?;
		}

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: vec![],
		})
	}

	fn balance_of(
		id: NonFungibleTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 1)?;

		let owner: H160 = input.read::<Address>(gasometer)?.into();

		let balance: Balance = {
			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			pallet_token_non_fungible::Pallet::<Runtime>::balance_of(id, owner).into()
		};

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(balance).build(),
			logs: vec![],
		})
	}

	fn owner_of(
		id: NonFungibleTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 1)?;

		let token_id: Runtime::TokenId = input.read::<TokenId>(gasometer)?.into();

		let owner_account_id: Runtime::AccountId =
			pallet_token_non_fungible::Pallet::<Runtime>::owner_of(id, token_id).unwrap();

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
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 3)?;
		let from: H160 = input.read::<Address>(gasometer)?.into();
		let to: H160 = input.read::<Address>(gasometer)?.into();
		let token_id = input.read::<TokenId>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			let token_id: Runtime::TokenId = token_id.into();

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_non_fungible::Call::<Runtime>::transfer_from {
					id,
					from,
					to,
					token_id,
				},
				gasometer,
			)?;
		}

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: LogsBuilder::new(context.address)
				.log3(SELECTOR_LOG_TRANSFER, from, to, EvmDataWriter::new().write(token_id).build())
				.build(),
		})
	}

	fn mint(
		id: NonFungibleTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 2)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();
		let token_id = input.read::<TokenId>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			let token_id: Runtime::TokenId = token_id.into();

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_non_fungible::Call::<Runtime>::mint { id, to, token_id },
				gasometer,
			)?;
		}

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: LogsBuilder::new(context.address)
				.log3(
					SELECTOR_LOG_TRANSFER,
					H160::zero(),
					to,
					EvmDataWriter::new().write(token_id).build(),
				)
				.build(),
		})
	}

	fn burn(
		id: NonFungibleTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 1)?;

		let token_id = input.read::<TokenId>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let token_id: Runtime::TokenId = token_id.into();

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_non_fungible::Call::<Runtime>::burn { id, token_id },
				gasometer,
			)?;
		}

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: LogsBuilder::new(context.address)
				.log3(
					SELECTOR_LOG_TRANSFER,
					context.caller,
					H160::zero(),
					EvmDataWriter::new().write(token_id).build(),
				)
				.build(),
		})
	}

	fn name(
		id: NonFungibleTokenIdOf<Runtime>,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
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
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
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
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 1)?;

		let token_id: Runtime::TokenId = input.read::<TokenId>(gasometer)?.into();

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
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
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
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 1)?;

		let token_index = input.read::<TokenIndex>(gasometer)?.into();

		let token_id: TokenId =
			pallet_token_non_fungible::Pallet::<Runtime>::token_by_index(id, token_index).into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(token_id).build(),
			logs: vec![],
		})
	}

	fn token_of_owner_by_index(
		id: NonFungibleTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 2)?;

		let owner: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>(gasometer)?.0);

		let token_index = input.read::<TokenIndex>(gasometer)?.into();

		let token_id: TokenId =
			pallet_token_non_fungible::Pallet::<Runtime>::token_of_owner_by_index(
				id,
				owner,
				token_index,
			)
			.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(token_id).build(),
			logs: vec![],
		})
	}
}
