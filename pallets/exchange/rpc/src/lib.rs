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

use codec::Codec;
use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
pub use pallet_exchange_rpc_runtime_api::ExchangeRuntimeApi;
pub use pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi as TransactionPaymentRuntimeApi;
use primitives::Balance;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::{marker::PhantomData, sync::Arc};

pub struct ExchangeRpc<Client, Block> {
	client: Arc<Client>,
	_marker: PhantomData<Block>,
}

impl<Client, Block> ExchangeRpc<Client, Block> {
	pub fn new(client: Arc<Client>) -> Self {
		Self { client, _marker: PhantomData }
	}
}

#[rpc(client, server)]
pub trait ExchangeRpcApi<BlockHash, AccountId> {
	#[method(name = "exchange_getAmountInPrice")]
	fn get_amount_in_price(
		&self,
		supply: Balance,
		path: Vec<u128>,
		at: Option<BlockHash>,
	) -> RpcResult<Option<Vec<Balance>>>;

	#[method(name = "exchange_getAmountOutPrice")]
	fn get_amount_out_price(
		&self,
		supply: Balance,
		path: Vec<u128>,
		at: Option<BlockHash>,
	) -> RpcResult<Option<Vec<Balance>>>;

	#[method(name = "exchange_getEstimateLpToken")]
	fn get_estimate_lp_token(
		&self,
		token_0: u128,
		amount_0: Balance,
		token_1: u128,
		amount_1: Balance,
		at: Option<BlockHash>,
	) -> RpcResult<Option<Balance>>;

	#[method(name = "exchange_getEstimateOutToken")]
	fn get_estimate_out_token(
		&self,
		supply: Balance,
		token_0: u128,
		token_1: u128,
		at: Option<BlockHash>,
	) -> RpcResult<Option<Balance>>;
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

#[async_trait]
impl<C, Block, AccountId> ExchangeRpcApiServer<<Block as BlockT>::Hash, AccountId>
	for ExchangeRpc<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: ExchangeRuntimeApi<Block, AccountId> + TransactionPaymentRuntimeApi<Block, Balance>,
	AccountId: Codec,
	Balance: Codec + std::fmt::Display + std::ops::Add<Output = Balance> + sp_runtime::traits::Zero,
{
	fn get_amount_in_price(
		&self,
		supply: Balance,
		path: Vec<u128>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<Vec<Balance>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
		// If the block hash is not supplied assume the best block.
		self.client.info().best_hash));

		api.get_amount_in_price(&at, supply, path).map_err(runtime_error_into_rpc_err)
	}
	fn get_amount_out_price(
		&self,
		supply: Balance,
		path: Vec<u128>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<Vec<Balance>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
		// If the block hash is not supplied assume the best block.
		self.client.info().best_hash));

		api.get_amount_out_price(&at, supply, path).map_err(runtime_error_into_rpc_err)
	}
	fn get_estimate_lp_token(
		&self,
		token_0: u128,
		amount_0: Balance,
		token_1: u128,
		amount_1: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
		// If the block hash is not supplied assume the best block.
		self.client.info().best_hash));

		api.get_estimate_lp_token(&at, token_0, amount_0, token_1, amount_1)
			.map_err(runtime_error_into_rpc_err)
	}
	fn get_estimate_out_token(
		&self,
		supply: Balance,
		token_0: u128,
		token_1: u128,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
		// If the block hash is not supplied assume the best block.
		self.client.info().best_hash));

		api.get_estimate_out_token(&at, supply, token_0, token_1)
			.map_err(runtime_error_into_rpc_err)
	}
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Display) -> JsonRpseeError {
	CallError::Custom(ErrorObject::owned(
		Error::RuntimeError.into(),
		"error in exchange pallet",
		Some(err.to_string()),
	))
	.into()
}
