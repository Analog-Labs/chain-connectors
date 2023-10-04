use fluent_uri::Uri;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Parses a URI reference from a byte sequence into a Uri<&str>.
    /// This function validates the input strictly except that UTF-8 validation is not performed on a percent-encoded registered name (see Section 3.2.2, RFC 3986 ). Care should be taken when dealing with such cases.
    /// # Errors
    ///
    /// The provided url must contain [`fluent_uri::Scheme`], [`fluent_uri::Host`] and port, otherwise returns `Err`
    pub fn parse(s: &'a str) -> Result<Self, NodeUriError> {
        let uri = Uri::parse(s).map_err(|_| NodeUriError::InvalidUri)?;

        // Parse uri components.
        let scheme = uri
            .scheme()
            .map(fluent_uri::Scheme::as_str)
            .ok_or(NodeUriError::InvalidScheme)?;
        if scheme.is_empty() {
            return Err(NodeUriError::InvalidScheme);
        }
        let authority = uri.authority().ok_or(NodeUriError::InvalidHost)?;
        let host = authority.host().as_str();
        if host.is_empty() {
            return Err(NodeUriError::InvalidHost);
        }
        let port = authority
            .port()
            .ok_or(NodeUriError::InvalidPort)?
            .parse::<u16>()
            .map_err(|_| NodeUriError::InvalidPort)?;
        let userinfo = authority.userinfo().map(fluent_uri::enc::EStr::as_str);

        Ok(Self {
            scheme,
            userinfo,
            host,
            port,
            path: uri.path().as_str(),
            query: uri.query().map(fluent_uri::enc::EStr::as_str),
            fragment: uri.fragment().map(fluent_uri::enc::EStr::as_str),
        })
    }

    #[must_use]
    pub const fn with_host<'b, 'c: 'b>(&'b self, host: &'c str) -> NodeUri<'b> {
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

    #[must_use]
    pub const fn with_scheme<'b, 'c: 'b>(&'b self, scheme: &'c str) -> NodeUri<'b> {
        NodeUri {
            scheme,
            userinfo: self.userinfo,
            host: self.host,
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
            write!(f, "{userinfo}@")?;
        }

        // host:port/path
        write!(f, "{}:{}{}", self.host, self.port, self.path)?;

        // ?query
        if let Some(query) = self.query {
            write!(f, "?{query}")?;
        }

        // #fragment
        if let Some(fragment) = self.fragment {
            write!(f, "#{fragment}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_uri_works() {
        let uri = NodeUri::parse("http://127.0.0.1:18443").unwrap();
        assert_eq!(uri.scheme, "http");
        assert_eq!(uri.userinfo, None);
        assert_eq!(uri.host, "127.0.0.1");
        assert_eq!(uri.port, 18443);
        assert_eq!(uri.path, "");
        assert_eq!(uri.query, None);
        assert_eq!(uri.fragment, None);
        assert_eq!(uri.to_string(), "http://127.0.0.1:18443");
    }

    #[test]
    fn parse_complex_uri_works() {
        let uri_str =
            "wss://username:password@some-random-host:12345/path/to?key=value&key2=value2#fragment";
        let uri = NodeUri::parse(uri_str).unwrap();
        assert_eq!(uri.scheme, "wss");
        assert_eq!(uri.userinfo, Some("username:password"));
        assert_eq!(uri.host, "some-random-host");
        assert_eq!(uri.port, 12345);
        assert_eq!(uri.path, "/path/to");
        assert_eq!(uri.query, Some("key=value&key2=value2"));
        assert_eq!(uri.fragment, Some("fragment"));
        assert_eq!(uri.to_string(), uri_str);
    }

    #[test]
    fn validate_uri() {
        let uri_str = "http:// :8080";
        let uri = NodeUri::parse(uri_str);
        assert_eq!(uri, Err(NodeUriError::InvalidUri));
    }

    #[test]
    fn validate_scheme() {
        let uri_str = "//127.0.0.1:12345";
        let uri = NodeUri::parse(uri_str);
        assert_eq!(uri, Err(NodeUriError::InvalidScheme));
    }

    #[test]
    fn validate_host() {
        let uri_str = "wss://";
        let uri = NodeUri::parse(uri_str);
        assert_eq!(uri, Err(NodeUriError::InvalidHost));
    }

    #[test]
    fn validate_port() {
        let uris = [
            "wss://username:password@some-random-host/path/to?key=value&key2=value2#fragment",
            "http://127.0.0.1/path/to",
            "http://127.0.0.1:123456",
            "http://127.0.0.1:999999",
        ];
        for uri_str in uris {
            let uri = NodeUri::parse(uri_str);
            assert_eq!(uri, Err(NodeUriError::InvalidPort));
        }
    }
}
