// SPDX-License-Identifier: MIT
// @custom:address 0x0000000000000000000000000000000000000403
pragma solidity ^0.8.0;

interface Exchange {
    function create_pool(uint256 token_a,uint256 token_b) external;
    function add_liquidity(uint256 pool_id,uint256 amount_a_desired,uint256 amount_b_desired,uint256 amount_a_min,uint256 amount_b_min,address to) external;
    function remove_liquidity(uint256 pool_id,uint256 token_a,uint256 token_b,uint256 liquidity,uint256 amount_a_min,uint256 amount_b_min,address to) external;
    function swap_exact_tokens_for_tokens(uint256 pool_id,uint256 amount_in,uint256 amount_out_min,uint256[] memory path,address to) external returns(bool);
    function swap_tokens_for_exact_tokens(uint256 pool_id,uint256 amount_out,uint256 amount_in_max,uint256[] memory path,address to) external returns(bool);
}
