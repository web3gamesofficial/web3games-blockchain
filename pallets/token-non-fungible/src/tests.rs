use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

pub type TokenId = u32;

#[test]
fn test_create_token_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
	})
}

#[test]
fn test_mint_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_id: TokenId = 1u32;
		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
	})
}

#[test]
fn test_approve_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_id: TokenId = 1u32;
		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
		let to: u64 = 2;
		assert_ok!(TokenNonFungible::approve(origin, id, to, token_id));
	})
}

#[test]
fn test_set_approve_for_all_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_id: TokenId = 1u32;
		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
		let operator: u64 = 2;
		let approved: bool = true;
		assert_ok!(TokenNonFungible::set_approve_for_all(origin, id, operator, approved));
	})
}

#[test]
fn test_transfer_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_id: TokenId = 1u32;
		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
		let to: u64 = 2;
		assert_ok!(TokenNonFungible::transfer(origin, id, to, token_id));
	})
}

#[test]
fn test_transfer_from_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_id: TokenId = 1u32;
		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
		let from: u64 = 1;
		let to: u64 = 2;
		assert_ok!(TokenNonFungible::transfer_from(origin, id, from, to, token_id));
	})
}

#[test]
fn test_burn_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name: Vec<u8> = "KING".to_string().into();
		let symbol: Vec<u8> = "KIN".to_string().into();
		let base_uri: Vec<u8> = "www.nft.com".to_string().into();
		assert_ok!(TokenNonFungible::create_token(origin.clone(), name, symbol, base_uri));
		let id: u32 = 0;
		let to: u64 = 1;
		let token_id: TokenId = 1u32;
		assert_ok!(TokenNonFungible::mint(origin.clone(), id.clone(), to, token_id.clone()));
		assert_ok!(TokenNonFungible::burn(origin, id, token_id));
	})
}
