use anyhow::Result;
use docker_api::conn::TtyChunk;
use docker_api::opts::{
    ContainerConnectionOpts, ContainerCreateOpts, ContainerListOpts, LogsOpts, NetworkCreateOpts,
    PublishPort,
};
use docker_api::{Container, Id, Network};
use futures::stream::StreamExt;
use rosetta::client::Client;
use rosetta::{BlockchainClient, BlockchainConfig};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone)]
pub struct Docker {
    docker: docker_api::Docker,
    networks: Arc<Mutex<Vec<Network>>>,
    containers: Arc<Mutex<Vec<Container>>>,
}

impl Docker {
    #[cfg(unix)]
    pub fn new() -> Result<Self> {
        Ok(Self::inner_new(docker_api::Docker::unix(
            "/var/run/docker.sock",
        )))
    }

    #[cfg(not(unix))]
    pub fn new() -> Result<Self> {
        Ok(Self::inner_new(docker_api::Docker::new(
            "tcp://127.0.0.1:8080",
        )?))
    }

    fn inner_new(docker: docker_api::Docker) -> Self {
        Self {
            docker,
            networks: Default::default(),
            containers: Default::default(),
        }
    }

    #[allow(unused)]
    async fn cleanup(&self) -> Result<()> {
        let opts = ContainerListOpts::builder().all(true).build();
        for container in self.docker.containers().list(&opts).await? {
            let container = Container::new(self.docker.clone(), container.id.unwrap());
            container.stop(None).await.ok();
            container.delete().await.ok();
        }
        self.docker.networks().prune(&Default::default()).await?;
        Ok(())
    }

    async fn create_network(&self) -> Result<Network> {
        let opts = NetworkCreateOpts::builder("rosetta-docker").build();
        let network = self.docker.networks().create(&opts).await?;
        let id = network.id().clone();
        self.networks.lock().unwrap().push(network);
        Ok(Network::new(self.docker.clone(), id))
    }

    async fn run_container(
        &self,
        name: String,
        opts: &ContainerCreateOpts,
        network: Option<&Network>,
    ) -> Result<Id> {
        log::info!("creating {}", name);
        let id = self.docker.containers().create(&opts).await?.id().clone();
        let container = Container::new(self.docker.clone(), id.clone());

        if let Some(network) = network.as_ref() {
            let opts = ContainerConnectionOpts::builder(&id).build();
            network.connect(&opts).await?;
        }

        container.start().await?;
        self.containers.lock().unwrap().push(container);

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
                        log::info!("{}: {}", name, stdout);
                    }
                    Ok(TtyChunk::StdErr(stderr)) => {
                        let stderr = std::str::from_utf8(&stderr).unwrap_or_default();
                        log::error!("{}: {}", name, stderr);
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

        Ok(id)
    }

    async fn run_node(&self, config: &BlockchainConfig, network: Option<&Network>) -> Result<Id> {
        let name = format!("node-{}-{}", config.blockchain, config.network);
        let mut opts = ContainerCreateOpts::builder()
            .name(&name)
            .image(config.node_image)
            .command(&config.node_command)
            .auto_remove(true)
            .attach_stdout(true)
            .attach_stderr(true);
        if network.is_some() {
            opts = opts.publish(PublishPort::tcp(config.node_port as _));
        } else {
            opts = opts.expose(
                PublishPort::tcp(config.node_port as _),
                config.node_port as _,
            );
        }
        for port in config.node_additional_ports {
            let port = *port as u32;
            opts = opts.expose(PublishPort::tcp(port), port);
        }
        self.run_container(name, &opts.build(), network).await
    }

    pub async fn node<T: BlockchainClient>(&self, config: &BlockchainConfig) -> Result<T> {
        self.run_node(config, None).await?;
        let addr = format!("127.0.0.1:{}", config.node_port);
        wait_for_http(&format!("http://{}", addr)).await?;
        T::new(config.network, &addr).await
    }

    async fn run_connector(
        &self,
        config: &BlockchainConfig,
        network: Option<&Network>,
    ) -> Result<Id> {
        let name = format!("connector-{}-{}", config.blockchain, config.network);
        let link = format!("node-{}-{}", config.blockchain, config.network);
        let opts = ContainerCreateOpts::builder()
            .name(&name)
            .image(format!("connector-{}", config.blockchain))
            .command(vec![
                format!("--network={}", config.network),
                format!("--addr=0.0.0.0:{}", config.connector_port),
                format!("--node-addr={}:{}", link, config.node_port),
            ])
            .auto_remove(true)
            .attach_stdout(true)
            .attach_stderr(true)
            .expose(
                PublishPort::tcp(config.connector_port as _),
                config.connector_port as _,
            )
            .build();
        self.run_container(name, &opts, network).await
    }

    pub async fn connector(&self, config: &BlockchainConfig) -> Result<Client> {
        let network = self.create_network().await?;
        self.run_node(config, Some(&network)).await?;
        self.run_connector(config, Some(&network)).await?;
        let url = format!("http://127.0.0.1:{}", config.connector_port);
        wait_for_http(&url).await?;
        Client::new(&url)
    }

    pub async fn shutdown(&self) -> Result<()> {
        for container in self.containers.lock().unwrap().drain(..) {
            container.stop(None).await?;
            //container.delete().await?;
        }
        for network in self.networks.lock().unwrap().drain(..) {
            network.delete().await?;
        }
        Ok(())
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
        match surf::get(&url).await {
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
