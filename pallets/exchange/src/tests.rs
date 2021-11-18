use super::*;
use crate::{mock::*};
use frame_support::{assert_ok};


pub type FungibleTokenId = u32;
pub type PoolId = u32;

#[test]
fn test_create_pool_works() {
    new_test_ext().execute_with(|| {
        let origin = Origin::signed(1);
        let name:Vec<u8> = "Test".to_string().into();
        let symbol:Vec<u8> = "TST".to_string().into();
        let decimals:u8 = 2;
        assert_ok!(TokenFungible::create_token(origin,name,symbol,decimals));
        assert_eq!(TokenFungible::next_token_id(),1);
        let origin = Origin::signed(2);
        let name:Vec<u8> = "KING".to_string().into();
        let symbol:Vec<u8> = "KIN".to_string().into();
        let decimals:u8 = 2;
        assert_ok!(TokenFungible::create_token(origin,name,symbol,decimals));
        assert_eq!(TokenFungible::next_token_id(),2);
        let who = &1u64;
        let token_a:FungibleTokenId = 0;
        let token_b:FungibleTokenId = 1;
        assert_ok!(Exchange::do_create_pool(who,token_a,token_b));
        assert_eq!(Exchange::get_pool((token_a,token_b)),0);
        assert_eq!(Exchange::next_pool_id(),1);
    })
}

#[test]
fn test_add_liquidity_works() {
    new_test_ext().execute_with(|| {
        let origin = Origin::signed(1);
        let name:Vec<u8> = "Test".to_string().into();
        let symbol:Vec<u8> = "TST".to_string().into();
        let decimals:u8 = 18;
        assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
        assert_eq!(TokenFungible::next_token_id(),1);
        let id:u32 = 0;
        let account:u64= 1;
        let amount:Balance = 1000000u128;
        assert_ok!(TokenFungible::mint(origin,id,account,amount));
        assert_eq!(TokenFungible::balance_of(id,account),1000000);

        let origin = Origin::signed(2);
        let name:Vec<u8> = "KING".to_string().into();
        let symbol:Vec<u8> = "KIN".to_string().into();
        let decimals:u8 = 18;
        assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
        assert_eq!(TokenFungible::next_token_id(),2);

        let id:u32 = 1;
        let account:u64= 1;
        let amount:Balance = 1000000u128;
        assert_ok!(TokenFungible::mint(origin,id,account,amount));
        assert_eq!(TokenFungible::balance_of(id,account),1000000);


        let who = &1u64;
        let token_a:FungibleTokenId = 0;
        let token_b:FungibleTokenId = 1;
        assert_ok!(Exchange::do_create_pool(who,token_a,token_b));
        assert_eq!(Exchange::get_pool((token_a,token_b)),0);
        assert_eq!(Exchange::next_pool_id(),1);

        let origin = Origin::signed(1);
        let id:PoolId = 0;
        let amount_a_desired :Balance = 1000000u128;
        let amount_b_desired :Balance = 1000000u128;
        let amount_a_min :Balance = 0u128;
        let amount_b_min :Balance = 0u128;
        let to = 0u64;
        assert_ok!(Exchange::add_liquidity(origin,id,amount_a_desired,amount_b_desired,amount_a_min,amount_b_min,to));
        assert_eq!(TokenFungible::balance_of(2,0),1000000);
    })
}


#[test]
fn test_swap_exact_tokens_for_tokens_works() {
    new_test_ext().execute_with(|| {
        let origin = Origin::signed(1);
        let name:Vec<u8> = "Test".to_string().into();
        let symbol:Vec<u8> = "TST".to_string().into();
        let decimals:u8 = 18;
        assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
        assert_eq!(TokenFungible::next_token_id(),1);
        let id:u32 = 0;
        let account:u64= 1;
        let amount:Balance = 1200000u128;
        assert_ok!(TokenFungible::mint(origin,id,account,amount));
        assert_eq!(TokenFungible::balance_of(id,account),1200000);

        let origin = Origin::signed(2);
        let name:Vec<u8> = "KING".to_string().into();
        let symbol:Vec<u8> = "KIN".to_string().into();
        let decimals:u8 = 18;
        assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
        assert_eq!(TokenFungible::next_token_id(),2);

        let id:u32 = 1;
        let account:u64= 1;
        let amount:Balance = 1000000u128;
        assert_ok!(TokenFungible::mint(origin,id,account,amount));
        assert_eq!(TokenFungible::balance_of(id,account),1000000);


        let who = &1u64;
        let token_a:FungibleTokenId = 0;
        let token_b:FungibleTokenId = 1;
        assert_ok!(Exchange::do_create_pool(who,token_a,token_b));
        assert_eq!(Exchange::get_pool((token_a,token_b)),0);
        assert_eq!(Exchange::next_pool_id(),1);

        let origin = Origin::signed(1);
        let id:PoolId = 0;
        let amount_a_desired :Balance = 1000000u128;
        let amount_b_desired :Balance = 1000000u128;
        let amount_a_min :Balance = 0u128;
        let amount_b_min :Balance = 0u128;
        let to = 0u64;
        assert_ok!(Exchange::add_liquidity(origin.clone(),id.clone(),amount_a_desired,amount_b_desired,amount_a_min,amount_b_min,to));
        assert_eq!(TokenFungible::balance_of(2,0),1000000);
        println!("{}",TokenFungible::balance_of(0,1));
        let amount_in: Balance = 100000u128;
        let amount_out_min: Balance = 90661u128;
        let path:Vec<FungibleTokenId> = vec![0,1];
        let to:u64 = 1;
        assert_ok!(Exchange::swap_exact_tokens_for_tokens(origin,id,amount_in,amount_out_min,path,to));
    })
}

#[test]
fn test_swap_tokens_for_exact_tokens_works() {
    new_test_ext().execute_with(|| {
        let origin = Origin::signed(1);
        let name:Vec<u8> = "Test".to_string().into();
        let symbol:Vec<u8> = "TST".to_string().into();
        let decimals:u8 = 18;
        assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
        assert_eq!(TokenFungible::next_token_id(),1);
        let id:u32 = 0;
        let account:u64= 1;
        let amount:Balance = 1200000u128;
        assert_ok!(TokenFungible::mint(origin,id,account,amount));
        assert_eq!(TokenFungible::balance_of(id,account),1200000);

        let origin = Origin::signed(2);
        let name:Vec<u8> = "KING".to_string().into();
        let symbol:Vec<u8> = "KIN".to_string().into();
        let decimals:u8 = 18;
        assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
        assert_eq!(TokenFungible::next_token_id(),2);

        let id:u32 = 1;
        let account:u64= 1;
        let amount:Balance = 1000000u128;
        assert_ok!(TokenFungible::mint(origin,id,account,amount));
        assert_eq!(TokenFungible::balance_of(id,account),1000000);


        let who = &1u64;
        let token_a:FungibleTokenId = 0;
        let token_b:FungibleTokenId = 1;
        assert_ok!(Exchange::do_create_pool(who,token_a,token_b));
        assert_eq!(Exchange::get_pool((token_a,token_b)),0);
        assert_eq!(Exchange::next_pool_id(),1);

        let origin = Origin::signed(1);
        let id:PoolId = 0;
        let amount_a_desired :Balance = 1000000u128;
        let amount_b_desired :Balance = 1000000u128;
        let amount_a_min :Balance = 0u128;
        let amount_b_min :Balance = 0u128;
        let to = 0u64;
        assert_ok!(Exchange::add_liquidity(origin.clone(),id.clone(),amount_a_desired,amount_b_desired,amount_a_min,amount_b_min,to));
        assert_eq!(TokenFungible::balance_of(2,0),1000000);
        println!("{}",TokenFungible::balance_of(0,1));
        let amount_out: Balance = 90661u128;
        let amount_in_max: Balance = 100000u128;
        let path:Vec<FungibleTokenId> = vec![0,1];
        let to:u64 = 1;
        assert_ok!(Exchange::swap_tokens_for_exact_tokens(origin,id,amount_out,amount_in_max,path,to));
    })
}
