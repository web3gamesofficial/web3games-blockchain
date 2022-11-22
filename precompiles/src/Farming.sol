// SPDX-License-Identifier: MIT
// @custom:address 0x0000000000000000000000000000000000000405
pragma solidity ^0.8.0;

interface Farming {
    function staking(uint256 pool_id,uint256 amount) external;
    function claim(uint256 pool_id) external;
}
