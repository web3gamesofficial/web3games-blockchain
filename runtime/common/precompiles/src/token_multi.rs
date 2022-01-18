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

use crate::{CREATE_SELECTOR, MT_PRECOMPILE_ADDRESS_PREFIX};
use fp_evm::{
	Context, ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileOutput,
	PrecompileResult,
};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use pallet_support::{MultiMetadata, TokenIdConversion};
use precompile_utils::{Address, Bytes, EvmDataReader, EvmDataWriter, Gasometer, RuntimeHelper};
use primitives::{Balance, TokenId};
use sp_core::{H160, U256};
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

pub type MultiTokenIdOf<Runtime> = <Runtime as pallet_token_multi::Config>::MultiTokenId;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
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
	<Runtime as pallet_token_multi::Config>::MultiTokenId: Into<u32>,
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
		let id: u32 = id.into();
		let mut data = [0u8; 20];
		data[0..4].copy_from_slice(MT_PRECOMPILE_ADDRESS_PREFIX);
		data[4..20].copy_from_slice(&id.to_be_bytes());
		H160::from_slice(&data)
	}
}

impl<Runtime> Precompile for MultiTokenExtension<Runtime>
where
	Runtime: pallet_token_multi::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_multi::Call<Runtime>>,
	<Runtime as pallet_token_multi::Config>::MultiTokenId: Into<u32>,
{
	fn execute(
		input: &[u8], // reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		_is_static: bool,
	) -> PrecompileResult {
		if let Some(multi_token_id) = Self::try_from_address(context.address) {
			if pallet_token_multi::Pallet::<Runtime>::exists(multi_token_id) {
				let (input, selector) = EvmDataReader::new_with_selector(input)?;

				let (origin, call) = match selector {
					// storage getters
					Action::BalanceOf => {
						return Self::balance_of(multi_token_id, input, target_gas)
					},
					Action::BalanceOfBatch => {
						return Self::balance_of_batch(multi_token_id, input, target_gas)
					},
					Action::URI => return Self::uri(multi_token_id, input, target_gas),
					// runtime methods (dispatchable)
					Action::TransferFrom => {
						Self::transfer_from(multi_token_id, input, target_gas, context)?
					},
					Action::BatchTransferFrom => {
						Self::batch_transfer_from(multi_token_id, input, target_gas, context)?
					},
					Action::Mint => Self::mint(multi_token_id, input, target_gas, context)?,
					Action::MintBatch => {
						Self::mint_batch(multi_token_id, input, target_gas, context)?
					},
					Action::Burn => Self::burn(multi_token_id, input, target_gas, context)?,
					Action::BurnBatch => {
						Self::burn_batch(multi_token_id, input, target_gas, context)?
					},
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
				// Action::Create = "create(bytes)"

				let selector = &input[0..4];
				if selector == CREATE_SELECTOR {
					let input = EvmDataReader::new(&input[4..]);
					return Self::create(input, target_gas, context);
				} else {
					return Err(PrecompileFailure::Error {
						exit_status: ExitError::Other("multi token not exists".into()),
					});
				}
			}
		}

		Err(PrecompileFailure::Error {
			exit_status: ExitError::Other("multi token precompile execution failed".into()),
		})
	}
}

impl<Runtime> MultiTokenExtension<Runtime>
where
	Runtime: pallet_token_multi::Config + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_multi::Call<Runtime>>,
	<Runtime as pallet_token_multi::Config>::MultiTokenId: Into<u32>,
{
	fn create(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(3)?;

		let token_uri: Vec<u8> = input.read::<Bytes>()?.into();

		let caller: Runtime::AccountId = Runtime::AddressMapping::into_account_id(context.caller);

		let id: u32 = pallet_token_multi::Pallet::<Runtime>::do_create_token(&caller, token_uri)
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
		id: MultiTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(2)?;

		let account: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let token_id = input.read::<TokenId>()?;

		let balance: Balance =
			pallet_token_multi::Pallet::<Runtime>::balance_of(id, (token_id, &account));

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(balance).build(),
			logs: vec![],
		})
	}

	fn balance_of_batch(
		id: MultiTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(2)?;

		let accounts: Vec<Runtime::AccountId> = input
			.read::<Vec<Address>>()?
			.iter()
			.map(|&a| Runtime::AddressMapping::into_account_id(a.0))
			.collect();
		let ids = input.read::<Vec<TokenId>>()?;

		let balances: Vec<Balance> =
			pallet_token_multi::Pallet::<Runtime>::balance_of_batch(id, &accounts, ids).unwrap();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Vec<Balance>>(balances.as_slice().into()).build(),
			logs: vec![],
		})
	}

	fn transfer_from(
		id: MultiTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<
		(<Runtime::Call as Dispatchable>::Origin, pallet_token_multi::Call<Runtime>),
		PrecompileFailure,
	> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(4)?;

		let from: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let to: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let token_id = input.read::<TokenId>()?;
		let amount = input.read::<Balance>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		let call =
			pallet_token_multi::Call::<Runtime>::transfer_from { id, from, to, token_id, amount };

		Ok((Some(origin).into(), call))
	}

	fn batch_transfer_from(
		id: MultiTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<
		(<Runtime::Call as Dispatchable>::Origin, pallet_token_multi::Call<Runtime>),
		PrecompileFailure,
	> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(4)?;

		let from: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let to: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let token_ids = input.read::<Vec<TokenId>>()?;
		let amounts = input.read::<Vec<Balance>>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		let call = pallet_token_multi::Call::<Runtime>::batch_transfer_from {
			id,
			from,
			to,
			token_ids,
			amounts,
		};

		Ok((Some(origin).into(), call))
	}

	fn mint(
		id: MultiTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<
		(<Runtime::Call as Dispatchable>::Origin, pallet_token_multi::Call<Runtime>),
		PrecompileFailure,
	> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(3)?;

		let to: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let token_id = input.read::<TokenId>()?;
		let amount = input.read::<Balance>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		let call = pallet_token_multi::Call::<Runtime>::mint { id, to, token_id, amount };

		Ok((Some(origin).into(), call))
	}

	fn mint_batch(
		id: MultiTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<
		(<Runtime::Call as Dispatchable>::Origin, pallet_token_multi::Call<Runtime>),
		PrecompileFailure,
	> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(3)?;

		let to: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let token_ids = input.read::<Vec<TokenId>>()?;
		let amounts = input.read::<Vec<Balance>>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		let call = pallet_token_multi::Call::<Runtime>::mint_batch { id, to, token_ids, amounts };

		Ok((Some(origin).into(), call))
	}

	fn burn(
		id: MultiTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<
		(<Runtime::Call as Dispatchable>::Origin, pallet_token_multi::Call<Runtime>),
		PrecompileFailure,
	> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(2)?;

		let token_id = input.read::<TokenId>()?;
		let amount = input.read::<Balance>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		let call = pallet_token_multi::Call::<Runtime>::burn { id, token_id, amount };

		Ok((Some(origin).into(), call))
	}

	fn burn_batch(
		id: MultiTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<
		(<Runtime::Call as Dispatchable>::Origin, pallet_token_multi::Call<Runtime>),
		PrecompileFailure,
	> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(2)?;

		let token_ids = input.read::<Vec<TokenId>>()?;
		let amounts = input.read::<Vec<Balance>>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		let call = pallet_token_multi::Call::<Runtime>::burn_batch { id, token_ids, amounts };

		Ok((Some(origin).into(), call))
	}

	fn uri(
		id: MultiTokenIdOf<Runtime>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, PrecompileFailure> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(1)?;

		let token_id = input.read::<TokenId>()?;

		let uri: Vec<u8> = pallet_token_multi::Pallet::<Runtime>::uri(id, token_id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Bytes>(uri.as_slice().into()).build(),
			logs: vec![],
		})
	}
}
