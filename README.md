# Rosetta SDK Documentation
The Rosetta SDK is a Rust-based package that implements Coinbase’s [Rosetta API](https://www.rosetta-api.org/docs/welcome.html) specifications. The goal of Rosetta API is to make blockchain interaction simpler, faster, and more reliable than using native integration efforts.

This repo provides a framework for Analog’s connectors—special nodes hosted by time node operators—to simplify their interactions with Analog-connected chains in a manner compliant with the Analog Network’s protocol. 


## Repository structure
This repo contains the following modules:
- `rosetta-server`. This is a generic implementation of the Rosetta Server. The Rosetta Server is a standalone server that a connector on any Analog-supported chain can connect to and listen to the port specified in the settings. 
- `rosetta-client`. This is a standard client that interacts with the Rosetta Server. 
- `rosetta-types`. It contains the request and response structs used by the client and server. It is initially autogenerated using the openapi-generator. 
- `rosetta-crypto`. It has cryptographic primitives used by the rosetta-client.
- `rosetta-wallet`. This is a command line interface (CLI) built with the rosetta-client.
- `rosetta-cli`. This is a CLI built with the rosetta-client.
- `rosetta-docker`. This is a generic Rosetta Server testing infrastructure.
- `chains`. These are chain-specific client/server components.

## Getting started
<!--This section needs to be refined -->

To get started with the Rosetta SDK, you must ensure you have [installed the latest version of Docker](https://www.docker.com/get-started/).   
Run the following command to download chain-connectors:  
```
$ git clone https://github.com/Analog-Labs/chain-connectors.git
```

You can also download the latest pre-built Docker image release from GitHub by running the following command: 
```
curl -sSfL https://raw.githubusercontent.com/Analog-Labs/chain-connectors/master/install.sh | sh -s
``` 
After cloning this repo, simply run the following command: 
```
make build-local
``` 
### Connector deployment

<!-- This section needs to describe how operators will deploy their connectors.-->
<!--I am assuming here is where we initiate the rosetta-server and rosetta-client.--> 

### Install CLI tools 
Install the CLI tools by running the commands below: 
```
cargo install --path rosetta-cli
cargo install --path rosetta-wallet
``` 
To run any command with **rosetta-cli**, simply execute the **rosetta-cli** tool from the command line as follows:
```
rosetta-cli [command]
```  
 
Similarly, to run a command with the **rosetta-wallet**, simply execute the **rosetta-wallet** tool from the command line as follows: 
```
rosetta-wallet [command]
``` 
## Reference wallet implementations
To help you get started with wallets on specific chains, we have developed complete Rosetta API reference implementations for Bitcoin (deprioritized for now), Ethereum, and Substrate-based chains. 

### Ethereum example 
We have tested this implementation on an [AWS c5.2xlarge instance](https://aws.amazon.com/ec2/instance-types/c5). This instance type provides 8 vCPUs and 16 GB RAM. To use this repository, you need to fork it and start playing with the code. For example, running these commands will help you learn more about Rosetta API implementation for the Ethereum-based wallets:
```
rosetta-wallet --chain eth --keyfile /tmp/alice faucet 100000000000000
rosetta-wallet --chain eth --keyfile /tmp/alice balance
rosetta-wallet --chain eth --keyfile /tmp/bob account
rosetta-wallet --chain eth --keyfile /tmp/alice transfer bob_acc_key 10000000000000
rosetta-wallet --chain eth --keyfile /tmp/bob balance
```

### Substrate example
We have tested this implementation on an [AWS c5.2xlarge instance](https://aws.amazon.com/ec2/instance-types/c5). This instance type provides 8 vCPUs and 16 GB RAM. To use this repository, you need to fork it and start playing with the code. For example, running these commands will help you learn more about Rosetta API implementation for Substrate-based wallets:
```
rosetta-wallet --chain dot --keyfile /tmp/alice faucet 3000000000000000
rosetta-wallet --chain dot --keyfile /tmp/bob account
rosetta-wallet --chain dot --keyfile /tmp/alice transfer bob_acc_key 1500000000000000
rosetta-wallet --chain dot --keyfile /tmp/bob balance
```
### Bitcoin example
To use this repository, you need to fork it and start playing with the code. For example, running these commands will help you learn more about Rosetta API implementation for Bitcoin wallets:
```
rosetta-wallet --chain btc --keyfile /tmp/alice faucet 1000
rosetta-wallet --chain btc --keyfile /tmp/bob account
rosetta-wallet --chain btc --keyfile /tmp/alice transfer bob_acc_key 1000
rosetta-wallet --chain btc --keyfile /tmp/alice faucet 1
rosetta-wallet --chain btc --keyfile /tmp/bob balance
``` 
## Reference CLI implementation 
To help you get started with rosetta-cli, we have developed a standard indexer endpoint that you can leverage to integrate external blockchains automatically. The indexer endpoint complements the existing Data and Construction API endpoints in Rosetta API specifications, allowing developers to fully support asset integration. 

You will need an indexer URL that gets passed with the “—indexer-URL” flag to run an indexer. For example, in a local environment, you can run these commands to use the indexer: 
```
rosetta-cli --chain=btc search --indexer-url=http://localhost:8083 --type=Transfer --success=true

rosetta-cli --chain=eth search --indexer-url=http://localhost:8084 --type=Transfer --success=true

rosetta-cli --chain=dot search --indexer-url=http://localhost:8085 --type=Transfer --success=true
```

### Block Explorer
To launch the Block Explorer in your browser, simply open your browser and point it to:
http://rosetta.analog.one:3000 

### Run local testnet
Running a local testnet with docker compose up initiates a number of containers, including:
- bitcoin: http://127.0.0.1:8080
- ethereum: http://127.0.0.1:8081
- polkadot: http://127.0.0.1:8082
- block explorer: [http://127.0.0.1:3000](http://127.0.0.1:3000)

You can override the default URL in rosetta-cli and rosetta-wallet with the “—URL” flag.

## Update AWS deployment
Create a new tag, push to master and use it to create a new github release.
## Contributing
You can contribute to this repo in a number of ways, including:
- [Asking questions](https://github.com/Analog-Labs/chain-connectors/issues/new?assignees=&labels=question&template=ask-a-question.md&title=)
- [Giving feedback](https://github.com/Analog-Labs/chain-connectors/issues/new?assignees=&labels=enhancement&template=suggest-a-feature.md&title=)
- [Reporting bugs](https://github.com/Analog-Labs/chain-connectors/issues/new?assignees=&labels=bug&template=report-a-bug.md&title=)
Read our [contribution guidelines](https://github.com/Analog-Labs/.github-private/wiki/Contribution-Guidelines) for more information on how to contribute to this repo. 

