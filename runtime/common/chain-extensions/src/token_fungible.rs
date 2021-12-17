use codec::Encode;
use frame_support::{dispatch::GetDispatchInfo, weights::extract_actual_weight};
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, SysConfig, UncheckedFrom,
};
use primitives::Balance;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

pub struct FungibleTokenExtension;

impl<C> ChainExtension<C> for FungibleTokenExtension
where
	C: pallet_contracts::Config + pallet_token_fungible::Config,
	<C as pallet_contracts::Config>::Call: From<pallet_token_fungible::Call<C>>,
{
	fn call<E>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			// create_token
			65537 => {
				log::info!("func id 65537");

				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();

				let (name, symbol, decimals): (Vec<u8>, Vec<u8>, u8) =
					env.read_as_unbounded(env.in_len())?;
				env.charge_weight(10000)?;

				let id = pallet_token_fungible::Pallet::<E::T>::do_create_token(
					&caller, name, symbol, decimals,
				)?;

				let id_slice = id.encode();
				log::info!("id slice {:?}", id_slice);

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create token")
				})?;
			}

			// approve
			65538 => {
				log::info!("func id 65538");
				let mut env = env.buf_in_buf_out();

				let (id, spender, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					<E::T as SysConfig>::AccountId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;

				let call =
					<E::T as pallet_contracts::Config>::Call::from(pallet_token_fungible::Call::<
						E::T,
					>::approve {
						id,
						spender,
						amount,
					});

				let dispatch_info = call.get_dispatch_info();
				let charged = env.charge_weight(dispatch_info.weight)?;
				let result = env.ext().call_runtime(call);
				let actual_weight = extract_actual_weight(&result, &dispatch_info);
				env.adjust_weight(charged, actual_weight);

				match result {
					Ok(_) => {}
					Err(_) => return Err(DispatchError::Other("Call runtime returned error")),
				}
			}

			// transfer
			65539 => {
				log::info!("func id 65539");
				let mut env = env.buf_in_buf_out();

				let (id, recipient, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					<E::T as SysConfig>::AccountId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;

				let call =
					<E::T as pallet_contracts::Config>::Call::from(pallet_token_fungible::Call::<
						E::T,
					>::transfer {
						id,
						recipient,
						amount,
					});

				let dispatch_info = call.get_dispatch_info();
				let charged = env.charge_weight(dispatch_info.weight)?;
				let result = env.ext().call_runtime(call);
				let actual_weight = extract_actual_weight(&result, &dispatch_info);
				env.adjust_weight(charged, actual_weight);

				match result {
					Ok(_) => {}
					Err(_) => return Err(DispatchError::Other("Call runtime returned error")),
				}
			}

			// transfer_from
			65540 => {
				log::info!("func id 65540");
				let mut env = env.buf_in_buf_out();

				let (id, sender, recipient, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					<E::T as SysConfig>::AccountId,
					<E::T as SysConfig>::AccountId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;

				let call =
					<E::T as pallet_contracts::Config>::Call::from(pallet_token_fungible::Call::<
						E::T,
					>::transfer_from {
						id,
						sender,
						recipient,
						amount,
					});

				let dispatch_info = call.get_dispatch_info();
				let charged = env.charge_weight(dispatch_info.weight)?;
				let result = env.ext().call_runtime(call);
				let actual_weight = extract_actual_weight(&result, &dispatch_info);
				env.adjust_weight(charged, actual_weight);

				match result {
					Ok(_) => {}
					Err(_) => return Err(DispatchError::Other("Call runtime returned error")),
				}
			}

			// mint
			65541 => {
				log::info!("func id 65541");
				let mut env = env.buf_in_buf_out();

				let (id, account, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					<E::T as SysConfig>::AccountId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;

				log::info!("{:#?} {:#?} {:#?}",id, account, amount);

				let call =
					<E::T as pallet_contracts::Config>::Call::from(pallet_token_fungible::Call::<
						E::T,
					>::mint {
						id,
						account,
						amount,
					});

				let dispatch_info = call.get_dispatch_info();
				let charged = env.charge_weight(dispatch_info.weight)?;
				let result = env.ext().call_runtime(call);

				let actual_weight = extract_actual_weight(&result, &dispatch_info);
				env.adjust_weight(charged, actual_weight);

				match result {
					Ok(_) => {}
					Err(_) => return Err(DispatchError::Other("Call runtime returned error")),
				}
			}

			// burn
			65542 => {
				log::info!("func id 65542");
				let mut env = env.buf_in_buf_out();

				let (id, amount): (
					<E::T as pallet_token_fungible::Config>::FungibleTokenId,
					Balance,
				) = env.read_as_unbounded(env.in_len())?;

				let call =
					<E::T as pallet_contracts::Config>::Call::from(pallet_token_fungible::Call::<
						E::T,
					>::burn {
						id,
						amount,
					});

				let dispatch_info = call.get_dispatch_info();
				let charged = env.charge_weight(dispatch_info.weight)?;
				let result = env.ext().call_runtime(call);
				let actual_weight = extract_actual_weight(&result, &dispatch_info);
				env.adjust_weight(charged, actual_weight);

				match result {
					Ok(_) => {}
					Err(_) => return Err(DispatchError::Other("Call runtime returned error")),
				}
			}

			// exists
			65543 => {
				log::info!("func id 65543");
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_fungible::Config>::FungibleTokenId = env.read_as()?;

				let exists: bool = pallet_token_fungible::Pallet::<E::T>::exists(id);

				let exists_slice = exists.encode();

				log::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);

				env.write(&exists_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create collection")
				})?;
			}

			// total_supply
			65544 => {
				log::info!("func id 65544");
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_fungible::Config>::FungibleTokenId = env.read_as()?;

				let exists: Balance = pallet_token_fungible::Pallet::<E::T>::total_supply(id);

				let exists_slice = exists.encode();

				log::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);

				env.write(&exists_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create collection")
				})?;
			}
			_ => {
				log::error!("call an unregistered `func_id`, func_id:{:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			}
		}
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
