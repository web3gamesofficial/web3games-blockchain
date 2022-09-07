// SPDX-License-Identifier: MIT
// @custom:address 0x0000000000000000000000000000000000000403
pragma solidity ^0.8.0;

interface Exchange {
    function create_pool(uint256 token_a,uint256 token_b) external;
    function add_liquidity(uint256 token_a,uint256 token_b,uint256 amount_a_desired,uint256 amount_b_desired,uint256 amount_a_min,uint256 amount_b_min,address to,uint256 deadline) external;
    function add_liquidity_w3g(uint256 token,uint256 amount_w3g_desired,uint256 amount_desired,uint256 amount_w3g_min,uint256 amount_min,address to,uint256 deadline) external;
    function remove_liquidity(uint256 token_a,uint256 token_b,uint256 liquidity,uint256 amount_a_min,uint256 amount_b_min,address to,uint256 deadline) external;
    function remove_liquidity_w3g(uint256 token,uint256 liquidity,uint256 amount_w3g_min,uint256 amount_min,address to,uint256 deadline) external;
    function swap_exact_tokens_for_tokens(uint256 amount_in,uint256 amount_out_min,uint256[] memory path,address to,uint256 deadline) external;
    function swap_exact_w3g_for_tokens(uint256 amount_in_w3g,uint256 amount_out_min,uint256[] memory path,address to,uint256 deadline) external;
    function swap_tokens_for_exact_tokens(uint256 amount_out,uint256 amount_in_max,uint256[] memory path,address to,uint256 deadline) external;
    function swap_tokens_for_exact_w3g(uint256 amount_out_w3g,uint256 amount_in_max,uint256[] memory path,address to,uint256 deadline) external;
}
