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
use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, PrecompileSet};
use pallet_support::{MultiMetadata, TokenIdConversion};
use precompile_utils::{
	Address, Bytes, EvmDataReader, EvmDataWriter, EvmResult, FunctionModifier, Gasometer,
	RuntimeHelper,
};
use primitives::{Balance, TokenId};
use sp_core::H160;
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

pub type MultiTokenIdOf<Runtime> = <Runtime as pallet_token_multi::Config>::MultiTokenId;

#[precompile_utils::generate_function_selector]
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
	fn execute(
		&self,
		address: H160,
		input: &[u8], // reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<EvmResult<PrecompileOutput>> {
		if let Some(multi_token_id) = Self::try_from_address(address) {
			let mut gasometer = Gasometer::new(target_gas);
			let gasometer = &mut gasometer;
			if pallet_token_multi::Pallet::<Runtime>::exists(multi_token_id) {
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
						Action::BalanceOf => Self::balance_of(multi_token_id, input, gasometer),
						Action::BalanceOfBatch => {
							Self::balance_of_batch(multi_token_id, input, gasometer)
						},
						Action::URI => Self::uri(multi_token_id, input, gasometer),
						// runtime methods (dispatchable)
						Action::TransferFrom => {
							Self::transfer_from(multi_token_id, input, gasometer, context)
						},
						Action::BatchTransferFrom => {
							Self::batch_transfer_from(multi_token_id, input, gasometer, context)
						},
						Action::Mint => Self::mint(multi_token_id, input, gasometer, context),
						Action::MintBatch => {
							Self::mint_batch(multi_token_id, input, gasometer, context)
						},
						Action::Burn => Self::burn(multi_token_id, input, gasometer, context),
						Action::BurnBatch => {
							Self::burn_batch(multi_token_id, input, gasometer, context)
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
	fn create(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 2)?;

		let id = input.read::<u128>(gasometer)?.into();
		let uri: Vec<u8> = input.read::<Bytes>(gasometer)?.into();

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_multi::Call::<Runtime>::create_token { id, uri },
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
		id: MultiTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 2)?;

		let account: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>(gasometer)?.0);
		let token_id: Runtime::TokenId = input.read::<TokenId>(gasometer)?.into();

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
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 2)?;

		let accounts: Vec<Runtime::AccountId> = input
			.read::<Vec<Address>>(gasometer)?
			.iter()
			.map(|&a| Runtime::AddressMapping::into_account_id(a.0))
			.collect();
		let token_ids: Vec<Runtime::TokenId> = input
			.read::<Vec<TokenId>>(gasometer)?
			.iter()
			.map(|&a| Runtime::TokenId::from(a))
			.collect();

		let balances: Vec<Balance> =
			pallet_token_multi::Pallet::<Runtime>::balance_of_batch(id, &accounts, token_ids)
				.unwrap();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Vec<Balance>>(balances.as_slice().into()).build(),
			logs: vec![],
		})
	}

	fn transfer_from(
		id: MultiTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 4)?;

		let from: H160 = input.read::<Address>(gasometer)?.into();
		let to: H160 = input.read::<Address>(gasometer)?.into();
		let token_id: Runtime::TokenId = input.read::<TokenId>(gasometer)?.into();
		let amount = input.read::<Balance>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_multi::Call::<Runtime>::transfer_from {
					id,
					from,
					to,
					token_id,
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

	fn batch_transfer_from(
		id: MultiTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 4)?;

		let from: H160 = input.read::<Address>(gasometer)?.into();
		let to: H160 = input.read::<Address>(gasometer)?.into();
		let token_ids: Vec<Runtime::TokenId> = input
			.read::<Vec<TokenId>>(gasometer)?
			.iter()
			.map(|&a| Runtime::TokenId::from(a))
			.collect();
		let amounts = input.read::<Vec<Balance>>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_multi::Call::<Runtime>::batch_transfer_from {
					id,
					from,
					to,
					token_ids,
					amounts,
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
		id: MultiTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 3)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();
		let token_id: Runtime::TokenId = input.read::<TokenId>(gasometer)?.into();
		let amount = input.read::<Balance>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_multi::Call::<Runtime>::mint { id, to, token_id, amount },
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

	fn mint_batch(
		id: MultiTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 3)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();
		let token_ids: Vec<Runtime::TokenId> = input
			.read::<Vec<TokenId>>(gasometer)?
			.iter()
			.map(|&a| Runtime::TokenId::from(a))
			.collect();
		let amounts = input.read::<Vec<Balance>>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_multi::Call::<Runtime>::mint_batch { id, to, token_ids, amounts },
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
		id: MultiTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 2)?;

		let token_id: Runtime::TokenId = input.read::<TokenId>(gasometer)?.into();
		let amount = input.read::<Balance>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_multi::Call::<Runtime>::burn { id, token_id, amount },
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

	fn burn_batch(
		id: MultiTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		input.expect_arguments(gasometer, 2)?;

		let token_ids: Vec<Runtime::TokenId> = input
			.read::<Vec<TokenId>>(gasometer)?
			.iter()
			.map(|&a| Runtime::TokenId::from(a))
			.collect();
		let amounts = input.read::<Vec<Balance>>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(caller).into(),
				pallet_token_multi::Call::<Runtime>::burn_batch { id, token_ids, amounts },
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

	fn uri(
		id: MultiTokenIdOf<Runtime>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 1)?;

		let token_id: Runtime::TokenId = input.read::<TokenId>(gasometer)?.into();

		let uri: Vec<u8> = pallet_token_multi::Pallet::<Runtime>::uri(id, token_id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write::<Bytes>(uri.as_slice().into()).build(),
			logs: vec![],
		})
	}
}
