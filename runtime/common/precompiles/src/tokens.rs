use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use fp_evm::Precompile;
use pallet_evm::AddressMapping;
use precompile_utils::{
	error, Address, EvmData, EvmDataReader, EvmDataWriter, Gasometer, RuntimeHelper,
};
use sp_core::H160;
use sp_std::{
    marker::PhantomData,
    prelude::*,
    convert::TryInto,
    fmt::Debug,
};

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
enum Action {
    BalanceOf = "balance_of(address,uint256,uint256)",
    CreateToken = "create_token(address,uint256,uint256,bool,bytes)",
}

pub struct TokensPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for TokensPrecompile<Runtime>
where
    Runtime: pallet_tokens::Config + pallet_evm::Config,
    // Runtime::AccountId: From<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
    Runtime::Call: From<pallet_tokens::Call<Runtime>>
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


impl<Runtime> TokensPrecompile<Runtime>
where
    Runtime: pallet_tokens::Config + pallet_evm::Config,
    // Runtime::AccountId: From<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
    Runtime::Call: From<pallet_tokens::Call<Runtime>>
{
    fn balance_of(
        mut input: EvmDataReader,
        target_gas: Option<u64>,
    ) -> Result<PrecompileOutput, ExitError> {
        let mut gasometer = Gasometer::new(target_gas);

        input.expect_arguments(3)?;
        // let account: Runtime::AccountId = input.read::<Address>()?.0.into();
        let account: Runtime::AccountId = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
        
        let instance_id = input.read::<u64>()?;
        let token_id = input.read::<u64>()?;

        gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        let balance: u128 = pallet_tokens::Pallet::<Runtime>::balance_of(
            &account,
            instance_id.into(),
            token_id.into(),
        );

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
            pallet_tokens::Call<Runtime>,
        ),
        ExitError,
    > {
        input.expect_arguments(5)?;
        // let account: Runtime::AccountId = input.read::<Address>()?.0.into();
        let account: Runtime::AccountId = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
        let instance_id = input.read::<u64>()?;
        let token_id = input.read::<u64>()?;
        let is_nf = input.read()?;
        let uri = input.read()?;

        let origin = Runtime::AddressMapping::into_account_id(context.caller);
        let call = pallet_tokens::Call::<Runtime>::create_token(
            instance_id.into(),
            token_id.into(),
            is_nf,
            uri,
        );

        Ok((Some(origin).into(), call))
    }
}

