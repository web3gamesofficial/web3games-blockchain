use super::*;
use crate::{mock::*};
use frame_support::{assert_ok};

#[test]
fn test_create_token_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name:Vec<u8> = "KING".to_string().into();
		let symbol:Vec<u8> = "KIN".to_string().into();
		let decimals:u8 = 2;
		assert_ok!(TokenFungible::create_token(origin,name,symbol,decimals));
		assert_eq!(TokenFungible::next_token_id(),1)
	})
}

#[test]
fn test_create_token_not_works_by_bad_metadata() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name:Vec<u8> = vec![1,2,3];
		let symbol:Vec<u8> = "KIN".to_string().into();
		let decimals:u8 = 2;
		assert_ok!(TokenFungible::create_token(origin,name,symbol,decimals));
		// assert_eq!(TokenFungible::next_token_id(),1)

	})
}


#[test]
fn test_mint_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let id:u32 = 0;
		let account:u64= 1;
		let amount:Balance = 1u128;
		let name:Vec<u8> = "KING".to_string().into();
		let symbol:Vec<u8> = "KIN".to_string().into();
		let decimals:u8 = 2;
		assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
		assert_eq!(TokenFungible::next_token_id(),1);
		assert_ok!(TokenFungible::mint(origin,id,account,amount));
		assert_eq!(TokenFungible::balance_of(id,account),1);
	})

}



#[test]
fn test_approve_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name:Vec<u8> = "KING".to_string().into();
		let symbol:Vec<u8> = "KIN".to_string().into();
		let decimals:u8 = 2;
		assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
		assert_eq!(TokenFungible::next_token_id(),1);
		let id:u32 = 0;
		let account:u64= 1;
		let amount:Balance = 1u128;
		assert_ok!(TokenFungible::mint(origin.clone(),id.clone(),account,amount.clone()));
		assert_eq!(TokenFungible::balance_of(id,account),1);
		let spender:u64= 2;
		assert_ok!(TokenFungible::approve(origin,id,spender,amount));
		assert_eq!(TokenFungible::allowances(id,(account,spender)),1);
	})

}




#[test]
fn test_transfer_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name:Vec<u8> = "KING".to_string().into();
		let symbol:Vec<u8> = "KIN".to_string().into();
		let decimals:u8 = 2;
		assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
		assert_eq!(TokenFungible::next_token_id(),1);
		let id:u32 = 0;
		let account:u64= 1;
		let amount:Balance = 1u128;
		assert_ok!(TokenFungible::mint(origin.clone(),id.clone(),account,amount.clone()));
		assert_eq!(TokenFungible::balance_of(id,account),1);
		let recipient:u64= 2;
		assert_ok!(TokenFungible::transfer(origin,id,recipient,amount));
		assert_eq!(TokenFungible::balance_of(id,account),0);
		assert_eq!(TokenFungible::balance_of(id,recipient),1);
	})

}

#[test]
fn test_transfer_from_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name:Vec<u8> = "KING".to_string().into();
		let symbol:Vec<u8> = "KIN".to_string().into();
		let decimals:u8 = 2;
		assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
		assert_eq!(TokenFungible::next_token_id(),1);
		let id:u32 = 0;
		let account:u64= 1;
		let amount:Balance = 1u128;
		assert_ok!(TokenFungible::mint(origin.clone(),id.clone(),account,amount.clone()));
		assert_eq!(TokenFungible::balance_of(id,account),1);
		let spender:u64 = 2;
		assert_ok!(TokenFungible::approve(origin.clone(),id,spender,amount.clone()));
		assert_eq!(TokenFungible::allowances(id,(account,spender)),1);
		let origin2 = Origin::signed(2);
		let sender:u64= 1;
		let recipient:u64= 3;
		assert_ok!(TokenFungible::transfer_from(origin2,id,sender,recipient,amount));
		assert_eq!(TokenFungible::balance_of(id,recipient),1);
	})
}

#[test]
fn test_burn_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name:Vec<u8> = "KING".to_string().into();
		let symbol:Vec<u8> = "KIN".to_string().into();
		let decimals:u8 = 2;
		assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
		assert_eq!(TokenFungible::next_token_id(),1);
		let id:u32 = 0;
		let account:u64= 1;
		let amount:Balance = 1u128;
		assert_ok!(TokenFungible::mint(origin.clone(),id.clone(),account,amount.clone()));
		assert_eq!(TokenFungible::balance_of(id,account),1);
		assert_ok!(TokenFungible::burn(origin,id,amount));
		assert_eq!(TokenFungible::balance_of(id,account),0);
	})

}