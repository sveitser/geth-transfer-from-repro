//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "hardhat/console.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract Deposit {

    uint256 dummy;

    function deposit(address asset, uint256 amount) public {
        ERC20 token = ERC20(asset);
        token.transferFrom(msg.sender, address(this), amount);
        // dummy = 1; // the tx only goes through on geth if this is uncommented
    }
}
