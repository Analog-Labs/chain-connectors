use anyhow::Result;
use docker_api::opts::{ContainerCreateOpts, PublishPort, PullOpts};
use docker_api::Container;
use std::net::SocketAddr;
use std::time::Duration;

pub trait BlockchainNode: Sized {
    fn new(network: &str, addr: SocketAddr) -> Self;
    fn name(&self) -> &str;
    fn image(&self) -> &str;
    fn command(&self) -> &[String];
    fn expose(&self) -> &[u16];
    fn pull_opts(&self) -> PullOpts {
        PullOpts::builder().image(self.image()).build()
    }
    fn container_create_opts(&self) -> ContainerCreateOpts {
        let mut opts = ContainerCreateOpts::builder()
            .name(self.name())
            .image(self.image())
            .command(self.command())
            .auto_remove(true)
            .attach_stdout(true)
            .attach_stderr(true);
        for port in self.expose() {
            let port = *port as u32;
            //opts = opts.publish(PublishPort::tcp(*port as _));
            opts = opts.expose(PublishPort::tcp(port), port);
        }
        opts.build()
    }
}

#[derive(Clone)]
pub struct Docker {
    docker: docker_api::Docker,
}

impl Docker {
    #[cfg(unix)]
    pub fn new() -> Result<Self> {
        let docker = docker_api::Docker::unix("/var/run/docker.sock");
        Ok(Self { docker })
    }

    #[cfg(not(unix))]
    pub fn new() -> Result<Self> {
        let docker = docker_api::Docker::new("tcp://127.0.0.1:8080")?;
        Ok(Self { docker })
    }

    async fn create_container(&self, opts: &ContainerCreateOpts) -> Result<Container> {
        let info = self.docker.containers().create(&opts).await?;
        Ok(self.docker.containers().get(info.id().clone()))
    }

    pub async fn run_node<T: BlockchainNode>(&self, node: &T) -> Result<Handle> {
        let container = self.create_container(&node.container_create_opts()).await?;
        container.start().await?;
        let handle = Handle::new(container);
        loop {
            match handle.health().await? {
                Some(Health::Unhealthy) => anyhow::bail!("healthcheck reports unhealthy"),
                Some(Health::Starting) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                _ => break,
            }
        }
        Ok(handle)
    }
}

#[must_use]
pub struct Handle {
    container: Container,
}

impl Handle {
    fn new(container: Container) -> Self {
        Self { container }
    }

    pub async fn health(&self) -> Result<Option<Health>> {
        let inspect = self.container.inspect().await?;
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

    pub async fn stop(self) -> Result<()> {
        self.container.stop(None).await?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Health {
    None,
    Starting,
    Healthy,
    Unhealthy,
}

pub async fn wait_for_http(url: &str) -> Result<()> {
    loop {
        match surf::get(url).await {
            Ok(_) => {
                break;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
    Ok(())
}
