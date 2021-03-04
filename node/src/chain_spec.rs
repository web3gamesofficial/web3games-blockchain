use sp_core::{U256, Pair, Public, H160, sr25519};
use sgc_runtime::{
    AccountId, AuraConfig, BalancesConfig, EVMConfig, EthereumConfig, GenesisConfig, GrandpaConfig,
    ContractsConfig, SudoConfig, SystemConfig, TokensConfig, WASM_BINARY, Signature, Balance, DOLLARS,
    TokenSymbol, CurrencyId,
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sp_core::crypto::UncheckedInto;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde_json::json;
use std::collections::BTreeMap;
use std::str::FromStr;
use hex_literal::hex;

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
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (
        get_from_seed::<AuraId>(s),
        get_from_seed::<GrandpaId>(s),
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || testnet_genesis(
            wasm_binary,
            // Initial PoA authorities
            vec![
                authority_keys_from_seed("Alice"),
            ],
            // Sudo account
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            // Pre-funded accounts
            vec![
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
            ],
            true,
        ),
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(json!({
            "tokenDecimals": 18,
            "tokenSymbol": "SGC"
          }).as_object().expect("Provided valid json map").clone()),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || testnet_genesis(
            wasm_binary,
            // Initial PoA authorities
            vec![
                authority_keys_from_seed("Alice"),
                authority_keys_from_seed("Bob"),
            ],
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
            ],
            true,
        ),
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(json!({
            "tokenDecimals": 18,
            "tokenSymbol": "SGC"
          }).as_object().expect("Provided valid json map").clone()),
        // Extensions
        None,
    ))
}

pub fn plum_staging_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "SGC Plum",
        // ID
        "sgc_plum",
        ChainType::Live,
        move || testnet_genesis(
            wasm_binary,
            // Initial PoA authorities
            vec![(
                // 5CAGc2P2g6jUVSmdcpHbGD1MWvTe7jAAmoD6riLnXUjc5PV4
                hex!["043e7709226be05310d0632dc1f7cb1b0016b74c0c051835e1093428a472d230"].unchecked_into(),
                // 5G8WWBAq4gpFLEnQjsxPkfYRdXNs3UkaeR8QG39HVjub8agw
                hex!["b3d7b25de30f345bfb40e0fb78f86acc36a7edc045615d5dee2cb9539faa8219"].unchecked_into(),
            ),(
                // 5EL1RhP3yJNVF7k1nB9U8Dm5AbcFUEbD7ZQ4f3TUeHVYV6Vj
                hex!["64244ac1fb0854c2f101beafa5d8032d0e381705514f74cf58c8f8361d65c769"].unchecked_into(),
                // 5EaM6zor3sPnqfYLM1CRu1RFPsyMkNa1B1r3J4itBSLs8mx8
                hex!["6f13f7e727ef6b4094b346e351e66242b51fbbb6a2eac532b55389f1314d2d11"].unchecked_into(),
            )],
            // Sudo account
            hex![
                // 5EUp2vXWQEmbT6ceUA5t3XCaHzvAbBgtXPYenE4ui6mhwX89
                "6adb264c6a79923eb1b3d47feab4db75b0fd140ba31a1f0bfee91ba3070f3541"
            ].into(),
            // Pre-funded accounts
            vec![
                // 5EUp2vXWQEmbT6ceUA5t3XCaHzvAbBgtXPYenE4ui6mhwX89
                hex!["6adb264c6a79923eb1b3d47feab4db75b0fd140ba31a1f0bfee91ba3070f3541"].into(),
            ],
            true,
        ),
        // Bootnodes
        vec![],
        // Telemetry
        TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
        // Protocol ID
        Some("plum"),
        // Properties
        Some(json!({
            "tokenDecimals": 18,
            "tokenSymbol": "SGC"
          }).as_object().expect("Provided valid json map").clone()),
        // Extensions
        None,
    ))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    enable_println: bool,
) -> GenesisConfig {
    let built_in_evm_account =
        H160::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap();
    let mut evm_accounts = BTreeMap::new();
    evm_accounts.insert(
        built_in_evm_account,
        pallet_evm::GenesisAccount {
            nonce: U256::from(0),
            balance: U256::from(100_000_000 * DOLLARS),
            storage: Default::default(),
            code: wasm_binary.to_vec(),
        },
    );

    const ENDOWMENT: Balance = 100_000_000 * DOLLARS;

    GenesisConfig {
        frame_system: Some(SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts.iter().cloned()
            .map(|k| (k, ENDOWMENT))
            .collect(),
        }),
        pallet_aura: Some(AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
        }),
        pallet_contracts: Some(ContractsConfig {
            current_schedule: pallet_contracts::Schedule {
                enable_println, // this should only be enabled on development chains
                ..Default::default()
            },
        }),
        pallet_sudo: Some(SudoConfig {
            // Assign network admin rights.
            key: root_key,
        }),
        pallet_evm: Some(EVMConfig {
            accounts: evm_accounts,
        }),
        pallet_ethereum: Some(EthereumConfig {}),
        orml_tokens: Some(TokensConfig {
            endowed_accounts: endowed_accounts
              .iter()
              .flat_map(|x| {
                vec![
                  (x.clone(), CurrencyId::Token(TokenSymbol::DOT), 1000000 * DOLLARS),
                  (x.clone(), CurrencyId::Token(TokenSymbol::ACA), 1000000 * DOLLARS),
                  (x.clone(), CurrencyId::Token(TokenSymbol::AUSD), 1000000 * DOLLARS),
                ]
              })
              .collect(),
          }),
    }
}
