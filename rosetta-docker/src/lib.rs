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
        let builder = EnvBuilder::new(prefix)?;
        let node_port = random_port();
        config.node_uri.port = node_port;
        log::info!("node: {}", node_port);
        builder.stop_container(&builder.node_name(&config)).await?;
        let node = builder.run_node(&config).await?;

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
        Wallet::from_config(config, &node_uri, None, None).await
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
    let retry_strategy = ExponentialBackoff::from_millis(2)
        .factor(100)
        .max_delay(Duration::from_secs(2))
        .take(20); // limit to 20 retries

    RetryIf::spawn(
        retry_strategy,
        || async move {
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

/// Helper function to run a test and shutdown docker containers regardless if the test panics or
/// not
#[allow(clippy::future_not_send, clippy::redundant_pub_crate)]
pub async fn run_test<T, Fut, F>(env: Env<T>, cb: F)
where
    T: Sync + Send + 'static + rosetta_core::BlockchainClient,
    Fut: Future<Output = ()> + Send + 'static,
    F: FnOnce(&'static mut Env<T>) -> Fut + Sync + Send,
{
    // Convert the context into a raw pointer
    let ptr = Box::into_raw(Box::new(env));

    // Execute the test and catch any panics
    let result = unsafe {
        let handler = tokio::spawn(cb(&mut *ptr));
        tokio::select! {
            result = handler => result,
            _ = tokio::signal::ctrl_c() => {
                log::info!("ctrl-c received, shutting down docker containers...");
                Ok(())
            },
        }
    };

    // Convert the raw pointer back into a context
    let env = unsafe { Box::from_raw(ptr) };

    let _ = Env::shutdown(*env).await;

    // Now is safe to panic
    if let Err(err) = result {
        // Resume the panic on the main task
        std::panic::resume_unwind(err.into_panic());
    }
}

#[cfg(feature = "tests")]
pub mod tests {
    use super::Env;
    use anyhow::{Ok, Result};
    use nanoid::nanoid;
    use rosetta_core::{
        types::{BlockIdentifier, PartialBlockIdentifier},
        BlockchainClient, BlockchainConfig,
    };
    use std::future::Future;

    fn env_id() -> String {
        nanoid!(
            10,
            &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',]
        )
    }

    #[allow(
        clippy::missing_panics_doc,
        clippy::unwrap_used,
        clippy::missing_errors_doc,
        clippy::future_not_send
    )]
    pub async fn network_status<T, Fut, F>(
        start_connector: F,
        config: BlockchainConfig,
    ) -> Result<()>
    where
        T: BlockchainClient<AtBlock = PartialBlockIdentifier, BlockIdentifier = BlockIdentifier>,
        Fut: Future<Output = Result<T>> + Send,
        F: FnMut(BlockchainConfig) -> Fut + Send,
    {
        let env_id = env_id();
        let env =
            Env::new(&format!("{env_id}-network-status"), config.clone(), start_connector).await?;

        crate::run_test(env, |env| async move {
            let client = env.node();

            // Check if the genesis is consistent
            let genesis_block = client.genesis_block();
            assert_eq!(genesis_block.index, 0);

            // Check if the current block is consistent
            let current_block = client.current_block().await.unwrap();
            if current_block.index > 0 {
                assert_ne!(current_block.hash, genesis_block.hash);
            } else {
                assert_eq!(current_block.hash, genesis_block.hash);
            }

            // Check if the finalized block is consistent
            let finalized_block = client.finalized_block().await.unwrap();
            assert!(finalized_block.index >= genesis_block.index);
        })
        .await;
        Ok(())
    }

    #[allow(
        clippy::missing_panics_doc,
        clippy::unwrap_used,
        clippy::missing_errors_doc,
        clippy::future_not_send
    )]
    pub async fn account<T, Fut, F>(start_connector: F, config: BlockchainConfig) -> Result<()>
    where
        T: BlockchainClient<AtBlock = PartialBlockIdentifier, BlockIdentifier = BlockIdentifier>,
        Fut: Future<Output = Result<T>> + Send,
        F: FnMut(BlockchainConfig) -> Fut + Send,
    {
        let env_id = env_id();
        let env = Env::new(&format!("{env_id}-account"), config.clone(), start_connector).await?;
        crate::run_test(env, |env| async move {
            let value = 100 * u128::pow(10, config.currency_decimals);
            let wallet = env.ephemeral_wallet().await.unwrap();
            wallet.faucet(value, None).await.unwrap();
            let balance = wallet.balance().await.unwrap();
            assert_eq!(balance, value);
        })
        .await;
        Ok(())
    }

    #[allow(
        clippy::missing_panics_doc,
        clippy::unwrap_used,
        clippy::missing_errors_doc,
        clippy::future_not_send
    )]
    pub async fn construction<T, Fut, F>(start_connector: F, config: BlockchainConfig) -> Result<()>
    where
        T: BlockchainClient<AtBlock = PartialBlockIdentifier, BlockIdentifier = BlockIdentifier>,
        Fut: Future<Output = Result<T>> + Send,
        F: FnMut(BlockchainConfig) -> Fut + Send,
    {
        let env_id = env_id();
        let env =
            Env::new(&format!("{env_id}-construction"), config.clone(), start_connector).await?;

        crate::run_test(env, |env| async move {
            let faucet = 100 * u128::pow(10, config.currency_decimals);
            let value = u128::pow(10, config.currency_decimals);
            let alice = env.ephemeral_wallet().await.unwrap();
            let bob = env.ephemeral_wallet().await.unwrap();
            assert_ne!(alice.public_key(), bob.public_key());

            // Alice and bob have no balance
            let balance = alice.balance().await.unwrap();
            assert_eq!(balance, 0);
            let balance = bob.balance().await.unwrap();
            assert_eq!(balance, 0);

            // Transfer faucets to alice
            alice.faucet(faucet, None).await.unwrap();
            let balance = alice.balance().await.unwrap();
            assert_eq!(balance, faucet);

            // Alice transfers to bob
            alice.transfer(bob.account(), value, None, None).await.unwrap();
            let amount = bob.balance().await.unwrap();
            assert_eq!(amount, value);
        })
        .await;
        Ok(())
    }
}
