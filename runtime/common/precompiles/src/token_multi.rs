use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use fp_evm::Precompile;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use precompile_utils::{Address, EvmDataReader, EvmDataWriter, Gasometer, RuntimeHelper};
use primitives::{Balance, TokenId};
use sp_core::U256;
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
enum Action {
	CreateToken = "createToken(bytes)",
	TransferFrom = "transferFrom(uint256,address,address,uint256,uint256)",
	Mint = "mint(uint256,address,uint256,uint256)",
	BalanceOf = "balanceOf(uint256,address,uint256)",
}

pub struct MultiTokenExtension<Runtime>(PhantomData<Runtime>);

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
	) -> Result<PrecompileOutput, ExitError> {
		log::info!("precompiles: tokens call");
		let mut input = EvmDataReader::new(input);

		let (origin, call) = match &input.read_selector()? {
			// pallet methods
			Action::CreateToken => return Self::create_token(input, context),
			// storage getters
			Action::BalanceOf => return Self::balance_of(input, target_gas),
			// runtime methods (dispatchable)
			Action::TransferFrom => Self::transfer_from(input, context)?,
			Action::Mint => Self::mint(input, context)?,
		};

		// initialize gasometer
		let mut gasometer = Gasometer::new(target_gas);
		// dispatch call (if enough gas).
		let used_gas =
			RuntimeHelper::<Runtime>::try_dispatch(origin, call, gasometer.remaining_gas()?)?;
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			logs: vec![],
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
	fn create_token(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		log::info!("create token");
		input.expect_arguments(3)?;

		let _count = input.read::<u64>()?; // length of solidity abi encode bytes
		let uri_len = input.read::<u32>()?;
		let uri = input.read_raw_bytes(uri_len as usize)?.to_vec();

		let caller: Runtime::AccountId =
		Runtime::AddressMapping::into_account_id(context.caller);

		let id: u32 = pallet_token_multi::Pallet::<Runtime>::do_create_token(&caller, uri).map_err(|e| {
			let err_msg: &str = e.into();
			ExitError::Other(err_msg.into())
		})?.into();

		let output = U256::from(id);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: 0,
			output: EvmDataWriter::new().write(output).build(),
			logs: vec![],
		})
	}

	fn transfer_from(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			pallet_token_multi::Call<Runtime>,
		),
		ExitError,
	> {
		log::info!("transfer from");
		input.expect_arguments(5)?;
		let id: <Runtime as pallet_token_multi::Config>::MultiTokenId = input.read::<u32>()?.into();
		let from: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let to: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let token_id = input.read::<TokenId>()?;
		let amount = input.read::<Balance>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		let call = pallet_token_multi::Call::<Runtime>::transfer_from(id, from, to, token_id, amount);

		Ok((Some(origin).into(), call))
	}

	fn mint(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			pallet_token_multi::Call<Runtime>,
		),
		ExitError,
	> {
		log::info!("mint");
		input.expect_arguments(4)?;
		let id: <Runtime as pallet_token_multi::Config>::MultiTokenId = input.read::<u32>()?.into();
		let to: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let token_id = input.read::<TokenId>()?;
		let amount = input.read::<Balance>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		let call = pallet_token_multi::Call::<Runtime>::mint(id, to, token_id, amount);

		Ok((Some(origin).into(), call))
	}

	fn balance_of(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		log::info!("balance of");
		let mut gasometer = Gasometer::new(target_gas);

		input.expect_arguments(3)?;

		let id: <Runtime as pallet_token_multi::Config>::MultiTokenId = input.read::<u32>()?.into();
		let account: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let token_id = input.read::<TokenId>()?;

		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let balance: Balance =
			pallet_token_multi::Pallet::<Runtime>::balance_of(id, (token_id, &account));

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(balance).build(),
			logs: vec![],
		})
	}
}
