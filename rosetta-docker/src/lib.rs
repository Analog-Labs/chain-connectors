use anyhow::Result;
use docker_api::conn::TtyChunk;
use docker_api::opts::{
    ContainerConnectionOpts, ContainerCreateOpts, ContainerListOpts, LogsOpts, NetworkCreateOpts,
    NetworkListOpts, PublishPort,
};
use docker_api::{Container, Docker, Network};
use futures::stream::StreamExt;
use rosetta_client::{Client, Signer, Wallet};
use rosetta_core::{BlockchainClient, BlockchainConfig};
use std::time::Duration;

pub struct Env {
    config: BlockchainConfig,
    network: Network,
    node: Container,
    connector: Container,
}

impl Env {
    pub async fn new(prefix: &'static str, mut config: BlockchainConfig) -> Result<Self> {
        env_logger::try_init().ok();
        let builder = EnvBuilder::new(prefix)?;
        config.node_port = builder.random_port();
        config.connector_port = builder.random_port();
        log::info!("node: {}", config.node_port);
        log::info!("connector: {}", config.connector_port);
        builder
            .stop_container(&builder.connector_name(&config))
            .await?;
        builder.stop_container(&builder.node_name(&config)).await?;
        builder.delete_network(&builder.network_name()).await?;
        let network = builder.create_network().await?;
        let node = builder.run_node(&config, &network).await?;
        let connector = builder.run_connector(&config, &network).await?;
        Ok(Self {
            config,
            network,
            node,
            connector,
        })
    }

    pub async fn node<T: BlockchainClient>(&self) -> Result<T> {
        let addr = format!("127.0.0.1:{}", self.config.node_port);
        T::new(self.config.network, &addr).await
    }

    pub fn connector(&self) -> Result<Client> {
        let url = format!("http://127.0.0.1:{}", self.config.connector_port);
        Client::new(&url)
    }

    pub fn ephemeral_wallet(&self) -> Result<Wallet> {
        let client = self.connector()?;
        let signer = Signer::generate()?;
        Wallet::new(self.config.clone(), &signer, client)
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.connector.stop(None).await?;
        self.node.stop(None).await?;
        self.network.delete().await?;
        Ok(())
    }
}

struct EnvBuilder {
    prefix: &'static str,
    docker: Docker,
}

impl EnvBuilder {
    pub fn new(prefix: &'static str) -> Result<Self> {
        #[cfg(unix)]
        let docker = Docker::unix("/var/run/docker.sock");
        #[cfg(not(unix))]
        let docker = Docker::new("tcp://127.0.0.1:8080")?;
        Ok(Self { prefix, docker })
    }

    fn random_port(&self) -> u16 {
        let mut bytes = [0; 2];
        getrandom::getrandom(&mut bytes).unwrap();
        u16::from_le_bytes(bytes)
    }

    fn network_name(&self) -> String {
        format!("{}-rosetta-docker", self.prefix)
    }

    fn node_name(&self, config: &BlockchainConfig) -> String {
        format!(
            "{}-node-{}-{}",
            self.prefix, config.blockchain, config.network
        )
    }

    fn connector_name(&self, config: &BlockchainConfig) -> String {
        format!(
            "{}-connector-{}-{}",
            self.prefix, config.blockchain, config.network
        )
    }

    async fn create_network(&self) -> Result<Network> {
        let opts = NetworkCreateOpts::builder(self.network_name()).build();
        let network = self.docker.networks().create(&opts).await?;
        let id = network.id().clone();
        Ok(Network::new(self.docker.clone(), id))
    }

    async fn delete_network(&self, name: &str) -> Result<()> {
        let opts = NetworkListOpts::builder().build();
        for network in self.docker.networks().list(&opts).await? {
            if network.name.as_ref().unwrap() == name {
                let network = Network::new(self.docker.clone(), network.id.unwrap());
                network.delete().await.ok();
            }
        }
        Ok(())
    }

    async fn stop_container(&self, name: &str) -> Result<()> {
        let opts = ContainerListOpts::builder().all(true).build();
        for container in self.docker.containers().list(&opts).await? {
            if container
                .names
                .as_ref()
                .unwrap()
                .iter()
                .any(|n| n.as_str().ends_with(name))
            {
                let container = Container::new(self.docker.clone(), container.id.unwrap());
                log::info!("stopping {}", name);
                container.stop(None).await?;
                container.delete().await.ok();
                break;
            }
        }
        Ok(())
    }

    async fn run_container(
        &self,
        name: String,
        opts: &ContainerCreateOpts,
        network: &Network,
    ) -> Result<Container> {
        log::info!("creating {}", name);
        let id = self.docker.containers().create(opts).await?.id().clone();
        let container = Container::new(self.docker.clone(), id.clone());

        let opts = ContainerConnectionOpts::builder(&id).build();
        network.connect(&opts).await?;

        container.start().await?;

        log::info!("starting {}", name);
        let container = Container::new(self.docker.clone(), id.clone());
        tokio::task::spawn(async move {
            let opts = LogsOpts::builder()
                .all()
                .follow(true)
                .stdout(true)
                .stderr(true)
                .build();
            let mut logs = container.logs(&opts);
            while let Some(chunk) = logs.next().await {
                match chunk {
                    Ok(TtyChunk::StdOut(stdout)) => {
                        let stdout = std::str::from_utf8(&stdout).unwrap_or_default();
                        log::info!("{}: stdout: {}", name, stdout);
                    }
                    Ok(TtyChunk::StdErr(stderr)) => {
                        let stderr = std::str::from_utf8(&stderr).unwrap_or_default();
                        log::info!("{}: stderr: {}", name, stderr);
                    }
                    Err(err) => {
                        log::error!("{}", err);
                    }
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
                }
                _ => break,
            }
        }

        Ok(container)
    }

    async fn run_node(&self, config: &BlockchainConfig, network: &Network) -> Result<Container> {
        let name = self.node_name(config);
        let mut opts = ContainerCreateOpts::builder()
            .name(&name)
            .image(config.node_image)
            .command((config.node_command)(config.network, config.node_port))
            .auto_remove(true)
            .attach_stdout(true)
            .attach_stderr(true)
            .publish(PublishPort::tcp(config.node_port as _))
            .expose(
                PublishPort::tcp(config.node_port as _),
                config.node_port as _,
            );
        for port in config.node_additional_ports {
            let port = *port as u32;
            opts = opts.expose(PublishPort::tcp(port), port);
        }
        let container = self.run_container(name, &opts.build(), network).await?;
        //wait_for_http(&format!("http://127.0.0.1:{}", config.node_port)).await?;
        tokio::time::sleep(Duration::from_secs(30)).await;
        Ok(container)
    }

    async fn run_connector(
        &self,
        config: &BlockchainConfig,
        network: &Network,
    ) -> Result<Container> {
        let name = self.connector_name(config);
        let link = self.node_name(config);
        let opts = ContainerCreateOpts::builder()
            .name(&name)
            .image(format!("analoglabs/connector-{}", config.blockchain))
            .command(vec![
                format!("--network={}", config.network),
                format!("--addr=0.0.0.0:{}", config.connector_port),
                format!("--node-addr={}:{}", link, config.node_port),
                "--path=/data".into(),
            ])
            .auto_remove(true)
            .attach_stdout(true)
            .attach_stderr(true)
            .expose(
                PublishPort::tcp(config.connector_port as _),
                config.connector_port as _,
            )
            .build();
        let container = self.run_container(name, &opts, network).await?;
        wait_for_http(&format!("http://127.0.0.1:{}", config.connector_port)).await?;
        Ok(container)
    }
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
    let status = inspect
        .state
        .and_then(|state| state.health)
        .and_then(|health| health.status);
    let Some(status) = status else {
        return Ok(None);
    };
    Ok(Some(match status.as_str() {
        "none" => Health::None,
        "starting" => Health::Starting,
        "healthy" => Health::Healthy,
        "unhealthy" => Health::Unhealthy,
        status => anyhow::bail!("unknown status {}", status),
    }))
}

async fn wait_for_http(url: &str) -> Result<()> {
    loop {
        match surf::get(url).await {
            Ok(_) => {
                break;
            }
            Err(err) => {
                log::error!("{}", err);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    Ok(())
}
