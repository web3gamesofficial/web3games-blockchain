// SPDX-License-Identifier: MIT
// @custom:address 0x0000000000000000000000000000000000000406
pragma solidity ^0.8.0;

interface Launchpad {
    function create_pool(uint256 sale_start,uint256 sale_duration,uint256 sale_token_id,uint256 buy_token_id,uint256 total_sale_amount,uint256 token_price) external;
    function buy_token(uint256 pool_id,uint256 amount) external;
    function owner_claim(uint256 pool_id) external;
    function claim(uint256 pool_id) external;
}
