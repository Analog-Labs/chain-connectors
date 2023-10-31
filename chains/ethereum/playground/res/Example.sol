// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.5.0 <0.9.0;

contract Example {
    uint256 private _value;

    function value() public view returns (uint256) {
        return _value;
    }

    function setValue(uint256 newValue) public {
        _value = newValue;
    }
}
