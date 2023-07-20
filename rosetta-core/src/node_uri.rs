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

/// The URI is structured as follows:
///
/// ```notrust
/// abc://username:password@example.com:1234/path/data?key=value&key2=value2#fragid1
/// |-|  |----------------| |---------| |--||--------| |-------------------| |-----|
///  |            |              |       |      |               |              |
/// scheme    userinfo         host    port   path            query         fragment
/// ```
#[derive(Clone)]
pub struct NodeUri<'a> {
    pub scheme: &'a str,
    pub userinfo: Option<&'a str>,
    pub host: &'a str,
    pub port: u16,
    pub path: &'a str,
    pub query: Option<&'a str>,
    pub fragment: Option<&'a str>,
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
            path: uri.path().as_str(),
            query: uri.query().map(|query| query.as_str()),
            fragment: uri.fragment().map(|fragment| fragment.as_str()),
        })
    }

    pub fn with_host<'b, 'c: 'b>(&'b self, host: &'c str) -> NodeUri<'b> {
        NodeUri {
            scheme: self.scheme,
            userinfo: self.userinfo,
            host,
            port: self.port,
            path: self.path,
            query: self.query,
            fragment: self.fragment,
        }
    }
}

impl Display for NodeUri<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // scheme://
        write!(f, "{}://", self.scheme)?;

        // userinfo@
        if let Some(userinfo) = self.userinfo {
            write!(f, "{}@", userinfo)?;
        }

        // host:port/path
        write!(f, "{}:{}{}", self.host, self.port, self.path)?;

        // ?query
        if let Some(query) = self.query {
            write!(f, "?{}", query)?;
        }

        // #fragment
        if let Some(fragment) = self.fragment {
            write!(f, "#{}", fragment)?;
        }
        Ok(())
    }
}
