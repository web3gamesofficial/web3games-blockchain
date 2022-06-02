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

use hex_literal::hex;
use pallet_evm::{AddressMapping, HashedAddressMapping};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde_json::json;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public, H160};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{BlakeTwo256, IdentifyAccount, Verify};
use std::str::FromStr;
use web3games_runtime::{
	AccountId, AuraConfig, Balance, BalancesConfig, EVMConfig, EthereumConfig, EvmChainIdConfig,
	GenesisConfig, GrandpaConfig, Precompiles, Signature, SudoConfig, SystemConfig,
	WrapCurrencyConfig, DOLLARS, WASM_BINARY,
};

// The URL for the telemetry server.
const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn get_account_id_from_evm_address(address: &str) -> AccountId {
	HashedAddressMapping::<BlakeTwo256>::into_account_id(
		H160::from_str(address).expect("invalid evm address"),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		// Name
		"Web3Games Development",
		// ID
		"web3games_dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_evm_address("6be02d1d3665660d22ff9624b7be0551ee1ac91b"),
				],
				// ChainId
				104,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		Some(
			json!({
				"tokenDecimals": 18,
				"tokenSymbol": "W3G"
			})
			.as_object()
			.expect("Provided valid json map")
			.clone(),
		),
		// Extensions
		None,
	))
}

pub fn local_devnet_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		// Name
		"Web3Games Local Devnet",
		// ID
		"web3games_devnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
					get_account_id_from_evm_address("6be02d1d3665660d22ff9624b7be0551ee1ac91b"),
				],
				// ChainId
				105,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		Some(
			json!({
				"tokenDecimals": 18,
				"tokenSymbol": "W3G"
			})
			.as_object()
			.expect("Provided valid json map")
			.clone(),
		),
		// Extensions
		None,
	))
}

pub fn staging_testnet_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		// Name
		"Web3Games Testnet",
		// ID
		"web3games_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				// Initial PoA authorities
				vec![
					(
						// 5CAGc2P2g6jUVSmdcpHbGD1MWvTe7jAAmoD6riLnXUjc5PV4
						hex!["043e7709226be05310d0632dc1f7cb1b0016b74c0c051835e1093428a472d230"]
							.unchecked_into(),
						// 5G8WWBAq4gpFLEnQjsxPkfYRdXNs3UkaeR8QG39HVjub8agw
						hex!["b3d7b25de30f345bfb40e0fb78f86acc36a7edc045615d5dee2cb9539faa8219"]
							.unchecked_into(),
					),
					(
						// 5EL1RhP3yJNVF7k1nB9U8Dm5AbcFUEbD7ZQ4f3TUeHVYV6Vj
						hex!["64244ac1fb0854c2f101beafa5d8032d0e381705514f74cf58c8f8361d65c769"]
							.unchecked_into(),
						// 5EaM6zor3sPnqfYLM1CRu1RFPsyMkNa1B1r3J4itBSLs8mx8
						hex!["6f13f7e727ef6b4094b346e351e66242b51fbbb6a2eac532b55389f1314d2d11"]
							.unchecked_into(),
					),
				],
				// Sudo account
				hex![
					// 5EUp2vXWQEmbT6ceUA5t3XCaHzvAbBgtXPYenE4ui6mhwX89
					"6adb264c6a79923eb1b3d47feab4db75b0fd140ba31a1f0bfee91ba3070f3541"
				]
				.into(),
				// Pre-funded accounts
				vec![
					// 5EUp2vXWQEmbT6ceUA5t3XCaHzvAbBgtXPYenE4ui6mhwX89
					hex!["6adb264c6a79923eb1b3d47feab4db75b0fd140ba31a1f0bfee91ba3070f3541"].into(),
				],
				// ChainId
				102,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		// Protocol ID
		None,
		None,
		// Properties
		Some(
			json!({
				"tokenDecimals": 18,
				"tokenSymbol": "W3G"
			})
			.as_object()
			.expect("Provided valid json map")
			.clone(),
		),
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	chain_id: u64,
) -> GenesisConfig {
	// This is the simplest bytecode to revert without returning any data.
	// We will pre-deploy it under all of our precompiles to ensure they can be called from
	// within contracts.
	// (PUSH1 0x00 PUSH1 0x00 REVERT)
	let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];

	const ENDOWMENT: Balance = 100_000_000 * DOLLARS;

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: WASM_BINARY.expect("WASM binary is not available.").to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, ENDOWMENT)).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig { key: Some(root_key) },
		transaction_payment: Default::default(),
		evm: EVMConfig {
			// We need _some_ code inserted at the precompile address so that
			// the evm will actually call the address.
			accounts: Precompiles::used_addresses()
				.iter()
				.map(|addr| {
					(
						addr.clone(),
						fp_evm::GenesisAccount {
							nonce: Default::default(),
							balance: Default::default(),
							storage: Default::default(),
							code: revert_bytecode.clone(),
						},
					)
				})
				.collect(),
		},
		ethereum: EthereumConfig {},
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
		treasury: Default::default(),
		wrap_currency: WrapCurrencyConfig {},
		evm_chain_id: EvmChainIdConfig { chain_id },
	}
}
