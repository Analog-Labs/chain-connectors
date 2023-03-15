This get starter to run example available in rosetta-client.

Ethereum:

__Running ethereum_voting_contract example__
1. You must have a geth node running
Mac users can do:
```shell
brew tap ethereum/ethereum
brew install ethereum
```

For Linux you can use:
```shell
sudo apt-get update
sudo apt-get install ethereum
```

2. Run the geth node with the following command:
```shell
geth --dev --allow-insecure-unlock --http --http.api eth,debug,admin,txpool,web3
```

3. copy the .ipc path from the logs and run geth console.
```shell
geth attach geth_ipc_path_here.ipc
```

Now we have an interface where we can interact with the geth node.

Contract compiled version is already available in `example/compiled_voting_contract.json`. Run the following commands in order to deploy voting contract.

```js
var voting_machine_contract = {"contracts":{"voting_machine.sol:VotingMachine":{"abi":[{"inputs":[],"stateMutability":"nonpayable","type":"constructor"},{"inputs":[],"name":"get_votes_stats","outputs":[{"internalType":"uint256","name":"","type":"uint256"},{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"vote_no","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[],"name":"vote_yes","outputs":[],"stateMutability":"nonpayable","type":"function"}],"bin":"608060405234801561001057600080fd5b5060008081905550600060018190555061019b8061002f6000396000f3fe608060405234801561001057600080fd5b50600436106100415760003560e01c806322721754146100465780635b7e0f27146100505780636fe95c461461005a575b600080fd5b61004e610079565b005b610058610094565b005b6100626100af565b6040516100709291906100d9565b60405180910390f35b600180600082825461008b9190610131565b92505081905550565b60016000808282546100a69190610131565b92505081905550565b600080600054600154915091509091565b6000819050919050565b6100d3816100c0565b82525050565b60006040820190506100ee60008301856100ca565b6100fb60208301846100ca565b9392505050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b600061013c826100c0565b9150610147836100c0565b925082820190508082111561015f5761015e610102565b5b9291505056fea2646970667358221220d6dcd80743cd85a7570e21755b69b903685d6b2d7ac85d68f2499adbd442480e64736f6c63430008120033"}},"version":"0.8.18+commit.87f61d96.Darwin.appleclang"}
```

```js 
var votingContract = eth.contract(voting_machine_contract.contracts["voting_machine.sol:VotingMachine"].abi)
```
```js
var deployTransactionObject = { from: eth.coinbase, data: "0x" + voting_machine_contract.contracts["voting_machine.sol:VotingMachine"].bin, gas: 1000000 }
```
```js
personal.unlockAccount(eth.coinbase)
```
when asked for password, just hit enter.

```js
var votingContractInstance = votingContract.new(deployTransactionObject)
```
```js
var votingContractAddress = eth.getTransactionReceipt(votingContractInstance.transactionHash).contractAddress
```
```js
var votingContractInstance = votingContract.at(votingContractAddress)
```

Now we have votingContractInstance which we can use to interact with the contract or get details of contract.

write in console `votingContractInstance`  and hit enter. you should see a similar output as below:

```json
{
  abi: [{
      inputs: [],
      stateMutability: "nonpayable",
      type: "constructor"
  }, {
      inputs: [],
      name: "get_votes_stats",
      outputs: [{...}, {...}],
      stateMutability: "view",
      type: "function"
  }, {
      inputs: [],
      name: "vote_no",
      outputs: [],
      stateMutability: "nonpayable",
      type: "function"
  }, {
      inputs: [],
      name: "vote_yes",
      outputs: [],
      stateMutability: "nonpayable",
      type: "function"
  }],
  address: "0xc3c2640cfda6cafabb33da7ac1609a0fb4c53afe",
  transactionHash: null,
  allEvents: function bound(),
  get_votes_stats: function bound(),
  vote_no: function bound(),
  vote_yes: function bound()
}
```

address is basically the address on which our contract is deployed and where we can interact with it. <br>
We have total of 3 functions
`vote_yes`, `vote_no` and `get_votes_stats` <br>
`vote_yes` and `vote_no` are used to vote for yes or no respectively. <br>
`get_votes_stats` is used to get the current stats of votes. <br>

Contract code is available in `example/voting_contract.sol` <br>

Since `vote_yes` and `vote_no` can change state of contract we have to call them using rosetta-wallet which signs and sends transaction.

`get_votes_stats` can be called using rosetta-client `call` endpoint without need of a wallet.

We can test functions of our contract using `example/ethereum_voting_contract.rs` <br>
Before that make sure your rosetta-server-ethereum is running. or you can run it using 
```shell
cargo run --bin rosetta-server-ethereum -- --network=dev --addr=127.0.0.1:8081 --node-addr=127.0.0.1:8545 --path=/tmp/rosetta-ethereum
```

Here we are specifying that we want to connect to developer network of ethereum. <br>
`--addr` is the address on which rosetta-server-ethereum will run. <br>
`--node-addr` is the address of geth node. <br>
`--path` is the path where rosetta-server-ethereum will store its data. <br>

Now modify `example/ethereum_voting_contract.rs and change the contract address to the address of your contract in main function.

and run this command in cli from `rosetta-wallet` folder
```shell
cargo run --example ethereum_voting_contract
```

each time you run example you will be able to see an increment in yes votes and total stats of no and yes votes in contract call response.


__Running ethereum.rs example__

In this example we run all available methods of rosetta-wallet and rosetta-client. <br>
To run this example you have to do quite number of changes.


1. you have to deploy `example/compiled_changeowner.sol` contract on your ethereum network. <br>
2. you have to change the contract address in `example/ethereum.rs` file in main function. <br>
3. You have to transfer ownership to your wallet address before running from your geth console in order for this account to be able to call `change_owner` function since it requires only owner to call this contract. you can do this after deploying contract in getch attach console.
```shell
ownerContractInstance.changeOwner("your wallet address here", { from: eth.coinbase })
```
and make sure you have your wallet address unlocked in geth console otherwise it will return error. you can do this via
```shell
personal.importRawKey("your wallet raw key", "")
```
and ofcourse transfer some ether to your wallet address as well.<br>
```shell
eth.sendTransaction({from: eth.coinbase, to: "0xe79b3bb766f2e8d713b872f139f8128dd5cce04f", value: "74000000000000000"})
```
4. You have to make changes addresses in `example/ethereum.rs` file where eth.coinbase is commented. <br>
5. you have to provide valid block and block tx hashes to `block` and `block_transaction` function in order for them to return output


