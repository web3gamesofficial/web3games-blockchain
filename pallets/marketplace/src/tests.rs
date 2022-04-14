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
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

#[test]
fn test_create_order_nft_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(1), 1, 1, 1));

		assert_ok!(Marketplace::create_order(
			Origin::signed(1),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			5,
			3600,
		));
	})
}

#[test]
fn test_create_order_mt_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenMulti::create_token(Origin::signed(1), 1, vec![0u8; 20]));
		assert_ok!(TokenMulti::mint(Origin::signed(1), 1, 1, 1, 100));

		assert_ok!(Marketplace::create_order(
			Origin::signed(1),
			Asset::MT(MTAsset { collection_id: 1, token_id: 1 }),
			5,
			3600,
		));
	})
}

#[test]
fn test_cancel_order_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(1), 1, 1, 1));

		assert_ok!(Marketplace::create_order(
			Origin::signed(1),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			5,
			3600,
		));

		assert_ok!(Marketplace::cancel_order(
			Origin::signed(1),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
		));
	})
}

#[test]
fn test_execute_order_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(1), 1, 1, 1));

		assert_ok!(Marketplace::create_order(
			Origin::signed(1),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			5,
			3600,
		));

		assert_ok!(Marketplace::execute_order(
			Origin::signed(2),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			5,
		));
	})
}

#[test]
fn test_place_bid_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(1), 1, 1, 1));

		assert_ok!(Marketplace::create_order(
			Origin::signed(1),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			5,
			3600,
		));

		assert_ok!(Marketplace::place_bid(
			Origin::signed(2),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			4,
			3000,
		));
	})
}

#[test]
fn test_cancel_bid_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(1), 1, 1, 1));

		assert_ok!(Marketplace::create_order(
			Origin::signed(1),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			5,
			3600,
		));

		assert_ok!(Marketplace::place_bid(
			Origin::signed(2),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			4,
			3000,
		));

		assert_ok!(Marketplace::cancel_bid(
			Origin::signed(2),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
		));
	})
}

#[test]
fn test_accept_bid_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(TokenNonFungible::create_token(
			Origin::signed(1),
			1,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 20]
		));
		assert_ok!(TokenNonFungible::mint(Origin::signed(1), 1, 1, 1));

		assert_ok!(Marketplace::create_order(
			Origin::signed(1),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			5,
			3600,
		));

		assert_ok!(Marketplace::place_bid(
			Origin::signed(2),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			4,
			3000,
		));

		assert_ok!(Marketplace::accept_bid(
			Origin::signed(1),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			4,
		));
	})
}

#[test]
fn test_set_royalties_no_root() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Marketplace::set_royalties(
				Origin::signed(1),
				Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
				Percent::from_percent(3),
				5,
			),
			BadOrigin
		);
	});
}

#[test]
fn test_set_royalties_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Marketplace::set_royalties(
			Origin::root(),
			Asset::NFT(NFTAsset { collection_id: 1, token_id: 1 }),
			Percent::from_percent(3),
			5,
		));
	});
}
