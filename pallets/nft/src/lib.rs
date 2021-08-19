#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{Currency, Get, ReservableCurrency},
};
use sp_runtime::{
	traits::{One, Zero},
	RuntimeDebug,
};
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type CollectionId = u64;
pub type AssetId = u64;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The minimum balance to create collection
		#[pallet::constant]
		type CreateCollectionDeposit: Get<BalanceOf<Self>>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Collections<T: Config> =
		StorageMap<_, Blake2_128Concat, CollectionId, CollectionInfo<T::AccountId, BalanceOf<T>>>;

	#[pallet::storage]
	#[pallet::getter(fn next_collection_id)]
	pub(super) type NextCollectionId<T: Config> = StorageValue<_, CollectionId, ValueQuery>;

	#[pallet::storage]
	pub(super) type NftAssets<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		CollectionId,
		Blake2_128Concat,
		AssetId,
		AssetInfo<T::AccountId>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub(super) type NextAssetId<T: Config> =
		StorageMap<_, Blake2_128Concat, CollectionId, AssetId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nft_owner)]
	pub(super) type NftOwner<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		(CollectionId, AssetId),
		(),
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CollectionCreated(CollectionId, T::AccountId),
		TokenMint(CollectionId, Vec<AssetId>, T::AccountId),
		TokenBurned(CollectionId, AssetId, T::AccountId),
		TokenTransferred(CollectionId, AssetId, T::AccountId, T::AccountId),
		CollectionDestroyed(CollectionId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NumOverflow,
		NoAvailableCollectionId,
		CollectionNotFound,
		NoAvailableAssetId,
		AssetNotFound,
		InvalidQuantity,
		NoPermission,
		CannotDestroyCollection,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_collection(
			origin: OriginFor<T>,
			properties: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_create_collection(&who, properties)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			metadata: Vec<u8>,
			quantity: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_mint(&who, collection_id, metadata, quantity)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			asset_id: AssetId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_burn(&who, collection_id, asset_id)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn destroy_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_destroy_collection(&who, collection_id)?;

			Ok(().into())
		}
	}
}

/// Collection info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct CollectionInfo<AccountId, Balance> {
	/// Class owner
	pub owner: AccountId,
	/// Total issuance for the class
	pub total_supply: u128,
	/// Minimum balance to create a collection
	pub deposit: Balance,
	/// Metadata from ipfs
	pub properties: Vec<u8>,
}

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct AssetInfo<AccountId> {
	/// Asset owner
	pub owner: AccountId,
	/// Metadata from ipfs
	pub metadata: Vec<u8>,
}

impl<T: Config> Pallet<T> {
	pub fn do_create_collection(
		who: &T::AccountId,
		properties: Vec<u8>,
	) -> Result<CollectionId, DispatchError> {
		let collection_id =
			NextCollectionId::<T>::try_mutate(|id| -> Result<CollectionId, DispatchError> {
				let current_id = *id;
				*id = id
					.checked_add(One::one())
					.ok_or(Error::<T>::NoAvailableCollectionId)?;
				Ok(current_id)
			})?;

		let deposit = T::CreateCollectionDeposit::get();
		T::Currency::reserve(who, deposit.clone())?;

		let collection_info = CollectionInfo {
			owner: who.clone(),
			total_supply: Default::default(),
			deposit,
			properties,
		};

		Collections::<T>::insert(collection_id, collection_info);

		Self::deposit_event(Event::CollectionCreated(collection_id, who.clone()));
		Ok(collection_id)
	}

	pub fn do_mint(
		to: &T::AccountId,
		collection_id: CollectionId,
		metadata: Vec<u8>,
		quantity: u32,
	) -> Result<Vec<AssetId>, DispatchError> {
		NextAssetId::<T>::try_mutate(collection_id, |id| -> Result<Vec<AssetId>, DispatchError> {
			ensure!(quantity >= 1u32, Error::<T>::InvalidQuantity);
			let next_id = *id;
			*id = id
				.checked_add(quantity as u64)
				.ok_or(Error::<T>::NoAvailableAssetId)?;

			let mut asset_ids: Vec<AssetId> = Vec::new();
			Collections::<T>::try_mutate(collection_id, |collection_info| -> DispatchResult {
				let info = collection_info
					.as_mut()
					.ok_or(Error::<T>::CollectionNotFound)?;

				ensure!(*to == info.owner, Error::<T>::NoPermission);

				let asset_info = AssetInfo {
					owner: to.clone(),
					metadata,
				};

				for i in 0..quantity {
					let asset_id = next_id + i as u64;
					asset_ids.push(asset_id);

					NftAssets::<T>::insert(collection_id, asset_id, asset_info.clone());
					NftOwner::<T>::insert(to, (collection_id, asset_id), ());
				}

				info.total_supply = info
					.total_supply
					.checked_add((quantity as u128).into())
					.ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;

			Self::deposit_event(Event::TokenMint(
				collection_id,
				asset_ids.clone(),
				to.clone(),
			));
			Ok(asset_ids)
		})
	}

	pub fn do_burn(
		who: &T::AccountId,
		collection_id: CollectionId,
		asset_id: AssetId,
	) -> DispatchResult {
		NftAssets::<T>::try_mutate_exists(collection_id, asset_id, |asset_info| -> DispatchResult {
			let info = asset_info.take().ok_or(Error::<T>::AssetNotFound)?;
			ensure!(info.owner == *who, Error::<T>::NoPermission);

			Collections::<T>::try_mutate(collection_id, |collection_info| -> DispatchResult {
				let info = collection_info
					.as_mut()
					.ok_or(Error::<T>::CollectionNotFound)?;
				info.total_supply = info
					.total_supply
					.checked_sub(One::one())
					.ok_or(Error::<T>::NumOverflow)?;
				Ok(())
			})?;

			NftOwner::<T>::remove(who, (collection_id, asset_id));

			Self::deposit_event(Event::TokenBurned(collection_id, asset_id, who.clone()));
			Ok(())
		})
	}

	pub fn do_transfer_from(
		from: &T::AccountId,
		to: &T::AccountId,
		collection_id: CollectionId,
		asset_id: AssetId,
	) -> DispatchResult {
		NftAssets::<T>::try_mutate(collection_id, asset_id, |asset_info| -> DispatchResult {
			let info = asset_info.as_mut().ok_or(Error::<T>::AssetNotFound)?;
			ensure!(info.owner == *from, Error::<T>::NoPermission);
			if from == to {
				return Ok(());
			}

			info.owner = to.clone();

			NftOwner::<T>::remove(from, (collection_id, asset_id));
			NftOwner::<T>::insert(to, (collection_id, asset_id), ());

			Self::deposit_event(Event::TokenTransferred(
				collection_id,
				asset_id,
				from.clone(),
				to.clone(),
			));

			Ok(())
		})
	}

	pub fn do_destroy_collection(
		who: &T::AccountId,
		collection_id: CollectionId,
	) -> DispatchResult {
		Collections::<T>::try_mutate_exists(collection_id, |collection_info| -> DispatchResult {
			let info = collection_info
				.take()
				.ok_or(Error::<T>::CollectionNotFound)?;
			ensure!(info.owner == *who, Error::<T>::NoPermission);
			ensure!(
				info.total_supply == Zero::zero(),
				Error::<T>::CannotDestroyCollection
			);

			NextAssetId::<T>::remove(collection_id);

			T::Currency::unreserve(who, info.deposit);

			Self::deposit_event(Event::CollectionDestroyed(collection_id, who.clone()));

			Ok(())
		})
	}
}
