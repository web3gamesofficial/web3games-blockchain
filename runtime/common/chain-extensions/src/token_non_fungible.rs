use codec::Encode;
use frame_support::{dispatch::GetDispatchInfo, weights::extract_actual_weight};
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, SysConfig, UncheckedFrom,
};
use primitives::TokenId;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

pub struct NonFungibleTokenExtension;

impl<C> ChainExtension<C> for NonFungibleTokenExtension
where
	C: pallet_contracts::Config + pallet_token_non_fungible::Config,
	<C as pallet_contracts::Config>::Call: From<pallet_token_non_fungible::Call<C>>,
{
	fn call<E>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal>
	where
		E: Ext<T = C>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			// create_token
			65601 => {
				log::info!("func id 65601");

				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				log::info!("caller {:?}", caller);

				let in_len = env.in_len();
				log::debug!("in_len {}", in_len);

				let name: Vec<u8> = env.read_as_unbounded(in_len)?;
				log::info!("name {:?}", name);

				let in_len = env.in_len();
				log::debug!("in_len {}", in_len);

				let symbol: Vec<u8> = env.read_as_unbounded(in_len)?;
				log::info!("symbol {:?}", symbol);

				let in_len = env.in_len();
				log::debug!("in_len {}", in_len);

				let base_uri: Vec<u8> = env.read_as_unbounded(in_len)?;
				log::info!("base_uri {:?}", base_uri);

				env.charge_weight(10000)?;

				let id = pallet_token_non_fungible::Pallet::<E::T>::do_create_token(
					&caller, name, symbol, base_uri,
				)?;

				let id_slice = id.encode();
				log::info!("id slice {:?}", id_slice);

				env.write(&id_slice, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call create token")
				})?;
			}

			// approve
			65602 => {
				log::info!("func id 65602");
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId =
					env.read_as()?;
				let to: <E::T as SysConfig>::AccountId = env.read_as()?;
				let token_id: TokenId = env.read_as()?;

				let call = <E::T as pallet_contracts::Config>::Call::from(
					pallet_token_non_fungible::Call::<E::T>::approve { id, to, token_id },
				);

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

			// set_approve_for_all
			65603 => {
				log::info!("func id 65603");
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId =
					env.read_as()?;
				let operator: <E::T as SysConfig>::AccountId = env.read_as()?;
				let approved: bool = env.read_as()?;

				let call = <E::T as pallet_contracts::Config>::Call::from(
					pallet_token_non_fungible::Call::<E::T>::set_approve_for_all {
						id,
						operator,
						approved,
					},
				);

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
			65604 => {
				log::info!("func id 65604");
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId =
					env.read_as()?;
				let from: <E::T as SysConfig>::AccountId = env.read_as()?;
				let to: <E::T as SysConfig>::AccountId = env.read_as()?;
				let token_id: TokenId = env.read_as()?;

				let call = <E::T as pallet_contracts::Config>::Call::from(
					pallet_token_non_fungible::Call::<E::T>::transfer_from {
						id,
						from,
						to,
						token_id,
					},
				);

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
			65605 => {
				log::info!("func id 65605");
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId =
					env.read_as()?;
				let to: <E::T as SysConfig>::AccountId = env.read_as()?;
				let token_id: TokenId = env.read_as()?;

				let call = <E::T as pallet_contracts::Config>::Call::from(
					pallet_token_non_fungible::Call::<E::T>::mint { id, to, token_id },
				);

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
			65606 => {
				log::info!("func id 65606");
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId =
					env.read_as()?;
				let token_id: TokenId = env.read_as()?;

				let call = <E::T as pallet_contracts::Config>::Call::from(
					pallet_token_non_fungible::Call::<E::T>::burn { id, token_id },
				);

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
			65607 => {
				log::info!("func id 65607");
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId =
					env.read_as()?;

				let exists: bool = pallet_token_non_fungible::Pallet::<E::T>::exists(id);

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
			// token_exists
			65608 => {
				log::info!("func id 65607");
				let mut env = env.buf_in_buf_out();

				let id: <E::T as pallet_token_non_fungible::Config>::NonFungibleTokenId =
					env.read_as()?;
				let token_id: TokenId = env.read_as()?;

				let token_exists: bool =
					pallet_token_non_fungible::Pallet::<E::T>::token_exists(id, token_id);

				let token_exists_slice = token_exists.encode();

				log::trace!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);

				env.write(&token_exists_slice, false, None).map_err(|_| {
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
