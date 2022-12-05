use crate::crypto::address::{AddressFormat, Ss58AddressFormatRegistry};
use crate::crypto::Algorithm;
use crate::types::{Currency, NetworkIdentifier};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockchainConfig {
    pub network: NetworkIdentifier,
    pub algorithm: Algorithm,
    pub address_format: AddressFormat,
    pub coin: u32,
    pub currency: Currency,
    pub bip44: bool,
    pub utxo: bool,
    pub unit: &'static str,
}

impl BlockchainConfig {
    pub fn bitcoin_regtest() -> Self {
        Self {
            network: NetworkIdentifier {
                blockchain: "Bitcoin".into(),
                network: "Regtest".into(),
                sub_network_identifier: None,
            },
            algorithm: Algorithm::EcdsaSecp256k1,
            address_format: AddressFormat::Bech32("bcrt"),
            coin: 1,
            currency: Currency {
                symbol: "tBTC".into(),
                decimals: 8,
                metadata: None,
            },
            bip44: true,
            utxo: true,
            unit: "satoshi",
        }
    }

    pub fn ethereum_dev() -> Self {
        Self {
            network: NetworkIdentifier {
                blockchain: "Ethereum".into(),
                network: "Dev".into(),
                sub_network_identifier: None,
            },
            algorithm: Algorithm::EcdsaRecoverableSecp256k1,
            address_format: AddressFormat::Eip55,
            coin: 1,
            currency: Currency {
                symbol: "ETH".into(),
                decimals: 18,
                metadata: None,
            },
            bip44: true,
            utxo: false,
            unit: "wei",
        }
    }

    pub fn polkadot_dev() -> Self {
        Self {
            network: NetworkIdentifier {
                blockchain: "Polkadot".into(),
                network: "Dev".into(),
                sub_network_identifier: None,
            },
            algorithm: Algorithm::Sr25519,
            address_format: AddressFormat::Ss58(Ss58AddressFormatRegistry::PolkadotAccount.into()),
            coin: 1,
            currency: Currency {
                symbol: "DOT".into(),
                decimals: 10,
                metadata: None,
            },
            bip44: false,
            utxo: false,
            unit: "planck",
        }
    }
}
