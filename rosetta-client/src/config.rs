use crate::crypto::Algorithm;
use crate::types::{Currency, NetworkIdentifier};

pub struct BlockchainConfig {
    pub url: String,
    pub network: NetworkIdentifier,
    pub algorithm: Algorithm,
    pub coin: u32,
    pub currency: Currency,
    pub bip44: bool,
    pub utxo: bool,
}

impl BlockchainConfig {
    pub fn bitcoin_regtest() -> Self {
        Self {
            url: "http://127.0.0.1:8080".into(),
            network: NetworkIdentifier {
                blockchain: "Bitcoin".into(),
                network: "Regtest".into(),
                sub_network_identifier: None,
            },
            algorithm: Algorithm::EcdsaSecp256k1,
            coin: 1,
            currency: Currency {
                symbol: "tBTC".into(),
                decimals: 8,
                metadata: None,
            },
            bip44: true,
            utxo: true,
        }
    }

    pub fn ethereum_dev() -> Self {
        Self {
            url: "http://127.0.0.1:8081".into(),
            network: NetworkIdentifier {
                blockchain: "Ethereum".into(),
                network: "Dev".into(),
                sub_network_identifier: None,
            },
            algorithm: Algorithm::EcdsaRecoverableSecp256k1,
            coin: 1,
            currency: Currency {
                symbol: "ETH".into(),
                decimals: 18,
                metadata: None,
            },
            bip44: true,
            utxo: false,
        }
    }

    pub fn polkadot_dev() -> Self {
        Self {
            url: "http://127.0.0.1:8082".into(),
            network: NetworkIdentifier {
                blockchain: "Polkadot".into(),
                network: "Dev".into(),
                sub_network_identifier: None,
            },
            algorithm: Algorithm::Sr25519,
            coin: 1,
            currency: Currency {
                symbol: "DOT".into(),
                decimals: 10,
                metadata: None,
            },
            bip44: false,
            utxo: false,
        }
    }
}
