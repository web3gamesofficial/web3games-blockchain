# pallet-token-non-fungible

## call

### create_token

### approve

### set_approve_for_all

### transfer_from

### mint

### burn

## storage

### AllTokens

NonFungibleTokenId<===>TokenIndex ===>TokenId

Query TokenId based on TokenIndex

```rust
	#[pallet::storage]
	pub(super) type AllTokens<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		TokenIndex,
		T::TokenId,
		ValueQuery,
	>;
```

### AllTokensIndex

NonFungibleTokenId<===>TokenId ===> TokenIndex

Query TokenIndex based on TokenId

```rust
	#[pallet::storage]
	pub(super) type AllTokensIndex<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NonFungibleTokenId,
		Blake2_128Concat,
		T::TokenId,
		TokenIndex,
		ValueQuery,
	>;
```

### Balances

NonFungibleTokenId<===>AccountId ===> u32

Query balance based on NonFungibleTokenId

```rust
#[pallet::storage]
#[pallet::getter(fn balance_of)]
pub(super) type Balances<T: Config> = StorageDoubleMap<
   _,
   Blake2_128Concat,
   T::NonFungibleTokenId,
   Blake2_128Concat,
   T::AccountId,
   u32,
   ValueQuery,
>;
```

### OperatorApprovals

NonFungibleTokenId<===>(T::AccountId, T::AccountId) ===> bool

Query if it is an operator based on NonFungibleTokenId

```rust
#[pallet::storage]
#[pallet::getter(fn is_approved_for_all)]
pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
   _,
   Blake2_128Concat,
   T::NonFungibleTokenId,
   Blake2_128Concat,
   // (owner, operator)
   (T::AccountId, T::AccountId),
   bool,
   ValueQuery,
>;
```

### OwnedTokens

NonFungibleTokenId<===>(T::AccountId, TokenIndex) ===> TokenId

Query the user's tokenId based on TokenIndex

```rust
#[pallet::storage]
pub(super) type OwnedTokens<T: Config> = StorageDoubleMap<
   _,
   Blake2_128Concat,
   T::NonFungibleTokenId,
   Blake2_128Concat,
   (T::AccountId, TokenIndex),
   T::TokenId,
   ValueQuery,
>;
```

### OwnedTokensIndex

NonFungibleTokenId<===>(T::AccountId, TokenId) ===> TokenIndex

Query the user's TokenIndex based on TokenId

```rust
#[pallet::storage]
pub(super) type OwnedTokensIndex<T: Config> = StorageDoubleMap<
   _,
   Blake2_128Concat,
   T::NonFungibleTokenId,
   Blake2_128Concat,
   (T::AccountId, T::TokenId),
   TokenIndex,
   ValueQuery,
>;
```

### owners

NonFungibleTokenId<===>TokenId ,  ===> AccountId

Query the TokenId's owner

```rust
#[pallet::storage]
#[pallet::getter(fn owner_of)]
pub(super) type Owners<T: Config> = StorageDoubleMap<
	_,
	Blake2_128Concat,
	T::NonFungibleTokenId,
	Blake2_128Concat,
	T::TokenId,
	T::AccountId,
	OptionQuery,
	GetDefault,
	ConstU32<300_000>,
>;
```

### TokenApprovals

NonFungibleTokenId<===>TokenId ,  ===> AccountId

Query the TokenId's approver

```rust
#[pallet::storage]
#[pallet::getter(fn get_approved)]
pub(super) type TokenApprovals<T: Config> = StorageDoubleMap<
   _,
   Blake2_128Concat,
   T::NonFungibleTokenId,
   Blake2_128Concat,
   T::TokenId,
   T::AccountId,
   OptionQuery,
   GetDefault,
   ConstU32<300_000>,
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
		T::NonFungibleTokenId,
		Token<T::AccountId, BoundedVec<u8, T::StringLimit>>,
	>;
```

```rust
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Token<AccountId, BoundedString> {
	owner: AccountId,
	name: BoundedString,
	symbol: BoundedString,
	base_uri: BoundedString,
}
```

### TotalSupply

NonFungibleTokenId ===> u32

Query token totalSupply

```rust
#[pallet::storage]
pub(super) type TotalSupply<T: Config> =
   StorageMap<_, Blake2_128Concat, T::NonFungibleTokenId, u32, ValueQuery>;
```
