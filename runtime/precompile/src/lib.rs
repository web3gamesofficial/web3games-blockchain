#![cfg_attr(not(feature = "std"), no_std)]

mod erc1155;

use pallet_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
use erc1155::Erc1155;

pub type SgcPrecompiles<Runtime> = (
	ECRecover,
	Sha256,
	Ripemd160,
	Identity,
	Erc1155<Runtime>,
);
