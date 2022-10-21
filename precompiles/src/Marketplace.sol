// SPDX-License-Identifier: MIT
// @custom:address 0x0000000000000000000000000000000000000404
pragma solidity ^0.8.0;

interface Marketplace {
    function create_order(uint256 group_id,uint256 token_id,uint256 asset_type,uint256 price,uint256 duration) external;
    function cancel_order(uint256 group_id,uint256 token_id,uint256 asset_type) external;
    function execute_order(uint256 group_id,uint256 token_id,uint256 asset_type) external;
    function place_bid(uint256 group_id,uint256 token_id,uint256 asset_type,uint256 price,uint256 duration) external;
    function cancel_bid(uint256 group_id,uint256 token_id,uint256 asset_type) external;
    function accept_bid(uint256 group_id,uint256 token_id,uint256 asset_type) external;
}
