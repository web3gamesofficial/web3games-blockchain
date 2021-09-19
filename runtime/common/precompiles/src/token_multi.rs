use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use fp_evm::Precompile;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use precompile_utils::{Address, EvmDataReader, EvmDataWriter, Gasometer, RuntimeHelper};
use primitives::{Balance, TokenId};
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
enum Action {
	BalanceOf = "balance_of(address,address,uint256)",
	CreateToken = "create_token(address,bytes)",
}

pub struct MultiTokenExtension<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for MultiTokenExtension<Runtime>
where
	Runtime: pallet_token_multi::Config + pallet_evm::Config,
	// Runtime::AccountId: From<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_multi::Call<Runtime>>,
{
	fn execute(
		input: &[u8], // reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		log::debug!("precompiles: tokens call");
		let mut input = EvmDataReader::new(input);

		let (origin, call) = match &input.read_selector()? {
			Action::BalanceOf => return Self::balance_of(input, target_gas),
			Action::CreateToken => Self::create_token(input, context)?,
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
	// Runtime::AccountId: From<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_token_multi::Call<Runtime>>,
{
	fn balance_of(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		input.expect_arguments(3)?;
		// let account: Runtime::AccountId = input.read::<Address>()?.0.into();
		let token_account: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let account: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		let id = input.read::<TokenId>()?;

		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let balance: Balance =
			pallet_token_multi::Pallet::<Runtime>::balance_of(&token_account, &account, id.into());

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(balance).build(),
			logs: vec![],
		})
	}

	fn create_token(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			pallet_token_multi::Call<Runtime>,
		),
		ExitError,
	> {
		input.expect_arguments(1)?;
		let uri = input.read()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = pallet_token_multi::Call::<Runtime>::create_token(uri);

		Ok((Some(origin).into(), call))
	}
}
