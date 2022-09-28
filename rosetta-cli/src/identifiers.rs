use clap::Parser;
use rosetta_client::types::{
    AccountIdentifier, BlockIdentifier, NetworkIdentifier, PartialBlockIdentifier,
    SubAccountIdentifier, SubNetworkIdentifier, TransactionIdentifier,
};

#[derive(Parser)]
pub struct NetworkIdentifierOpts {
    #[clap(long)]
    blockchain: Option<String>,
    #[clap(long)]
    network: Option<String>,
    #[clap(long)]
    subnetwork: Option<String>,
}

impl NetworkIdentifierOpts {
    pub fn network_identifier(&self) -> Option<NetworkIdentifier> {
        Some(NetworkIdentifier {
            blockchain: self.blockchain.as_ref()?.into(),
            network: self.network.as_ref()?.into(),
            sub_network_identifier: self.subnetwork.as_ref().map(|subnetwork| {
                SubNetworkIdentifier {
                    network: subnetwork.clone(),
                    metadata: None,
                }
            }),
        })
    }
}

#[derive(Parser)]
pub struct AccountIdentifierOpts {
    account: String,
    #[clap(long)]
    subaccount: Option<String>,
}

impl AccountIdentifierOpts {
    pub fn account_identifier(&self) -> AccountIdentifier {
        AccountIdentifier {
            address: self.account.clone(),
            sub_account: self
                .subaccount
                .as_ref()
                .map(|subaccount| SubAccountIdentifier {
                    address: subaccount.clone(),
                    metadata: None,
                }),
            metadata: None,
        }
    }
}

#[derive(Parser)]
pub struct BlockIdentifierOpts {
    #[clap(long)]
    index: Option<u64>,
    #[clap(name = "block", long)]
    hash: Option<String>,
}

impl BlockIdentifierOpts {
    pub fn partial_block_identifier(&self) -> Option<PartialBlockIdentifier> {
        if self.index.is_none() && self.hash.is_none() {
            return None;
        }
        Some(PartialBlockIdentifier {
            index: self.index,
            hash: self.hash.clone(),
        })
    }

    pub fn block_identifier(&self) -> Option<BlockIdentifier> {
        if let (Some(index), Some(hash)) = (self.index, &self.hash) {
            Some(BlockIdentifier {
                index,
                hash: hash.clone(),
            })
        } else {
            None
        }
    }
}

#[derive(Parser)]
pub struct TransactionIdentifierOpts {
    #[clap(name = "transaction", long)]
    hash: Option<String>,
}

impl TransactionIdentifierOpts {
    pub fn transaction_identifier(&self) -> Option<TransactionIdentifier> {
        Some(TransactionIdentifier {
            hash: self.hash.as_ref()?.clone(),
        })
    }
}
