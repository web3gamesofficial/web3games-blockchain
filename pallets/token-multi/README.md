# pallet-token-multi-fungible

## call

### create_token

### set_approve_for_all

### transfer_from

### batch_transfer_from

### mint

### mint_batch

### burn

### burn_batch

## storage

### Balances

MultiTokenId<===>(T::TokenId, T::AccountId) ===> Balance

Query balance based on NonFungibleTokenId

```rust
	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::MultiTokenId,
		Blake2_128Concat,
		(T::TokenId, T::AccountId),
		Balance,
		ValueQuery,
	>;
```

### OperatorApprovals

MultiTokenId<===>(T::AccountId, T::AccountId) ===> bool

Query if it is an operator based on NonFungibleTokenId

```rust
	#[pallet::storage]
	#[pallet::getter(fn is_approved_for_all)]
	pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::MultiTokenId,
		Blake2_128Concat,
		// (owner, operator)
		(T::AccountId, T::AccountId),
		bool,
		ValueQuery,
	>;
```

### Tokens

NonFungibleTokenId ===> Token

Query token info

```rust
	#[pallet::storage]
	pub(super) type Tokens<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::MultiTokenId,
		Token<T::AccountId, BoundedVec<u8, T::StringLimit>>,
	>;
```

```rust
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Token<AccountId, BoundedString> {
	owner: AccountId,
	uri: BoundedString,
	total_supply: Balance,
}
```
