use anyhow::Context;
use dirs::home_dir;
use env_vars::{DOCKER_CONFIG, DOCKER_CONTEXT, DOCKER_HOST};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, str::FromStr};

// Ref: https://github.com/docker/cli/blob/v24.0.5/opts/hosts.go#L11-L33
#[cfg(unix)]
const DEFAULT_DOCKER_ENDPOINT: &str = "unix:///var/run/docker.sock";

/// For windows the default endpoint is "npipe:////./pipe/docker_engine"
/// But currently this is not supported by docker-api, using to default tcp endpoint instead
/// (https://github.com/vv9k/docker-api-rs/issues/57)
#[cfg(not(unix))]
const DEFAULT_DOCKER_ENDPOINT: &str = "tcp://127.0.0.1:2375";

/// List of environment variables supported by the `docker` command
pub mod env_vars {
    use std::ffi::OsStr;

    /// The location of your client configuration files.
    pub const DOCKER_CONFIG: &str = "DOCKER_CONFIG";

    /// Name of the `docker context` to use (overrides `DOCKER_HOST` env var and default context set
    /// with `docker context use`)
    pub const DOCKER_CONTEXT: &str = "DOCKER_CONTEXT";

    /// Daemon socket to connect to.
    pub const DOCKER_HOST: &str = "DOCKER_HOST";

    /// Load an environment variable and verify if it's not empty
    pub fn non_empty_var<K: AsRef<OsStr>>(key: K) -> Option<String> {
        let value = std::env::var(key).ok()?;
        if value.trim().is_empty() {
            None
        } else {
            Some(value)
        }
    }
}

/// Find the docker endpoint
/// 1. Try to load the endpoint from the `DOCKER_CONTEXT` environment variable
/// 2. Try to load the endpoint from the `DOCKER_HOST` environment variable
/// 3. Try to load the endpoint from the `config.json` file
/// 4. Return the default endpoint
pub fn docker_endpoint() -> String {
    // Try to load the endpoint from the `DOCKER_CONTEXT` environment variable
    if let Some(context) = env_vars::non_empty_var(DOCKER_CONTEXT) {
        match docker_config_dir() {
            Ok(config_dir) => match find_context_endpoint(config_dir, &context) {
                Ok(endpoint) => return endpoint,
                Err(error) => log::warn!("Failed to find {context} endpoint: {error}"),
            },
            Err(error) => log::warn!("Can't find the config directory: {error}"),
        }
    }

    // Try to load the endpoint from the `DOCKER_HOST` environment variable
    if let Some(host) = env_vars::non_empty_var(DOCKER_HOST) {
        return host;
    }

    // If the config directory exists, try to load the endpoint from the config.json file
    // otherwise return the default endpoint
    docker_config_dir()
        .ok()
        .and_then(|config| endpoint_from_config(config).ok())
        .unwrap_or_else(|| DEFAULT_DOCKER_ENDPOINT.to_string())
}

/// By default, the Docker-cli stores its configuration files in a directory called
/// `.docker` within your `$HOME` directory. Is possible to override the default location
/// of the configuration files via the `DOCKER_CONFIG` environment variable
///
/// Reference:
/// <https://github.com/docker/cli/blob/v24.0.5/man/docker-config-json.5.md>
pub fn docker_config_dir() -> anyhow::Result<PathBuf> {
    // Verifies if the config directory exists
    let directory_exists = |directory: PathBuf| {
        if directory.exists() {
            Ok(directory)
        } else {
            anyhow::bail!("Docker config directory not found: '{directory:?}'");
        }
    };

    // Try to find the config directory from the `DOCKER_CONFIG` environment variable
    if let Some(config) =
        env_vars::non_empty_var(DOCKER_CONFIG).and_then(|path| PathBuf::from_str(&path).ok())
    {
        return directory_exists(config);
    }

    // Use the default config directory at $HOME/.docker/
    let Some(config_directory) = home_dir().map(|path| path.join(".docker/")) else {
        anyhow::bail!("Could not find home directory");
    };
    directory_exists(config_directory)
}

/// By default, the Docker-cli stores its configuration files in a directory called
/// `.docker` within your `$HOME` directory. Is possible to override the default location
/// of the configuration files via the `DOCKER_CONFIG` environment variable
///
/// Reference:
/// - <https://github.com/docker/cli/blob/v24.0.5/man/docker-config-json.5.md>
/// - <https://github.com/docker/cli/blob/v24.0.5/cli/config/configfile/file.go#L17-L44>
pub fn endpoint_from_config(config_dir: PathBuf) -> anyhow::Result<String> {
    // Extract the current context from config.json file
    let config_file = config_dir.join("config.json");
    if !config_file.is_file() {
        anyhow::bail!("Docker config.json file not found.");
    }

    // Read the config.json file and extract the current context
    let current_context = fs::read_to_string(config_file)
        .context("Failed to read docker config.json")?
        .parse::<serde_json::Value>()
        .context("config.json is not a valid json")?
        .get("currentContext")
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| "default".to_string(), str::to_string);

    // Find the endpoint
    find_context_endpoint(config_dir, &current_context)
}

/// Find the Docker Endpoint of a given context, the host endpoint is located at:
/// UNIX:
///  - $HOME/.docker/contexts/meta/<sha256 context>/meta.json
/// Windows:
/// - %USERPROFILE%\.docker\contexts\meta\<sha256 context>\meta.json
///
/// Is possible to list contexts by running `docker context ls`
pub fn find_context_endpoint(mut config_dir: PathBuf, context: &str) -> anyhow::Result<String> {
    let metadata_filepath = {
        // $HOME/.docker/contexts/meta/<sha256 context>/meta.json
        let digest = sha256_digest(context);
        config_dir.extend(["contexts", "meta", digest.as_str(), "meta.json"]);
        config_dir
    };

    if !metadata_filepath.is_file() {
        anyhow::bail!("Docker context metadata file not found: '{metadata_filepath:?}'");
    }

    let host = fs::read_to_string(metadata_filepath)
        .context("Cannot read meta.json")?
        .parse::<serde_json::Value>()
        .context("meta.json is not a valid json")?
        .get("Endpoints")
        .context("meta.json does not contain Endpoints")?
        .get("docker")
        .context("meta.json does not contain Endpoints.docker")?
        .get("Host")
        .context("meta.json does not contain Endpoints.docker.Host")?
        .as_str()
        .context("meta.json Endpoints.docker.Host is not a string")?
        .to_string();

    Ok(host)
}

fn sha256_digest(name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use docker_api::{opts::ContainerListOpts, Docker};

    #[test]
    fn test_sha256_digest() {
        assert_eq!(
            sha256_digest("colima"),
            "f24fd3749c1368328e2b149bec149cb6795619f244c5b584e844961215dadd16"
        );
    }

    #[tokio::test]
    async fn endpoint_works() {
        // Obs: docker must be running
        let host = docker_endpoint();
        let docker = Docker::new(&host).unwrap();
        let result = docker.containers().list(&ContainerListOpts::default()).await;
        assert!(result.is_ok());
    }
}
