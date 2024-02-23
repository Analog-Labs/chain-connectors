mod config;

use anyhow::{Context, Result};
use docker_api::{
    conn::TtyChunk,
    opts::{
        ContainerCreateOpts, ContainerListOpts, ContainerStopOpts, HostPort, LogsOpts, PublishPort,
    },
    ApiVersion, Container, Docker,
};
use futures::stream::StreamExt;
use rosetta_client::Wallet;
use rosetta_core::{BlockchainClient, BlockchainConfig};
use std::{future::Future, sync::Arc, time::Duration};
use tokio_retry::{strategy::ExponentialBackoff, RetryIf};

pub struct Env<T> {
    client: Arc<T>,
    node: Container,
}

impl<T: BlockchainClient> Env<T> {
    #[allow(clippy::missing_errors_doc)]
    pub async fn new<Fut, F>(
        prefix: &str,
        mut config: BlockchainConfig,
        start_connector: F,
    ) -> Result<Self>
    where
        Fut: Future<Output = Result<T>> + Send,
        F: FnMut(BlockchainConfig) -> Fut + Send,
    {
       println!("inside env 1");
        let builder = EnvBuilder::new(prefix)?;
        let node_port = random_port();
        config.node_uri.port = node_port;
        log::info!("node: {}", node_port);
        builder.stop_container(&builder.node_name(&config)).await?;
        println!("inside env 2 {:?}",node_port);
        let node = builder.run_node(&config).await?;
    println!("inside env 3");
        let client = match builder.run_connector::<T, Fut, F>(start_connector, config).await {
            Ok(connector) => connector,
            Err(e) => {
                let opts = ContainerStopOpts::builder().build();
                let _ = node.stop(&opts).await;
                return Err(e);
            },
        };

        Ok(Self { client: Arc::new(client), node })
    }

    #[must_use]
    pub fn node(&self) -> Arc<T> {
        Arc::clone(&self.client)
    }

    /// Creates a new ephemeral wallet
    ///
    /// # Errors
    /// Returns `Err` if the node uri is invalid or keyfile doesn't exists
    pub async fn ephemeral_wallet(&self) -> Result<Wallet> {
        let config = self.client.config().clone();
        let node_uri = config.node_uri.to_string();
        Wallet::from_config(config, &node_uri, None).await
    }

    /// Stop all containers
    ///
    /// # Errors
    /// Will return `Err` if it fails to stop the container for some reason
    pub async fn shutdown(self) -> Result<()> {
        let opts = ContainerStopOpts::builder().build();
        self.node.stop(&opts).await?;
        Ok(())
    }
}

struct EnvBuilder<'a> {
    prefix: &'a str,
    docker: Docker,
}

impl<'a> EnvBuilder<'a> {
    pub fn new(prefix: &'a str) -> Result<Self> {
        let version = ApiVersion::new(1, Some(41), None);
        let endpoint = config::docker_endpoint();
        let docker = Docker::new_versioned(endpoint, version)?;
        Ok(Self { prefix, docker })
    }

    fn node_name(&self, config: &BlockchainConfig) -> String {
        format!("{}-node-{}-{}", self.prefix, config.blockchain, config.network)
    }

    async fn stop_container(&self, name: &str) -> Result<()> {
        let opts = ContainerListOpts::builder().all(true).build();
        for container in self.docker.containers().list(&opts).await? {
            if container
                .names
                .as_ref()
                .context("no containers found")?
                .iter()
                .any(|n| n.as_str().ends_with(name))
            {
                let container = Container::new(
                    self.docker.clone(),
                    container.id.context("container doesn't have id")?,
                );
                log::info!("stopping {}", name);
                container.stop(&ContainerStopOpts::builder().build()).await?;
                container.delete().await.ok();
                break;
            }
        }
        Ok(())
    }

    async fn run_container(&self, name: String, opts: &ContainerCreateOpts) -> Result<Container> {
        log::info!("creating {}", name);
        let id = self.docker.containers().create(opts).await?.id().clone();
        let container = Container::new(self.docker.clone(), id.clone());
        container.start().await?;

        log::info!("starting {}", name);
        let container = Container::new(self.docker.clone(), id.clone());
        tokio::task::spawn(async move {
            let opts = LogsOpts::builder().all().follow(true).stdout(true).stderr(true).build();
            let mut logs = container.logs(&opts);
            while let Some(chunk) = logs.next().await {
                match chunk {
                    Ok(TtyChunk::StdOut(stdout)) => {
                        let stdout = std::str::from_utf8(&stdout).unwrap_or_default();
                        log::info!("{}: stdout: {}", name, stdout);
                    },
                    Ok(TtyChunk::StdErr(stderr)) => {
                        let stderr = std::str::from_utf8(&stderr).unwrap_or_default();
                        log::info!("{}: stderr: {}", name, stderr);
                    },
                    Err(err) => {
                        log::error!("{}", err);
                    },
                    Ok(TtyChunk::StdIn(_)) => unreachable!(),
                }
            }
            log::info!("{}: exited", name);
        });

        let container = Container::new(self.docker.clone(), id.clone());
        loop {
            match health(&container).await? {
                Some(Health::Unhealthy) => anyhow::bail!("healthcheck reports unhealthy"),
                Some(Health::Starting) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                },
                _ => break,
            }
        }

        Ok(container)
    }

    async fn run_node(&self, config: &BlockchainConfig) -> Result<Container> {
    println!("inside run_node");
        let name = self.node_name(config);
        let mut opts = ContainerCreateOpts::builder()
            .name(&name)
            .image(config.node_image)
            .command((config.node_command)(config.network, config.node_uri.port))
            .auto_remove(true)
            .attach_stdout(true)
            .attach_stderr(true)
            .publish(PublishPort::tcp(u32::from(config.node_uri.port)))
            .expose(
                PublishPort::tcp(u32::from(config.node_uri.port)),
                HostPort::new(u32::from(config.node_uri.port)),
            );
        for port in config.node_additional_ports {
            let port = u32::from(*port);
            opts = opts.expose(PublishPort::tcp(port), port);
        }
        let container = self.run_container(name, &opts.build()).await?;
    println!("inside run_node 2");
        // TODO: replace this by a proper healthcheck
        let maybe_error = if matches!(config.node_uri.scheme, "http" | "https" | "ws" | "wss") {
            wait_for_http(
                config
                    .node_uri
                    .with_scheme("http") // any ws endpoint is also a http endpoint
                    .with_host("127.0.0.1")
                    .to_string(),
                &container,
            )
            .await
            .err()
        } else {
            println!("here we are in run node error");
            // Wait 15 seconds to guarantee the node didn't crash
            tokio::time::sleep(Duration::from_secs(15)).await;
            health(&container).await.err()
        };

        if let Some(err) = maybe_error {
            log::error!("node failed to start: {}", err);
            let _ = container.stop(&ContainerStopOpts::default()).await;
            return Err(err);
        }
        Ok(container)
    }

    async fn run_connector<T, Fut, F>(
        &self,
        mut start_connector: F,
        config: BlockchainConfig,
    ) -> Result<T>
    where
        T: BlockchainClient,
        Fut: Future<Output = Result<T>> + Send,
        F: FnMut(BlockchainConfig) -> Fut + Send,
    {
        const MAX_RETRIES: usize = 10;

        let client = {
            let retry_strategy = tokio_retry::strategy::FibonacciBackoff::from_millis(1000)
                .max_delay(Duration::from_secs(5))
                .take(MAX_RETRIES);
            let mut result = Err(anyhow::anyhow!("failed to start connector"));
            for delay in retry_strategy {
                match start_connector(config.clone()).await {
                    Ok(client) => {
                        if let Err(error) = client.finalized_block().await {
                            result = Err(error);
                            continue;
                        }
                        result = Ok(client);
                        break;
                    },
                    Err(error) => {
                        result = Err(error);
                        tokio::time::sleep(delay).await;
                    },
                }
            }
            result?
        };

        Ok(client)
    }
}

fn random_port() -> u16 {
    let mut bytes = [0; 2];
    #[allow(clippy::unwrap_used)]
    getrandom::getrandom(&mut bytes).unwrap();
    u16::from_le_bytes(bytes)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Health {
    None,
    Starting,
    Healthy,
    Unhealthy,
}

async fn health(container: &Container) -> Result<Option<Health>> {
    let inspect = container.inspect().await?;
    let status = inspect.state.and_then(|state| state.health).and_then(|health| health.status);
    let Some(status) = status else { return Ok(None) };
    Ok(Some(match status.as_str() {
        "none" => Health::None,
        "starting" => Health::Starting,
        "healthy" => Health::Healthy,
        "unhealthy" => Health::Unhealthy,
        status => anyhow::bail!("unknown status {}", status),
    }))
}

#[derive(Debug)]
enum RetryError {
    Retry(anyhow::Error),
    ContainerExited(anyhow::Error),
}

async fn wait_for_http<S: AsRef<str> + Send>(url: S, container: &Container) -> Result<()> {
    let url = url.as_ref();
println!("wait_for_http");
    let retry_strategy = ExponentialBackoff::from_millis(2)
        .factor(100)
        .max_delay(Duration::from_secs(2))
        .take(20); // limit to 20 retries
println!("wait_for_http2 {:?}",retry_strategy);
 
    RetryIf::spawn(
        retry_strategy,
        || async move {
            println!("url : {:?}",url);
            match surf::get(url).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    // Check if the container exited
                    let health_status = health(container).await;
                    if matches!(health_status, Err(_) | Ok(Some(Health::Unhealthy))) {
                        return Err(RetryError::ContainerExited(err.into_inner()));
                    }
                    Err(RetryError::Retry(err.into_inner()))
                },
            }
        },
        // Retry Condition
        |error: &RetryError| matches!(error, RetryError::Retry(_)),
    )
    .await
    .map_err(|err| match err {
        RetryError::Retry(error) | RetryError::ContainerExited(error) => error,
    })
}

#[cfg(feature = "tests")]
pub mod tests {
    use super::Env;
    use anyhow::Result;
    use nanoid::nanoid;
    use rosetta_core::{types::PartialBlockIdentifier, BlockchainClient, BlockchainConfig};
    use std::future::Future;

    fn env_id() -> String {
        nanoid!(
            10,
            &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',]
        )
    }

    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub async fn network_status<T, Fut, F>(
        start_connector: F,
        config: BlockchainConfig,
    ) -> Result<()>
    where
        T: BlockchainClient,
        Fut: Future<Output = Result<T>> + Send,
        F: FnMut(BlockchainConfig) -> Fut + Send,
    {
    println!("inside network status 1");
        let env_id = env_id();
        let env =
            Env::new(&format!("{env_id}-network-status"), config.clone(), start_connector).await?;

    println!("inside network status 2");
        let client = env.node();

        // Check if the genesis is consistent
        let expected_genesis = client.genesis_block().clone();
        let actual_genesis = client
            .block(&PartialBlockIdentifier { index: Some(0), hash: None })
            .await?
            .block_identifier;
        assert_eq!(expected_genesis, actual_genesis);

    println!("inside network status 3");
        // Check if the current block is consistent
        let expected_current = client.current_block().await?;
        let actual_current = client
            .block(&PartialBlockIdentifier {
                index: None,
                hash: Some(expected_current.hash.clone()),
            })
            .await?
            .block_identifier;
        assert_eq!(expected_current, actual_current);
    println!("inside network status 4");
    
        // Check if the finalized block is consistent
        let expected_finalized = client.finalized_block().await?;
        let actual_finalized = client
            .block(&PartialBlockIdentifier {
                index: None,
                hash: Some(expected_finalized.hash.clone()),
            })
            .await?
            .block_identifier;
        assert_eq!(expected_finalized, actual_finalized);
    println!("inside network status 5");
        env.shutdown().await?;
        Ok(())
    }

    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub async fn account<T, Fut, F>(start_connector: F, config: BlockchainConfig) -> Result<()>
    where
        T: BlockchainClient,
        Fut: Future<Output = Result<T>> + Send,
        F: FnMut(BlockchainConfig) -> Fut + Send,
    {
        let env_id = env_id();
        let env = Env::new(&format!("{env_id}-account"), config.clone(), start_connector).await?;

        let value = 100 * u128::pow(10, config.currency_decimals);
        let wallet = env.ephemeral_wallet().await?;
        wallet.faucet(value).await?;
        let amount = wallet.balance().await?;
        assert_eq!(amount.value, value.to_string());
        assert_eq!(amount.currency, config.currency());
        assert!(amount.metadata.is_none());

        env.shutdown().await?;
        Ok(())
    }

    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub async fn construction<T, Fut, F>(start_connector: F, config: BlockchainConfig) -> Result<()>
    where
        T: BlockchainClient,
        Fut: Future<Output = Result<T>> + Send,
        F: FnMut(BlockchainConfig) -> Fut + Send,
    {
        let env_id = env_id();
        let env =
            Env::new(&format!("{env_id}-construction"), config.clone(), start_connector).await?;

        let faucet = 100 * u128::pow(10, config.currency_decimals);
        let value = u128::pow(10, config.currency_decimals);
        let alice = env.ephemeral_wallet().await?;
        let bob = env.ephemeral_wallet().await?;
        assert_ne!(alice.public_key(), bob.public_key());

        // Alice and bob have no balance
        let balance = alice.balance().await?;
        assert_eq!(balance.value, "0");
        let balance = bob.balance().await?;
        assert_eq!(balance.value, "0");

        // Transfer faucets to alice
        alice.faucet(faucet).await?;
        let balance = alice.balance().await?;
        assert_eq!(balance.value, faucet.to_string());

        // Alice transfers to bob
        alice.transfer(bob.account(), value).await?;
        let amount = bob.balance().await?;
        assert_eq!(amount.value, value.to_string());

        env.shutdown().await?;
        Ok(())
    }
}
