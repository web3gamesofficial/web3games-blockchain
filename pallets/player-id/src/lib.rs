#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_std::{convert::TryFrom, prelude::*};
use sp_std::vec::Vec;

pub use weights::WeightInfo;

pub mod weights;

// pub mod ethereum;


type Address = Vec<u8>;
type PlayerID = Vec<u8>;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo, Copy)]
pub enum Chain {
	Polkadot,
	Ethereum,
	BSC
}


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_core::crypto::AccountId32;
	use crate::WeightInfo;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn playerchain)]
	pub(super) type PlayerChain<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Address,
		Blake2_128Concat,
		Chain,
		PlayerID
	>;

	#[pallet::storage]
	#[pallet::getter(fn chainplayer)]
	pub(super) type ChainPlayer<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		PlayerID,
		Blake2_128Concat,
		Chain,
		Vec<Address>
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PlayerIDCreated(T::AccountId,Address,Chain,PlayerID),
	}

	#[pallet::error]
	pub enum Error<T> {
		Repeat
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_player_id(
			origin: OriginFor<T>,
			address:Vec<u8>,
			chain:Chain,
			player_id:PlayerID
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(PlayerChain::<T>::get(address.clone(),chain) != Some(player_id.clone()),Error::<T>::Repeat);

			let all_address = ChainPlayer::<T>::get(player_id.clone(),chain);

			match all_address {
				Some(mut all_address_list) =>{
					all_address_list.push(player_id.clone());
					PlayerChain::<T>::insert(address.clone(),chain,player_id.clone());
					ChainPlayer::<T>::insert(player_id.clone(),chain,all_address_list);
					Self::deposit_event(Event::PlayerIDCreated(who,address,chain.clone(),player_id.clone()));
				}
				None =>{
					let new_address_list = vec![address.clone()];
					PlayerChain::<T>::insert(address.clone(),chain,player_id.clone());
					ChainPlayer::<T>::insert(player_id.clone(),chain,new_address_list);
					Self::deposit_event(Event::PlayerIDCreated(who,address,chain.clone(),player_id.clone()));
				}
			}
			Ok(().into())
		}

		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn valid_signature(
		// 	origin: OriginFor<T>,
		// 	signer: AccountId32,
		// 	signature: EthereumSignature,
		// 	msg:Vec<u8>
		// ) -> DispatchResult {
		// 	let _who = ensure_signed(origin)?;
		// 	let _result = signature.verify(msg.as_slice(), &signer);
		// 	Ok(())
		// }
	}
}
