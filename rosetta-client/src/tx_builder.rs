use crate::{
    client::{GenericMetadata, GenericMetadataParams},
    crypto::{address::Address, SecretKey},
    BlockchainConfig,
};
use anyhow::Result;
use rosetta_core::TransactionBuilder;
use rosetta_server_astar::AstarMetadataParams;

pub enum GenericTransactionBuilder {
    Astar(rosetta_tx_ethereum::EthereumTransactionBuilder),
    Ethereum(rosetta_tx_ethereum::EthereumTransactionBuilder),
    Polkadot(rosetta_tx_polkadot::PolkadotTransactionBuilder),
}

impl GenericTransactionBuilder {
    pub fn new(config: &BlockchainConfig) -> Result<Self> {
        Ok(match config.blockchain {
            "astar" => Self::Astar(rosetta_tx_ethereum::EthereumTransactionBuilder),
            "ethereum" | "polygon" | "zkevm" | "arbitrum" | "binance" | "base" | "avalanche" => {
                Self::Ethereum(rosetta_tx_ethereum::EthereumTransactionBuilder)
            },
            "polkadot" | "westend" | "rococo" => {
                Self::Polkadot(rosetta_tx_polkadot::PolkadotTransactionBuilder)
            },
            _ => anyhow::bail!("unsupported blockchain: {}", config.blockchain),
        })
    }

    pub fn transfer(&self, address: &Address, amount: u128) -> Result<GenericMetadataParams> {
        Ok(match self {
            Self::Astar(tx) => AstarMetadataParams(tx.transfer(address, amount)?).into(),
            Self::Ethereum(tx) => tx.transfer(address, amount)?.into(),
            Self::Polkadot(tx) => tx.transfer(address, amount)?.into(),
        })
    }

    pub fn method_call(
        &self,
        contract: &[u8; 20],
        data: &[u8],
        amount: u128,
    ) -> Result<GenericMetadataParams> {
        Ok(match self {
            Self::Astar(tx) => AstarMetadataParams(tx.method_call(contract, data, amount)?).into(),
            Self::Ethereum(tx) => tx.method_call(contract, data, amount)?.into(),
            Self::Polkadot(tx) => tx.method_call(contract, data, amount)?.into(),
        })
    }

    pub fn deploy_contract(&self, contract_binary: Vec<u8>) -> Result<GenericMetadataParams> {
        Ok(match self {
            Self::Astar(tx) => AstarMetadataParams(tx.deploy_contract(contract_binary)?).into(),
            Self::Ethereum(tx) => tx.deploy_contract(contract_binary)?.into(),
            Self::Polkadot(tx) => tx.deploy_contract(contract_binary)?.into(),
        })
    }

    pub fn create_and_sign(
        &self,
        config: &BlockchainConfig,
        params: &GenericMetadataParams,
        metadata: &GenericMetadata,
        secret_key: &SecretKey,
    ) -> Result<Vec<u8>> {
        Ok(match (self, params, metadata) {
            (
                Self::Astar(tx),
                GenericMetadataParams::Astar(params),
                GenericMetadata::Astar(metadata),
            ) => tx.create_and_sign(config, &params.0, &metadata.0, secret_key),
            (
                Self::Ethereum(tx),
                GenericMetadataParams::Ethereum(params),
                GenericMetadata::Ethereum(metadata),
            ) => tx.create_and_sign(config, params, metadata, secret_key),
            (
                Self::Polkadot(tx),
                GenericMetadataParams::Polkadot(params),
                GenericMetadata::Polkadot(metadata),
            ) => tx.create_and_sign(config, params, metadata, secret_key),
            _ => anyhow::bail!("invalid params"),
        })
    }
}
