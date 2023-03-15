// SPDX-License-Identifier: MIT
pragma solidity >=0.4.22;

contract VotingMachine{
    uint yes_votes;
    uint no_votes;

    constructor() {
        yes_votes = 0;
        no_votes = 0;
    }

    function vote_yes() public {
        yes_votes += 1;
    }

    function vote_no() public {
        no_votes += 1;
    }

    function get_votes_stats() external view returns (uint, uint) {
        return (yes_votes, no_votes);
    }
}
