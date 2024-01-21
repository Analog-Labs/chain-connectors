use crate::types::ClientConfig;

pub struct PolkadotClient {
    config: BlockchainConfig,
    client: OnlineClient<PolkadotConfig>,
    rpc_methods: LegacyRpcMethods<PolkadotConfig>,
    genesis_block: BlockIdentifier,
}
