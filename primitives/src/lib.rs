#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::{
	generic,
	traits::{Verify, BlakeTwo256, IdentifyAccount},
	OpaqueExtrinsic, MultiSignature, RuntimeDebug,
};
use sp_std::{
    convert::{Into, TryFrom, TryInto},
    prelude::*,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Timestamp = u64;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;
/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;

/// Signed version of Balance
pub type Amount = i128;

///Currency ID
// pub type CurrencyId = u64;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenSymbol {
	SGC = 0,
	DOT = 1,
	ACA = 2,
    AUSD = 3,
}

impl TryFrom<u8> for TokenSymbol {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(TokenSymbol::SGC),
            1 => Ok(TokenSymbol::DOT),
            2 => Ok(TokenSymbol::ACA),
            3 => Ok(TokenSymbol::AUSD),
            _ => Err(()),
        }
    }
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
    Token(TokenSymbol),
}

impl CurrencyId {
    pub fn is_token_currency_id(&self) -> bool {
        matches!(self, CurrencyId::Token(_))
    }
}

impl TryFrom<Vec<u8>> for CurrencyId {
    type Error = ();
    fn try_from(v: Vec<u8>) -> Result<CurrencyId, ()> {
        match v.as_slice() {
            b"SGC" => Ok(CurrencyId::Token(TokenSymbol::SGC)),
            b"DOT" => Ok(CurrencyId::Token(TokenSymbol::DOT)),
            b"ACA" => Ok(CurrencyId::Token(TokenSymbol::ACA)),
            b"AUSD" => Ok(CurrencyId::Token(TokenSymbol::AUSD)),
            _ => Err(()),
        }
    }
}

/// Note the pre-deployed ERC20 contracts depend on `CurrencyId` implementation,
/// and need to be updated if any change.
impl TryFrom<[u8; 32]> for CurrencyId {
    type Error = ();

    fn try_from(v: [u8; 32]) -> Result<Self, Self::Error> {
        if !v.starts_with(&[0u8; 29][..]) {
            return Err(());
        }

        // token
        if v[29] == 0 && v[31] == 0 {
            return v[30].try_into().map(CurrencyId::Token);
        }

        Err(())
    }
}

/// Note the pre-deployed ERC20 contracts depend on `CurrencyId` implementation,
/// and need to be updated if any change.
impl Into<[u8; 32]> for CurrencyId {
    fn into(self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        match self {
            CurrencyId::Token(token) => {
                bytes[30] = token as u8;
            }
        }
        bytes
    }
}

impl Into<u64> for CurrencyId {
	fn into(self) -> u64 {
		match self {
			CurrencyId::Token(token) => {
				token as u64
			}
		}
	}
}
