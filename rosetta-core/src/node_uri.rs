use fluent_uri::Uri;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeUriError {
    #[error("Invalid uri")]
    InvalidUri,
    #[error("Invalid uri scheme")]
    InvalidScheme,
    #[error("Invalid uri port")]
    InvalidPort,
    #[error("Invalid uri host")]
    InvalidHost,
}

#[derive(Clone)]
pub struct NodeUri<'a> {
    pub scheme: &'a str,
    pub userinfo: Option<&'a str>,
    pub host: &'a str,
    pub port: u16,
}

impl<'a> NodeUri<'a> {
    pub fn parse(s: &'a str) -> Result<Self, NodeUriError> {
        let uri = Uri::parse(s).map_err(|_| NodeUriError::InvalidUri)?;

        // Parse uri components.
        let scheme = uri
            .scheme()
            .map(|scheme| scheme.as_str())
            .ok_or(NodeUriError::InvalidScheme)?;
        let authority = uri.authority().ok_or(NodeUriError::InvalidHost)?;
        let port = authority
            .port()
            .ok_or(NodeUriError::InvalidPort)?
            .parse::<u16>()
            .map_err(|_| NodeUriError::InvalidPort)?;
        let host = authority.host().as_str();
        let userinfo = authority.userinfo().map(|userinfo| userinfo.as_str());

        Ok(Self {
            scheme,
            userinfo,
            host,
            port,
        })
    }

    pub fn with_host<'b, 'c: 'b, S: Into<&'c str>>(&'b self, host: S) -> NodeUri<'b> {
        NodeUri {
            scheme: self.scheme,
            userinfo: self.userinfo,
            host: host.into(),
            port: self.port,
        }
    }
}

impl Display for NodeUri<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(userinfo) = self.userinfo {
            write!(
                f,
                "{}://{}@{}:{}",
                self.scheme, userinfo, self.host, self.port
            )
        } else {
            write!(f, "{}://{}:{}", self.scheme, self.host, self.port)
        }
    }
}
