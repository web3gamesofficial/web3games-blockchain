// SPDX-License-Identifier: MIT
// @custom:address 0xFFFFFFFF00000000000000000000000000000000
pragma solidity ^0.8.0;

interface TokenFungible {
    function create(bytes memory name,bytes memory symbol,uint8 decimals) external;
    function name() external view returns (string memory);
    function symbol() external view returns (string memory);
    function decimals() external view returns (uint256);
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    function transfer(address to, uint256 amount) external;
    function transferFrom(address from,address to, uint256 amount) external;
    function mint(address account, uint256 amount) external;
    function burn(uint256 amount) external;
}
