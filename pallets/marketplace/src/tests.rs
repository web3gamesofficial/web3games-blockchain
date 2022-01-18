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

use super::*;
use crate::{
	mock::*,
	NftType::{MultiToken, NonFungibleToken},
};
use frame_support::assert_ok;

#[test]
fn test_create_collection_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
		let id: u32 = 0;
		let to: u64 = 0;
		let token_id: TokenId = 0u32;
		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
		let nft_type = NonFungibleToken;
		let nft_id: u32 = 0;
		let metadata: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(Marketplace::create_collection(origin.clone(), nft_type, nft_id, metadata));
	})
}

#[test]
fn test_add_sale_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(0);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
		let id: u32 = 0;
		let to: u64 = 0;
		let token_id: TokenId = 0;
		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
		let nft_type = NonFungibleToken;
		let nft_id: u32 = 0;
		let metadata: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(Marketplace::create_collection(origin.clone(), nft_type, nft_id, metadata));
		let collection_id: u32 = 0;
		let price: Balance = 10000;
		let amount: Balance = 1;
		assert_ok!(Marketplace::add_sale(origin.clone(), collection_id, token_id, price, amount));
	})
}

#[test]
fn test_remove_sale_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(0);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
		let id: u32 = 0;
		let to: u64 = 0;
		let token_id: TokenId = 0;
		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
		let nft_type = NonFungibleToken;
		let nft_id: u32 = 0;
		let metadata: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(Marketplace::create_collection(origin.clone(), nft_type, nft_id, metadata));
		let collection_id: u32 = 0;
		let price: Balance = 10000;
		let amount: Balance = 1;
		assert_ok!(Marketplace::add_sale(
			origin.clone(),
			collection_id.clone(),
			token_id,
			price,
			amount
		));
		let sale_id: SaleId = 0;
		assert_ok!(Marketplace::remove_sale(origin.clone(), collection_id, sale_id));
	})
}

#[test]
fn test_update_price_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(0);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
		let id: u32 = 0;
		let to: u64 = 0;
		let token_id: TokenId = 0;
		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
		let nft_type = NonFungibleToken;
		let nft_id: u32 = 0;
		let metadata: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(Marketplace::create_collection(origin.clone(), nft_type, nft_id, metadata));
		let collection_id: u32 = 0;
		let price: Balance = 10000;
		let amount: Balance = 1;
		assert_ok!(Marketplace::add_sale(
			origin.clone(),
			collection_id.clone(),
			token_id,
			price,
			amount
		));
		let sale_id: SaleId = 0;
		let new_price: Balance = 20000;
		assert_ok!(Marketplace::update_price(origin.clone(), collection_id, sale_id, new_price));
	})
}

#[test]
fn test_update_amount_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(0);
		let uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenMulti::create_token(origin.clone(), uri));
		let id: u32 = 0;
		let to: u64 = 0;
		let token_id: TokenId = 0;
		let amount: Balance = 20000;
		assert_ok!(TokenMulti::mint(
			origin.clone(),
			id.clone(),
			to,
			token_id.clone(),
			amount.clone()
		));
		let nft_type = MultiToken;
		let nft_id: u32 = 0;
		let metadata: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(Marketplace::create_collection(origin.clone(), nft_type, nft_id, metadata));
		let collection_id: u32 = 0;
		let price: Balance = 10000;
		let amount: Balance = 1;
		assert_ok!(Marketplace::add_sale(origin.clone(), collection_id, token_id, price, amount));
		let sale_id: SaleId = 0;
		let new_amount: Balance = 20000;
		assert_ok!(Marketplace::update_amount(origin.clone(), collection_id, sale_id, new_amount));
	})
}

// #[test]
// fn test_offer_works() {
// 	new_test_ext().execute_with(|| {
// 		let origin = Origin::signed(0);
// 		let name: Vec<u8> = "KING".to_string().into();
// 		let symbol: Vec<u8> = "KIN".to_string().into();
// 		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
// 		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
// 		let id: u32 = 0;
// 		let to: u64 = 0;
// 		let token_id: TokenId = 0;
// 		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
// 		let nft_type = NonFungibleToken;
// 		let nft_id: u32 = 0;
// 		let metadata: Vec<u8> = "www.nft.com".to_string().into();
// 		assert_ok!(Marketplace::create_collection(origin.clone(), nft_type, nft_id, metadata));
// 		let collection_id: u32 = 0;
// 		let price: Balance = 10000;
// 		let amount: Balance = 1;
// 		assert_ok!(Marketplace::add_sale(
// 			origin.clone(),
// 			collection_id.clone(),
// 			token_id,
// 			price.clone(),
// 			amount
// 		));
//
// 		let origin1 = Origin::signed(1);
// 		let sale_id: SaleId = 0;
// 		assert_ok!(Marketplace::offer(origin1, collection_id, sale_id, price));
// 	})
// }
//
// #[test]
// fn test_accept_offer_works() {
// 	new_test_ext().execute_with(|| {
// 		let origin = Origin::signed(0);
// 		let name: Vec<u8> = "KING".to_string().into();
// 		let symbol: Vec<u8> = "KIN".to_string().into();
// 		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
// 		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
// 		let id: u32 = 0;
// 		let to: u64 = 0;
// 		let token_id: TokenId = 0;
// 		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
// 		let nft_type = NonFungibleToken;
// 		let nft_id: u32 = 0;
// 		let metadata: Vec<u8> = "www.nft.com".to_string().into();
// 		assert_ok!(Marketplace::create_collection(origin.clone(), nft_type, nft_id, metadata));
// 		let collection_id: u32 = 0;
// 		let price: Balance = 10000;
// 		let amount: Balance = 1;
// 		assert_ok!(Marketplace::add_sale(
// 			origin.clone(),
// 			collection_id.clone(),
// 			token_id,
// 			price.clone(),
// 			amount
// 		));
// 		let origin = Origin::signed(1);
// 		let sale_id: u32 = 0;
// 		let price: Balance = 2000;
// 		assert_ok!(Marketplace::offer(origin, collection_id, sale_id, price));
// 		let origin = Origin::signed(0);
// 		assert_ok!(Marketplace::accept_offer(origin, collection_id, sale_id));
// 	})
// }
//
// #[test]
// fn test_do_offer_check_works() {
// 	new_test_ext().execute_with(|| {
// 		let origin = Origin::signed(0);
// 		let name: Vec<u8> = "KING".to_string().into();
// 		let symbol: Vec<u8> = "KIN".to_string().into();
// 		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
// 		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
// 		let id: u32 = 0;
// 		let to: u64 = 0;
// 		let token_id: TokenId = 0;
// 		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
// 		let nft_type = NonFungibleToken;
// 		let nft_id: u32 = 0;
// 		let metadata: Vec<u8> = "www.nft.com".to_string().into();
// 		assert_ok!(Marketplace::create_collection(origin.clone(), nft_type, nft_id, metadata));
// 		let collection_id: u32 = 0;
// 		let price: Balance = 10000;
// 		let amount: Balance = 1;
// 		assert_ok!(Marketplace::add_sale(
// 			origin.clone(),
// 			collection_id.clone(),
// 			token_id,
// 			price.clone(),
// 			amount
// 		));
// 		let origin = Origin::signed(1);
// 		let sale_id: SaleId = 0;
// 		let price: Balance = 2000;
// 		assert_ok!(Marketplace::offer(origin, collection_id, sale_id, price));
//
// 		let origin = Origin::signed(2);
// 		let sale_id: SaleId = 0;
// 		let price: Balance = 3000;
// 		assert_ok!(Marketplace::offer(origin, collection_id, sale_id, price));
// 	})
// }

#[test]
fn test_destroy_collection_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(0);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
		let id: u32 = 0;
		let to: u64 = 0;
		let token_id: TokenId = 0;
		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
		let nft_type = NonFungibleToken;
		let nft_id: u32 = 0;
		let metadata: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(Marketplace::create_collection(origin.clone(), nft_type, nft_id, metadata));
		let collection_id: u32 = 0;
		assert_ok!(Marketplace::destroy_collection(origin.clone(), collection_id.clone()));
	})
}
