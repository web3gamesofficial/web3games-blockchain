# pallet-token-fungible

## call

### create_token

### approve

### transfer

### transfer_from

### mint

### burn

## storage

### Allowances

FungibleTokenId<===>(T::AccountId, T::AccountId) ===>Balance

Query the authorized balance

```rust
	#[pallet::storage]
	#[pallet::getter(fn allowances)]
	pub(super) type Allowances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::FungibleTokenId,
		Blake2_128Concat,
		// (owner, operator)
		(T::AccountId, T::AccountId),
		Balance,
		ValueQuery,
	>;
```

### Balances

FungibleTokenId<===>T::AccountId ===>Balance

Query account balance

```rust
	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::FungibleTokenId,
		Blake2_128Concat,
		T::AccountId,
		Balance,
		ValueQuery,
	>;
```

### Tokens

FungibleTokenId ===> Token

Query token info

```rust
	#[pallet::storage]
	pub(super) type Tokens<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::FungibleTokenId,
		Token<T::AccountId, BoundedVec<u8, T::StringLimit>>,
	>;
```

```rust
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Token<AccountId, BoundedString> {
	owner: AccountId,
	name: BoundedString,
	symbol: BoundedString,
	decimals: u8,
	total_supply: Balance,
}
```

