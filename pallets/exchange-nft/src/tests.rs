// This file is part of Web3Games.

// Copyright (C) 2021 Web3Games https://web3games.org
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
use crate::mock::*;
use frame_support::assert_ok;

type MultiTokenId = u32;
type FungibleTokenId = u32;

#[test]
fn test_create_pool_works() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name: Vec<u8> = "Test".to_string().into();
		let symbol: Vec<u8> = "TST".to_string().into();
		let decimals: u8 = 2;
		assert_ok!(TokenFungible::create_token(origin, name, symbol, decimals));
		assert_eq!(TokenFungible::next_token_id(), 1);
		let origin = Origin::signed(2);
		let uri: Vec<u8> = "KING".to_string().into();
		assert_ok!(TokenMulti::create_token(origin.clone(), uri));
		assert_eq!(TokenMulti::next_token_id(), 1);
		let origin = Origin::signed(1);
		let currency: FungibleTokenId = 0;
		let token: MultiTokenId = 0;
		assert_ok!(ExchangeNft::create_pool(origin, currency, token));
		assert_eq!(ExchangeNft::get_pool((currency, token)), 0);
		assert_eq!(ExchangeNft::next_pool_id(), 1);
	})
}

// #[test]
// fn test_add_liquidity_works() {
//     new_test_ext().execute_with(|| {
//         let origin = Origin::signed(1);
//         let name:Vec<u8> = "Test".to_string().into();
//         let symbol:Vec<u8> = "TST".to_string().into();
//         let decimals:u8 = 2;
//         assert_ok!(TokenFungible::create_token(origin.clone(),name,symbol,decimals));
//         assert_eq!(TokenFungible::next_token_id(),1);
//
//         let id:u32 = 0;
//         let account:u64= 1;
//         let amount:Balance = 1000000u128;
//         assert_ok!(TokenFungible::mint(origin,id,account,amount));
//         assert_eq!(TokenFungible::balance_of(id,account),1000000);
//
//
//         let origin = Origin::signed(2);
//         let uri:Vec<u8> = "KING".to_string().into();
//         assert_ok!(TokenMulti::create_token(origin.clone(),uri));
//         assert_eq!(TokenMulti::next_token_id(),1);
//
//         let id:u32 = 0;
//         let to:u64 = 1;
//         let token_id:TokenId= 1;
//         let amount:Balance = 1000000u128;
//         assert_ok!(TokenMulti::mint(origin.clone(),id,to,token_id,amount));
//         let token_id = 1;
//         assert_eq!(TokenMulti::balance_of(id,(token_id,account)),1000000);
//
//
//         let origin = Origin::signed(1);
//         let currency:FungibleTokenId = 0;
//         let token:MultiTokenId = 0;
//         assert_ok!(ExchangeNft::create_pool(origin,currency,token));
//         assert_eq!(ExchangeNft::get_pool((currency,token)),0);
//         assert_eq!(ExchangeNft::next_pool_id(),1);
//
//         let origin = Origin::signed(1);
//         let id:PoolId = 0;
//         let token_ids :Vec<TokenId> = vec![0];
//         let token_amounts :Vec<Balance> = vec![1000000];
//         let max_currencies :Vec<Balance> = vec![1000000];
//         assert_ok!(ExchangeNft::add_liquidity(origin,id,token_ids,token_amounts,max_currencies));
//         // assert_eq!(TokenFungible::balance_of(2,0),1000000);
//     })
// }
