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

use crate::{CREATE_SELECTOR, FT_PRECOMPILE_ADDRESS_PREFIX};
use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, PrecompileSet};
use pallet_support::{FungibleMetadata, TokenIdConversion};
use precompile_utils::{
	Address, Bytes, EvmDataReader, EvmDataWriter, EvmResult, FunctionModifier, Gasometer,
	RuntimeHelper,
};
use primitives::Balance;
use sp_core::H160;
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

pub type FungibleTokenIdOf<Runtime> = <Runtime as pallet_token_fungible::Config>::FungibleTokenId;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	Name = "name()",
	Symbol = "symbol()",
	Decimals = "decimals()",
	TotalSupply = "totalSupply()",
	BalanceOf = "balanceOf(address)",
	Transfer = "transfer(address,uint256)",
	TransferFrom = "transferFrom(address,address,uint256)",
	Mint = "mint(address,uint256)",
	Burn = "burn(uint256)",
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
	fn execute(
		&self,
		address: H160,
		input: &[u8], // reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<EvmResult<PrecompileOutput>> {
		if let Some(fungible_token_id) = Self::try_from_address(address) {
			let mut gasometer = Gasometer::new(target_gas);
			let gasometer = &mut gasometer;
			if pallet_token_fungible::Pallet::<Runtime>::exists(fungible_token_id) {
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
						Action::Name => Self::name(fungible_token_id, gasometer),
						Action::Symbol => Self::symbol(fungible_token_id, gasometer),
						Action::Decimals => Self::decimals(fungible_token_id, gasometer),
						Action::TotalSupply => Self::total_supply(fungible_token_id, gasometer),
						Action::BalanceOf => Self::balance_of(fungible_token_id, input, gasometer),
						// call methods (dispatchable)
						Action::Transfer => {
							Self::transfer(fungible_token_id, input, gasometer, context)
						},
						Action::TransferFrom => {
							Self::transfer_from(fungible_token_id, input, gasometer, context)
						},
						Action::Mint => Self::mint(fungible_token_id, input, gasometer, context),
						Action::Burn => Self::burn(fungible_token_id, input, gasometer, context),
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
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 4)?;

		let id = input.read::<u128>(gasometer)?.into();
		let name: Vec<u8> = input.read::<Bytes>(gasometer)?.into();
		let symbol: Vec<u8> = input.read::<Bytes>(gasometer)?.into();
		let decimals = input.read::<u8>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_fungible::Call::<Runtime>::create_token { id, name, symbol, decimals },
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

	fn total_supply(
		id: FungibleTokenIdOf<Runtime>,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let balance: Balance = pallet_token_fungible::Pallet::<Runtime>::total_supply(id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(balance).build(),
			logs: vec![],
		})
	}

	fn balance_of(
		id: FungibleTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 1)?;

		let account: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>(gasometer)?.0);

		let balance: Balance = pallet_token_fungible::Pallet::<Runtime>::balance_of(id, account);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(balance).build(),
			logs: vec![],
		})
	}

	fn transfer(
		id: FungibleTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 2)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();
		let amount = input.read::<Balance>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_fungible::Call::<Runtime>::transfer { id, recipient: to, amount },
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

	fn transfer_from(
		id: FungibleTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 3)?;

		let from: H160 = input.read::<Address>(gasometer)?.into();
		let to: H160 = input.read::<Address>(gasometer)?.into();
		let amount = input.read::<Balance>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_fungible::Call::<Runtime>::transfer_from {
					id,
					sender: from,
					recipient: to,
					amount,
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

	fn mint(
		id: FungibleTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 2)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();
		let amount = input.read::<Balance>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_fungible::Call::<Runtime>::mint { id, account: to, amount },
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

	fn burn(
		id: FungibleTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 1)?;

		let amount = input.read::<Balance>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_fungible::Call::<Runtime>::burn { id, amount },
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

	fn name(
		id: FungibleTokenIdOf<Runtime>,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let name: Vec<u8> = pallet_token_fungible::Pallet::<Runtime>::token_name(id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Bytes>(name.as_slice().into()).build(),
			logs: vec![],
		})
	}

	fn symbol(
		id: FungibleTokenIdOf<Runtime>,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let symbol: Vec<u8> = pallet_token_fungible::Pallet::<Runtime>::token_symbol(id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Bytes>(symbol.as_slice().into()).build(),
			logs: vec![],
		})
	}

	fn decimals(
		id: FungibleTokenIdOf<Runtime>,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let decimals: u8 = pallet_token_fungible::Pallet::<Runtime>::token_decimals(id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(decimals).build(),
			logs: vec![],
		})
	}
}
