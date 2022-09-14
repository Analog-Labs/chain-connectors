#![allow(unused_qualifications)]

#[cfg(any(feature = "client", feature = "server"))]
use crate::header;
use crate::models;

/// An AccountBalanceRequest is utilized to make a balance request on the /account/balance endpoint. If the block_identifier is populated, a historical balance query should be performed.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AccountBalanceRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "account_identifier")]
    pub account_identifier: models::AccountIdentifier,

    #[serde(rename = "block_identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_identifier: Option<models::PartialBlockIdentifier>,

    /// In some cases, the caller may not want to retrieve all available balances for an AccountIdentifier. If the currencies field is populated, only balances for the specified currencies will be returned. If not populated, all available balances will be returned.
    #[serde(rename = "currencies")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currencies: Option<Vec<models::Currency>>,
}

impl AccountBalanceRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        account_identifier: models::AccountIdentifier,
    ) -> AccountBalanceRequest {
        AccountBalanceRequest {
            network_identifier: network_identifier,
            account_identifier: account_identifier,
            block_identifier: None,
            currencies: None,
        }
    }
}

/// Converts the AccountBalanceRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AccountBalanceRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping account_identifier in query parameter serialization

        // Skipping block_identifier in query parameter serialization

        // Skipping currencies in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AccountBalanceRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AccountBalanceRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub account_identifier: Vec<models::AccountIdentifier>,
            pub block_identifier: Vec<models::PartialBlockIdentifier>,
            pub currencies: Vec<Vec<models::Currency>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AccountBalanceRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "account_identifier" => intermediate_rep.account_identifier.push(<models::AccountIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "block_identifier" => intermediate_rep.block_identifier.push(<models::PartialBlockIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "currencies" => return std::result::Result::Err("Parsing a container in this style is not supported in AccountBalanceRequest".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing AccountBalanceRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AccountBalanceRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in AccountBalanceRequest".to_string())?,
            account_identifier: intermediate_rep
                .account_identifier
                .into_iter()
                .next()
                .ok_or("account_identifier missing in AccountBalanceRequest".to_string())?,
            block_identifier: intermediate_rep.block_identifier.into_iter().next(),
            currencies: intermediate_rep.currencies.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AccountBalanceRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AccountBalanceRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AccountBalanceRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AccountBalanceRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<AccountBalanceRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AccountBalanceRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AccountBalanceRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// An AccountBalanceResponse is returned on the /account/balance endpoint. If an account has a balance for each AccountIdentifier describing it (ex: an ERC-20 token balance on a few smart contracts), an account balance request must be made with each AccountIdentifier.  The `coins` field was removed and replaced by by `/account/coins` in `v1.4.7`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AccountBalanceResponse {
    #[serde(rename = "block_identifier")]
    pub block_identifier: models::BlockIdentifier,

    /// A single account may have a balance in multiple currencies.
    #[serde(rename = "balances")]
    pub balances: Vec<models::Amount>,

    /// Account-based blockchains that utilize a nonce or sequence number should include that number in the metadata. This number could be unique to the identifier or global across the account address.
    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl AccountBalanceResponse {
    pub fn new(
        block_identifier: models::BlockIdentifier,
        balances: Vec<models::Amount>,
    ) -> AccountBalanceResponse {
        AccountBalanceResponse {
            block_identifier: block_identifier,
            balances: balances,
            metadata: None,
        }
    }
}

/// Converts the AccountBalanceResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AccountBalanceResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping block_identifier in query parameter serialization

        // Skipping balances in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AccountBalanceResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AccountBalanceResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub block_identifier: Vec<models::BlockIdentifier>,
            pub balances: Vec<Vec<models::Amount>>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AccountBalanceResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "block_identifier" => intermediate_rep.block_identifier.push(<models::BlockIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "balances" => return std::result::Result::Err("Parsing a container in this style is not supported in AccountBalanceResponse".to_string()),
                    "metadata" => intermediate_rep.metadata.push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing AccountBalanceResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AccountBalanceResponse {
            block_identifier: intermediate_rep
                .block_identifier
                .into_iter()
                .next()
                .ok_or("block_identifier missing in AccountBalanceResponse".to_string())?,
            balances: intermediate_rep
                .balances
                .into_iter()
                .next()
                .ok_or("balances missing in AccountBalanceResponse".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AccountBalanceResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AccountBalanceResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AccountBalanceResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AccountBalanceResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<AccountBalanceResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AccountBalanceResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AccountBalanceResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// AccountCoinsRequest is utilized to make a request on the /account/coins endpoint.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AccountCoinsRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "account_identifier")]
    pub account_identifier: models::AccountIdentifier,

    /// Include state from the mempool when looking up an account's unspent coins. Note, using this functionality breaks any guarantee of idempotency.
    #[serde(rename = "include_mempool")]
    pub include_mempool: bool,

    /// In some cases, the caller may not want to retrieve coins for all currencies for an AccountIdentifier. If the currencies field is populated, only coins for the specified currencies will be returned. If not populated, all unspent coins will be returned.
    #[serde(rename = "currencies")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currencies: Option<Vec<models::Currency>>,
}

impl AccountCoinsRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        account_identifier: models::AccountIdentifier,
        include_mempool: bool,
    ) -> AccountCoinsRequest {
        AccountCoinsRequest {
            network_identifier: network_identifier,
            account_identifier: account_identifier,
            include_mempool: include_mempool,
            currencies: None,
        }
    }
}

/// Converts the AccountCoinsRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AccountCoinsRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping account_identifier in query parameter serialization

        params.push("include_mempool".to_string());
        params.push(self.include_mempool.to_string());

        // Skipping currencies in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AccountCoinsRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AccountCoinsRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub account_identifier: Vec<models::AccountIdentifier>,
            pub include_mempool: Vec<bool>,
            pub currencies: Vec<Vec<models::Currency>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AccountCoinsRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "account_identifier" => intermediate_rep.account_identifier.push(
                        <models::AccountIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "include_mempool" => intermediate_rep.include_mempool.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "currencies" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in AccountCoinsRequest"
                            .to_string(),
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AccountCoinsRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AccountCoinsRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in AccountCoinsRequest".to_string())?,
            account_identifier: intermediate_rep
                .account_identifier
                .into_iter()
                .next()
                .ok_or("account_identifier missing in AccountCoinsRequest".to_string())?,
            include_mempool: intermediate_rep
                .include_mempool
                .into_iter()
                .next()
                .ok_or("include_mempool missing in AccountCoinsRequest".to_string())?,
            currencies: intermediate_rep.currencies.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AccountCoinsRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AccountCoinsRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AccountCoinsRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AccountCoinsRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<AccountCoinsRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AccountCoinsRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AccountCoinsRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// AccountCoinsResponse is returned on the /account/coins endpoint and includes all unspent Coins owned by an AccountIdentifier.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AccountCoinsResponse {
    #[serde(rename = "block_identifier")]
    pub block_identifier: models::BlockIdentifier,

    /// If a blockchain is UTXO-based, all unspent Coins owned by an account_identifier should be returned alongside the balance. It is highly recommended to populate this field so that users of the Rosetta API implementation don't need to maintain their own indexer to track their UTXOs.
    #[serde(rename = "coins")]
    pub coins: Vec<models::Coin>,

    /// Account-based blockchains that utilize a nonce or sequence number should include that number in the metadata. This number could be unique to the identifier or global across the account address.
    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl AccountCoinsResponse {
    pub fn new(
        block_identifier: models::BlockIdentifier,
        coins: Vec<models::Coin>,
    ) -> AccountCoinsResponse {
        AccountCoinsResponse {
            block_identifier: block_identifier,
            coins: coins,
            metadata: None,
        }
    }
}

/// Converts the AccountCoinsResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AccountCoinsResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping block_identifier in query parameter serialization

        // Skipping coins in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AccountCoinsResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AccountCoinsResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub block_identifier: Vec<models::BlockIdentifier>,
            pub coins: Vec<Vec<models::Coin>>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AccountCoinsResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "block_identifier" => intermediate_rep.block_identifier.push(<models::BlockIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "coins" => return std::result::Result::Err("Parsing a container in this style is not supported in AccountCoinsResponse".to_string()),
                    "metadata" => intermediate_rep.metadata.push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing AccountCoinsResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AccountCoinsResponse {
            block_identifier: intermediate_rep
                .block_identifier
                .into_iter()
                .next()
                .ok_or("block_identifier missing in AccountCoinsResponse".to_string())?,
            coins: intermediate_rep
                .coins
                .into_iter()
                .next()
                .ok_or("coins missing in AccountCoinsResponse".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AccountCoinsResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AccountCoinsResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AccountCoinsResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AccountCoinsResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<AccountCoinsResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AccountCoinsResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AccountCoinsResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// The account_identifier uniquely identifies an account within a network. All fields in the account_identifier are utilized to determine this uniqueness (including the metadata field, if populated).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AccountIdentifier {
    /// The address may be a cryptographic public key (or some encoding of it) or a provided username.
    #[serde(rename = "address")]
    pub address: String,

    #[serde(rename = "sub_account")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_account: Option<models::SubAccountIdentifier>,

    /// Blockchains that utilize a username model (where the address is not a derivative of a cryptographic public key) should specify the public key(s) owned by the address in metadata.
    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl AccountIdentifier {
    pub fn new(address: String) -> AccountIdentifier {
        AccountIdentifier {
            address: address,
            sub_account: None,
            metadata: None,
        }
    }
}

/// Converts the AccountIdentifier value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AccountIdentifier {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("address".to_string());
        params.push(self.address.to_string());

        // Skipping sub_account in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AccountIdentifier value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AccountIdentifier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub address: Vec<String>,
            pub sub_account: Vec<models::SubAccountIdentifier>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AccountIdentifier".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "address" => intermediate_rep.address.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "sub_account" => intermediate_rep.sub_account.push(
                        <models::SubAccountIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AccountIdentifier".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AccountIdentifier {
            address: intermediate_rep
                .address
                .into_iter()
                .next()
                .ok_or("address missing in AccountIdentifier".to_string())?,
            sub_account: intermediate_rep.sub_account.into_iter().next(),
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AccountIdentifier> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AccountIdentifier>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AccountIdentifier>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AccountIdentifier - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<AccountIdentifier>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AccountIdentifier as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AccountIdentifier - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Allow specifies supported Operation status, Operation types, and all possible error statuses. This Allow object is used by clients to validate the correctness of a Rosetta Server implementation. It is expected that these clients will error if they receive some response that contains any of the above information that is not specified here.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Allow {
    /// All Operation.Status this implementation supports. Any status that is returned during parsing that is not listed here will cause client validation to error.
    #[serde(rename = "operation_statuses")]
    pub operation_statuses: Vec<models::OperationStatus>,

    /// All Operation.Type this implementation supports. Any type that is returned during parsing that is not listed here will cause client validation to error.
    #[serde(rename = "operation_types")]
    pub operation_types: Vec<String>,

    /// All Errors that this implementation could return. Any error that is returned during parsing that is not listed here will cause client validation to error.
    #[serde(rename = "errors")]
    pub errors: Vec<models::Error>,

    /// Any Rosetta implementation that supports querying the balance of an account at any height in the past should set this to true.
    #[serde(rename = "historical_balance_lookup")]
    pub historical_balance_lookup: bool,

    /// If populated, `timestamp_start_index` indicates the first block index where block timestamps are considered valid (i.e. all blocks less than `timestamp_start_index` could have invalid timestamps). This is useful when the genesis block (or blocks) of a network have timestamp 0.  If not populated, block timestamps are assumed to be valid for all available blocks.
    #[serde(rename = "timestamp_start_index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_start_index: Option<i64>,

    /// All methods that are supported by the /call endpoint. Communicating which parameters should be provided to /call is the responsibility of the implementer (this is en lieu of defining an entire type system and requiring the implementer to define that in Allow).
    #[serde(rename = "call_methods")]
    pub call_methods: swagger::Nullable<Vec<String>>,

    /// BalanceExemptions is an array of BalanceExemption indicating which account balances could change without a corresponding Operation.  BalanceExemptions should be used sparingly as they may introduce significant complexity for integrators that attempt to reconcile all account balance changes.  If your implementation relies on any BalanceExemptions, you MUST implement historical balance lookup (the ability to query an account balance at any BlockIdentifier).
    #[serde(rename = "balance_exemptions")]
    pub balance_exemptions: swagger::Nullable<Vec<models::BalanceExemption>>,

    /// Any Rosetta implementation that can update an AccountIdentifier's unspent coins based on the contents of the mempool should populate this field as true. If false, requests to `/account/coins` that set `include_mempool` as true will be automatically rejected.
    #[serde(rename = "mempool_coins")]
    pub mempool_coins: bool,

    #[serde(rename = "block_hash_case")]
    #[serde(deserialize_with = "swagger::nullable_format::deserialize_optional_nullable")]
    #[serde(default = "swagger::nullable_format::default_optional_nullable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash_case: Option<swagger::Nullable<models::Case>>,

    #[serde(rename = "transaction_hash_case")]
    #[serde(deserialize_with = "swagger::nullable_format::deserialize_optional_nullable")]
    #[serde(default = "swagger::nullable_format::default_optional_nullable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash_case: Option<swagger::Nullable<models::Case>>,
}

impl Allow {
    pub fn new(
        operation_statuses: Vec<models::OperationStatus>,
        operation_types: Vec<String>,
        errors: Vec<models::Error>,
        historical_balance_lookup: bool,
        call_methods: swagger::Nullable<Vec<String>>,
        balance_exemptions: swagger::Nullable<Vec<models::BalanceExemption>>,
        mempool_coins: bool,
    ) -> Allow {
        Allow {
            operation_statuses: operation_statuses,
            operation_types: operation_types,
            errors: errors,
            historical_balance_lookup: historical_balance_lookup,
            timestamp_start_index: None,
            call_methods: call_methods,
            balance_exemptions: balance_exemptions,
            mempool_coins: mempool_coins,
            block_hash_case: None,
            transaction_hash_case: None,
        }
    }
}

/// Converts the Allow value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Allow {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping operation_statuses in query parameter serialization

        params.push("operation_types".to_string());
        params.push(
            self.operation_types
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
                .to_string(),
        );

        // Skipping errors in query parameter serialization

        params.push("historical_balance_lookup".to_string());
        params.push(self.historical_balance_lookup.to_string());

        if let Some(ref timestamp_start_index) = self.timestamp_start_index {
            params.push("timestamp_start_index".to_string());
            params.push(timestamp_start_index.to_string());
        }

        // Skipping balance_exemptions in query parameter serialization

        params.push("mempool_coins".to_string());
        params.push(self.mempool_coins.to_string());

        // Skipping block_hash_case in query parameter serialization

        // Skipping transaction_hash_case in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Allow value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Allow {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub operation_statuses: Vec<Vec<models::OperationStatus>>,
            pub operation_types: Vec<Vec<String>>,
            pub errors: Vec<Vec<models::Error>>,
            pub historical_balance_lookup: Vec<bool>,
            pub timestamp_start_index: Vec<i64>,
            pub call_methods: Vec<Vec<String>>,
            pub balance_exemptions: Vec<Vec<models::BalanceExemption>>,
            pub mempool_coins: Vec<bool>,
            pub block_hash_case: Vec<models::Case>,
            pub transaction_hash_case: Vec<models::Case>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Allow".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "operation_statuses" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Allow"
                                .to_string(),
                        )
                    }
                    "operation_types" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Allow"
                                .to_string(),
                        )
                    }
                    "errors" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Allow"
                                .to_string(),
                        )
                    }
                    "historical_balance_lookup" => intermediate_rep.historical_balance_lookup.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "timestamp_start_index" => intermediate_rep.timestamp_start_index.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "call_methods" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Allow"
                                .to_string(),
                        )
                    }
                    "balance_exemptions" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Allow"
                                .to_string(),
                        )
                    }
                    "mempool_coins" => intermediate_rep.mempool_coins.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "block_hash_case" => {
                        return std::result::Result::Err(
                            "Parsing a nullable type in this style is not supported in Allow"
                                .to_string(),
                        )
                    }
                    "transaction_hash_case" => {
                        return std::result::Result::Err(
                            "Parsing a nullable type in this style is not supported in Allow"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Allow".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Allow {
            operation_statuses: intermediate_rep
                .operation_statuses
                .into_iter()
                .next()
                .ok_or("operation_statuses missing in Allow".to_string())?,
            operation_types: intermediate_rep
                .operation_types
                .into_iter()
                .next()
                .ok_or("operation_types missing in Allow".to_string())?,
            errors: intermediate_rep
                .errors
                .into_iter()
                .next()
                .ok_or("errors missing in Allow".to_string())?,
            historical_balance_lookup: intermediate_rep
                .historical_balance_lookup
                .into_iter()
                .next()
                .ok_or("historical_balance_lookup missing in Allow".to_string())?,
            timestamp_start_index: intermediate_rep.timestamp_start_index.into_iter().next(),
            call_methods: std::result::Result::Err(
                "Nullable types not supported in Allow".to_string(),
            )?,
            balance_exemptions: std::result::Result::Err(
                "Nullable types not supported in Allow".to_string(),
            )?,
            mempool_coins: intermediate_rep
                .mempool_coins
                .into_iter()
                .next()
                .ok_or("mempool_coins missing in Allow".to_string())?,
            block_hash_case: std::result::Result::Err(
                "Nullable types not supported in Allow".to_string(),
            )?,
            transaction_hash_case: std::result::Result::Err(
                "Nullable types not supported in Allow".to_string(),
            )?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Allow> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Allow>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Allow>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Allow - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Allow> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Allow as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => {
                    std::result::Result::Ok(header::IntoHeaderValue(value))
                }
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Allow - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Amount is some Value of a Currency. It is considered invalid to specify a Value without a Currency.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Amount {
    /// Value of the transaction in atomic units represented as an arbitrary-sized signed integer.  For example, 1 BTC would be represented by a value of 100000000.
    #[serde(rename = "value")]
    pub value: String,

    #[serde(rename = "currency")]
    pub currency: models::Currency,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Amount {
    pub fn new(value: String, currency: models::Currency) -> Amount {
        Amount {
            value: value,
            currency: currency,
            metadata: None,
        }
    }
}

/// Converts the Amount value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Amount {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("value".to_string());
        params.push(self.value.to_string());

        // Skipping currency in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Amount value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Amount {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub value: Vec<String>,
            pub currency: Vec<models::Currency>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Amount".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "value" => intermediate_rep.value.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "currency" => intermediate_rep.currency.push(
                        <models::Currency as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Amount".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Amount {
            value: intermediate_rep
                .value
                .into_iter()
                .next()
                .ok_or("value missing in Amount".to_string())?,
            currency: intermediate_rep
                .currency
                .into_iter()
                .next()
                .ok_or("currency missing in Amount".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Amount> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Amount>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Amount>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Amount - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Amount> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Amount as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Amount - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// BalanceExemption indicates that the balance for an exempt account could change without a corresponding Operation. This typically occurs with staking rewards, vesting balances, and Currencies with a dynamic supply.  Currently, it is possible to exempt an account from strict reconciliation by SubAccountIdentifier.Address or by Currency. This means that any account with SubAccountIdentifier.Address would be exempt or any balance of a particular Currency would be exempt, respectively.  BalanceExemptions should be used sparingly as they may introduce significant complexity for integrators that attempt to reconcile all account balance changes.  If your implementation relies on any BalanceExemptions, you MUST implement historical balance lookup (the ability to query an account balance at any BlockIdentifier).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct BalanceExemption {
    /// SubAccountAddress is the SubAccountIdentifier.Address that the BalanceExemption applies to (regardless of the value of SubAccountIdentifier.Metadata).
    #[serde(rename = "sub_account_address")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_account_address: Option<String>,

    #[serde(rename = "currency")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<models::Currency>,

    #[serde(rename = "exemption_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exemption_type: Option<models::ExemptionType>,
}

impl BalanceExemption {
    pub fn new() -> BalanceExemption {
        BalanceExemption {
            sub_account_address: None,
            currency: None,
            exemption_type: None,
        }
    }
}

/// Converts the BalanceExemption value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for BalanceExemption {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref sub_account_address) = self.sub_account_address {
            params.push("sub_account_address".to_string());
            params.push(sub_account_address.to_string());
        }

        // Skipping currency in query parameter serialization

        // Skipping exemption_type in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a BalanceExemption value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for BalanceExemption {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub sub_account_address: Vec<String>,
            pub currency: Vec<models::Currency>,
            pub exemption_type: Vec<models::ExemptionType>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing BalanceExemption".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "sub_account_address" => intermediate_rep.sub_account_address.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "currency" => intermediate_rep.currency.push(
                        <models::Currency as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "exemption_type" => intermediate_rep.exemption_type.push(
                        <models::ExemptionType as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing BalanceExemption".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(BalanceExemption {
            sub_account_address: intermediate_rep.sub_account_address.into_iter().next(),
            currency: intermediate_rep.currency.into_iter().next(),
            exemption_type: intermediate_rep.exemption_type.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<BalanceExemption> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<BalanceExemption>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<BalanceExemption>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for BalanceExemption - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<BalanceExemption>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <BalanceExemption as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into BalanceExemption - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Blocks contain an array of Transactions that occurred at a particular BlockIdentifier.  A hard requirement for blocks returned by Rosetta implementations is that they MUST be _inalterable_: once a client has requested and received a block identified by a specific BlockIndentifier, all future calls for that same BlockIdentifier must return the same block contents.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Block {
    #[serde(rename = "block_identifier")]
    pub block_identifier: models::BlockIdentifier,

    #[serde(rename = "parent_block_identifier")]
    pub parent_block_identifier: models::BlockIdentifier,

    /// The timestamp of the block in milliseconds since the Unix Epoch. The timestamp is stored in milliseconds because some blockchains produce blocks more often than once a second.
    #[serde(rename = "timestamp")]
    pub timestamp: i64,

    #[serde(rename = "transactions")]
    pub transactions: Vec<models::Transaction>,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Block {
    pub fn new(
        block_identifier: models::BlockIdentifier,
        parent_block_identifier: models::BlockIdentifier,
        timestamp: i64,
        transactions: Vec<models::Transaction>,
    ) -> Block {
        Block {
            block_identifier: block_identifier,
            parent_block_identifier: parent_block_identifier,
            timestamp: timestamp,
            transactions: transactions,
            metadata: None,
        }
    }
}

/// Converts the Block value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Block {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping block_identifier in query parameter serialization

        // Skipping parent_block_identifier in query parameter serialization

        params.push("timestamp".to_string());
        params.push(self.timestamp.to_string());

        // Skipping transactions in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Block value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Block {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub block_identifier: Vec<models::BlockIdentifier>,
            pub parent_block_identifier: Vec<models::BlockIdentifier>,
            pub timestamp: Vec<i64>,
            pub transactions: Vec<Vec<models::Transaction>>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Block".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "block_identifier" => intermediate_rep.block_identifier.push(
                        <models::BlockIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "parent_block_identifier" => intermediate_rep.parent_block_identifier.push(
                        <models::BlockIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "timestamp" => intermediate_rep.timestamp.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "transactions" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Block"
                                .to_string(),
                        )
                    }
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Block".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Block {
            block_identifier: intermediate_rep
                .block_identifier
                .into_iter()
                .next()
                .ok_or("block_identifier missing in Block".to_string())?,
            parent_block_identifier: intermediate_rep
                .parent_block_identifier
                .into_iter()
                .next()
                .ok_or("parent_block_identifier missing in Block".to_string())?,
            timestamp: intermediate_rep
                .timestamp
                .into_iter()
                .next()
                .ok_or("timestamp missing in Block".to_string())?,
            transactions: intermediate_rep
                .transactions
                .into_iter()
                .next()
                .ok_or("transactions missing in Block".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Block> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Block>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Block>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Block - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Block> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Block as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => {
                    std::result::Result::Ok(header::IntoHeaderValue(value))
                }
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Block - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// BlockEvent represents the addition or removal of a BlockIdentifier from storage. Streaming BlockEvents allows lightweight clients to update their own state without needing to implement their own syncing logic.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct BlockEvent {
    /// sequence is the unique identifier of a BlockEvent within the context of a NetworkIdentifier.
    #[serde(rename = "sequence")]
    pub sequence: i64,

    #[serde(rename = "block_identifier")]
    pub block_identifier: models::BlockIdentifier,

    #[serde(rename = "type")]
    pub r#type: models::BlockEventType,
}

impl BlockEvent {
    pub fn new(
        sequence: i64,
        block_identifier: models::BlockIdentifier,
        r#type: models::BlockEventType,
    ) -> BlockEvent {
        BlockEvent {
            sequence: sequence,
            block_identifier: block_identifier,
            r#type: r#type,
        }
    }
}

/// Converts the BlockEvent value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for BlockEvent {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("sequence".to_string());
        params.push(self.sequence.to_string());

        // Skipping block_identifier in query parameter serialization

        // Skipping type in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a BlockEvent value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for BlockEvent {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub sequence: Vec<i64>,
            pub block_identifier: Vec<models::BlockIdentifier>,
            pub r#type: Vec<models::BlockEventType>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing BlockEvent".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "sequence" => intermediate_rep.sequence.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "block_identifier" => intermediate_rep.block_identifier.push(
                        <models::BlockIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "type" => intermediate_rep.r#type.push(
                        <models::BlockEventType as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing BlockEvent".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(BlockEvent {
            sequence: intermediate_rep
                .sequence
                .into_iter()
                .next()
                .ok_or("sequence missing in BlockEvent".to_string())?,
            block_identifier: intermediate_rep
                .block_identifier
                .into_iter()
                .next()
                .ok_or("block_identifier missing in BlockEvent".to_string())?,
            r#type: intermediate_rep
                .r#type
                .into_iter()
                .next()
                .ok_or("type missing in BlockEvent".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<BlockEvent> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<BlockEvent>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<BlockEvent>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for BlockEvent - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<BlockEvent> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <BlockEvent as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into BlockEvent - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// BlockEventType determines if a BlockEvent represents the addition or removal of a block.
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum BlockEventType {
    #[serde(rename = "block_added")]
    Added,
    #[serde(rename = "block_removed")]
    Removed,
}

impl std::fmt::Display for BlockEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BlockEventType::Added => write!(f, "{}", "block_added"),
            BlockEventType::Removed => write!(f, "{}", "block_removed"),
        }
    }
}

impl std::str::FromStr for BlockEventType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "block_added" => std::result::Result::Ok(BlockEventType::Added),
            "block_removed" => std::result::Result::Ok(BlockEventType::Removed),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

/// The block_identifier uniquely identifies a block in a particular network.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct BlockIdentifier {
    /// This is also known as the block height.
    #[serde(rename = "index")]
    pub index: i64,

    /// This should be normalized according to the case specified in the block_hash_case network options.
    #[serde(rename = "hash")]
    pub hash: String,
}

impl BlockIdentifier {
    pub fn new(index: i64, hash: String) -> BlockIdentifier {
        BlockIdentifier {
            index: index,
            hash: hash,
        }
    }
}

/// Converts the BlockIdentifier value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for BlockIdentifier {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("index".to_string());
        params.push(self.index.to_string());

        params.push("hash".to_string());
        params.push(self.hash.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a BlockIdentifier value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for BlockIdentifier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub index: Vec<i64>,
            pub hash: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing BlockIdentifier".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "index" => intermediate_rep.index.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "hash" => intermediate_rep.hash.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing BlockIdentifier".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(BlockIdentifier {
            index: intermediate_rep
                .index
                .into_iter()
                .next()
                .ok_or("index missing in BlockIdentifier".to_string())?,
            hash: intermediate_rep
                .hash
                .into_iter()
                .next()
                .ok_or("hash missing in BlockIdentifier".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<BlockIdentifier> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<BlockIdentifier>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<BlockIdentifier>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for BlockIdentifier - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<BlockIdentifier>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <BlockIdentifier as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into BlockIdentifier - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// A BlockRequest is utilized to make a block request on the /block endpoint.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct BlockRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "block_identifier")]
    pub block_identifier: models::PartialBlockIdentifier,
}

impl BlockRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        block_identifier: models::PartialBlockIdentifier,
    ) -> BlockRequest {
        BlockRequest {
            network_identifier: network_identifier,
            block_identifier: block_identifier,
        }
    }
}

/// Converts the BlockRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for BlockRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping block_identifier in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a BlockRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for BlockRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub block_identifier: Vec<models::PartialBlockIdentifier>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing BlockRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "block_identifier" => intermediate_rep.block_identifier.push(
                        <models::PartialBlockIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing BlockRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(BlockRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in BlockRequest".to_string())?,
            block_identifier: intermediate_rep
                .block_identifier
                .into_iter()
                .next()
                .ok_or("block_identifier missing in BlockRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<BlockRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<BlockRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<BlockRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for BlockRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<BlockRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <BlockRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into BlockRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// A BlockResponse includes a fully-populated block or a partially-populated block with a list of other transactions to fetch (other_transactions).  As a result of the consensus algorithm of some blockchains, blocks can be omitted (i.e. certain block indices can be skipped). If a query for one of these omitted indices is made, the response should not include a `Block` object.  It is VERY important to note that blocks MUST still form a canonical, connected chain of blocks where each block has a unique index. In other words, the `PartialBlockIdentifier` of a block after an omitted block should reference the last non-omitted block.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct BlockResponse {
    #[serde(rename = "block")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<models::Block>,

    /// Some blockchains may require additional transactions to be fetched that weren't returned in the block response (ex: block only returns transaction hashes). For blockchains with a lot of transactions in each block, this can be very useful as consumers can concurrently fetch all transactions returned.
    #[serde(rename = "other_transactions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_transactions: Option<Vec<models::TransactionIdentifier>>,
}

impl BlockResponse {
    pub fn new() -> BlockResponse {
        BlockResponse {
            block: None,
            other_transactions: None,
        }
    }
}

/// Converts the BlockResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for BlockResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping block in query parameter serialization

        // Skipping other_transactions in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a BlockResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for BlockResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub block: Vec<models::Block>,
            pub other_transactions: Vec<Vec<models::TransactionIdentifier>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing BlockResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "block" => intermediate_rep.block.push(
                        <models::Block as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "other_transactions" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in BlockResponse"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing BlockResponse".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(BlockResponse {
            block: intermediate_rep.block.into_iter().next(),
            other_transactions: intermediate_rep.other_transactions.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<BlockResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<BlockResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<BlockResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for BlockResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<BlockResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <BlockResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into BlockResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// BlockTransaction contains a populated Transaction and the BlockIdentifier that contains it.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct BlockTransaction {
    #[serde(rename = "block_identifier")]
    pub block_identifier: models::BlockIdentifier,

    #[serde(rename = "transaction")]
    pub transaction: models::Transaction,
}

impl BlockTransaction {
    pub fn new(
        block_identifier: models::BlockIdentifier,
        transaction: models::Transaction,
    ) -> BlockTransaction {
        BlockTransaction {
            block_identifier: block_identifier,
            transaction: transaction,
        }
    }
}

/// Converts the BlockTransaction value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for BlockTransaction {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping block_identifier in query parameter serialization

        // Skipping transaction in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a BlockTransaction value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for BlockTransaction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub block_identifier: Vec<models::BlockIdentifier>,
            pub transaction: Vec<models::Transaction>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing BlockTransaction".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "block_identifier" => intermediate_rep.block_identifier.push(
                        <models::BlockIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "transaction" => intermediate_rep.transaction.push(
                        <models::Transaction as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing BlockTransaction".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(BlockTransaction {
            block_identifier: intermediate_rep
                .block_identifier
                .into_iter()
                .next()
                .ok_or("block_identifier missing in BlockTransaction".to_string())?,
            transaction: intermediate_rep
                .transaction
                .into_iter()
                .next()
                .ok_or("transaction missing in BlockTransaction".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<BlockTransaction> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<BlockTransaction>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<BlockTransaction>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for BlockTransaction - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<BlockTransaction>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <BlockTransaction as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into BlockTransaction - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// A BlockTransactionRequest is used to fetch a Transaction included in a block that is not returned in a BlockResponse.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct BlockTransactionRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "block_identifier")]
    pub block_identifier: models::BlockIdentifier,

    #[serde(rename = "transaction_identifier")]
    pub transaction_identifier: models::TransactionIdentifier,
}

impl BlockTransactionRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        block_identifier: models::BlockIdentifier,
        transaction_identifier: models::TransactionIdentifier,
    ) -> BlockTransactionRequest {
        BlockTransactionRequest {
            network_identifier: network_identifier,
            block_identifier: block_identifier,
            transaction_identifier: transaction_identifier,
        }
    }
}

/// Converts the BlockTransactionRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for BlockTransactionRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping block_identifier in query parameter serialization

        // Skipping transaction_identifier in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a BlockTransactionRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for BlockTransactionRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub block_identifier: Vec<models::BlockIdentifier>,
            pub transaction_identifier: Vec<models::TransactionIdentifier>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing BlockTransactionRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "block_identifier" => intermediate_rep.block_identifier.push(
                        <models::BlockIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "transaction_identifier" => intermediate_rep.transaction_identifier.push(
                        <models::TransactionIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing BlockTransactionRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(BlockTransactionRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in BlockTransactionRequest".to_string())?,
            block_identifier: intermediate_rep
                .block_identifier
                .into_iter()
                .next()
                .ok_or("block_identifier missing in BlockTransactionRequest".to_string())?,
            transaction_identifier: intermediate_rep
                .transaction_identifier
                .into_iter()
                .next()
                .ok_or("transaction_identifier missing in BlockTransactionRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<BlockTransactionRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<BlockTransactionRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<BlockTransactionRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for BlockTransactionRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<BlockTransactionRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <BlockTransactionRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into BlockTransactionRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// A BlockTransactionResponse contains information about a block transaction.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct BlockTransactionResponse {
    #[serde(rename = "transaction")]
    pub transaction: models::Transaction,
}

impl BlockTransactionResponse {
    pub fn new(transaction: models::Transaction) -> BlockTransactionResponse {
        BlockTransactionResponse {
            transaction: transaction,
        }
    }
}

/// Converts the BlockTransactionResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for BlockTransactionResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping transaction in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a BlockTransactionResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for BlockTransactionResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub transaction: Vec<models::Transaction>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing BlockTransactionResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "transaction" => intermediate_rep.transaction.push(
                        <models::Transaction as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing BlockTransactionResponse".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(BlockTransactionResponse {
            transaction: intermediate_rep
                .transaction
                .into_iter()
                .next()
                .ok_or("transaction missing in BlockTransactionResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<BlockTransactionResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<BlockTransactionResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<BlockTransactionResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for BlockTransactionResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<BlockTransactionResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <BlockTransactionResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into BlockTransactionResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// CallRequest is the input to the `/call` endpoint.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CallRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// Method is some network-specific procedure call. This method could map to a network-specific RPC endpoint, a method in an SDK generated from a smart contract, or some hybrid of the two.  The implementation must define all available methods in the Allow object. However, it is up to the caller to determine which parameters to provide when invoking `/call`.
    #[serde(rename = "method")]
    pub method: String,

    /// Parameters is some network-specific argument for a method. It is up to the caller to determine which parameters to provide when invoking `/call`.
    #[serde(rename = "parameters")]
    pub parameters: serde_json::Value,
}

impl CallRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        method: String,
        parameters: serde_json::Value,
    ) -> CallRequest {
        CallRequest {
            network_identifier: network_identifier,
            method: method,
            parameters: parameters,
        }
    }
}

/// Converts the CallRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CallRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        params.push("method".to_string());
        params.push(self.method.to_string());

        // Skipping parameters in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CallRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CallRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub method: Vec<String>,
            pub parameters: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing CallRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "method" => intermediate_rep.method.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "parameters" => intermediate_rep.parameters.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing CallRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CallRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in CallRequest".to_string())?,
            method: intermediate_rep
                .method
                .into_iter()
                .next()
                .ok_or("method missing in CallRequest".to_string())?,
            parameters: intermediate_rep
                .parameters
                .into_iter()
                .next()
                .ok_or("parameters missing in CallRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CallRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<CallRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<CallRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for CallRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CallRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <CallRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into CallRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// CallResponse contains the result of a `/call` invocation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CallResponse {
    /// Result contains the result of the `/call` invocation. This result will not be inspected or interpreted by Rosetta tooling and is left to the caller to decode.
    #[serde(rename = "result")]
    pub result: serde_json::Value,

    /// Idempotent indicates that if `/call` is invoked with the same CallRequest again, at any point in time, it will return the same CallResponse.  Integrators may cache the CallResponse if this is set to true to avoid making unnecessary calls to the Rosetta implementation. For this reason, implementers should be very conservative about returning true here or they could cause issues for the caller.
    #[serde(rename = "idempotent")]
    pub idempotent: bool,
}

impl CallResponse {
    pub fn new(result: serde_json::Value, idempotent: bool) -> CallResponse {
        CallResponse {
            result: result,
            idempotent: idempotent,
        }
    }
}

/// Converts the CallResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CallResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping result in query parameter serialization

        params.push("idempotent".to_string());
        params.push(self.idempotent.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CallResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CallResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub result: Vec<serde_json::Value>,
            pub idempotent: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing CallResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "result" => intermediate_rep.result.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "idempotent" => intermediate_rep.idempotent.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing CallResponse".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CallResponse {
            result: intermediate_rep
                .result
                .into_iter()
                .next()
                .ok_or("result missing in CallResponse".to_string())?,
            idempotent: intermediate_rep
                .idempotent
                .into_iter()
                .next()
                .ok_or("idempotent missing in CallResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CallResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<CallResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<CallResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for CallResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CallResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <CallResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into CallResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Case specifies the expected case for strings and hashes.
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum Case {
    #[serde(rename = "upper_case")]
    UpperCase,
    #[serde(rename = "lower_case")]
    LowerCase,
    #[serde(rename = "case_sensitive")]
    CaseSensitive,
    #[serde(rename = "null")]
    Null,
}

impl std::fmt::Display for Case {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Case::UpperCase => write!(f, "{}", "upper_case"),
            Case::LowerCase => write!(f, "{}", "lower_case"),
            Case::CaseSensitive => write!(f, "{}", "case_sensitive"),
            Case::Null => write!(f, "{}", "null"),
        }
    }
}

impl std::str::FromStr for Case {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "upper_case" => std::result::Result::Ok(Case::UpperCase),
            "lower_case" => std::result::Result::Ok(Case::LowerCase),
            "case_sensitive" => std::result::Result::Ok(Case::CaseSensitive),
            "null" => std::result::Result::Ok(Case::Null),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

/// Coin contains its unique identifier and the amount it represents.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Coin {
    #[serde(rename = "coin_identifier")]
    pub coin_identifier: models::CoinIdentifier,

    #[serde(rename = "amount")]
    pub amount: models::Amount,
}

impl Coin {
    pub fn new(coin_identifier: models::CoinIdentifier, amount: models::Amount) -> Coin {
        Coin {
            coin_identifier: coin_identifier,
            amount: amount,
        }
    }
}

/// Converts the Coin value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Coin {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping coin_identifier in query parameter serialization

        // Skipping amount in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Coin value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Coin {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub coin_identifier: Vec<models::CoinIdentifier>,
            pub amount: Vec<models::Amount>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing Coin".to_string())
                }
            };

            if let Some(key) = key_result {
                match key {
                    "coin_identifier" => intermediate_rep.coin_identifier.push(
                        <models::CoinIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "amount" => intermediate_rep.amount.push(
                        <models::Amount as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Coin".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Coin {
            coin_identifier: intermediate_rep
                .coin_identifier
                .into_iter()
                .next()
                .ok_or("coin_identifier missing in Coin".to_string())?,
            amount: intermediate_rep
                .amount
                .into_iter()
                .next()
                .ok_or("amount missing in Coin".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Coin> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Coin>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Coin>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Coin - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Coin> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Coin as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => {
                    std::result::Result::Ok(header::IntoHeaderValue(value))
                }
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Coin - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// CoinActions are different state changes that a Coin can undergo. When a Coin is created, it is coin_created. When a Coin is spent, it is coin_spent. It is assumed that a single Coin cannot be created or spent more than once.
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum CoinAction {
    #[serde(rename = "coin_created")]
    Created,
    #[serde(rename = "coin_spent")]
    Spent,
}

impl std::fmt::Display for CoinAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CoinAction::Created => write!(f, "{}", "coin_created"),
            CoinAction::Spent => write!(f, "{}", "coin_spent"),
        }
    }
}

impl std::str::FromStr for CoinAction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "coin_created" => std::result::Result::Ok(CoinAction::Created),
            "coin_spent" => std::result::Result::Ok(CoinAction::Spent),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

/// CoinChange is used to represent a change in state of a some coin identified by a coin_identifier. This object is part of the Operation model and must be populated for UTXO-based blockchains.  Coincidentally, this abstraction of UTXOs allows for supporting both account-based transfers and UTXO-based transfers on the same blockchain (when a transfer is account-based, don't populate this model).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CoinChange {
    #[serde(rename = "coin_identifier")]
    pub coin_identifier: models::CoinIdentifier,

    #[serde(rename = "coin_action")]
    pub coin_action: models::CoinAction,
}

impl CoinChange {
    pub fn new(
        coin_identifier: models::CoinIdentifier,
        coin_action: models::CoinAction,
    ) -> CoinChange {
        CoinChange {
            coin_identifier: coin_identifier,
            coin_action: coin_action,
        }
    }
}

/// Converts the CoinChange value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CoinChange {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping coin_identifier in query parameter serialization

        // Skipping coin_action in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CoinChange value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CoinChange {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub coin_identifier: Vec<models::CoinIdentifier>,
            pub coin_action: Vec<models::CoinAction>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing CoinChange".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "coin_identifier" => intermediate_rep.coin_identifier.push(
                        <models::CoinIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "coin_action" => intermediate_rep.coin_action.push(
                        <models::CoinAction as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing CoinChange".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CoinChange {
            coin_identifier: intermediate_rep
                .coin_identifier
                .into_iter()
                .next()
                .ok_or("coin_identifier missing in CoinChange".to_string())?,
            coin_action: intermediate_rep
                .coin_action
                .into_iter()
                .next()
                .ok_or("coin_action missing in CoinChange".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CoinChange> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<CoinChange>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<CoinChange>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for CoinChange - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CoinChange> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <CoinChange as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into CoinChange - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// CoinIdentifier uniquely identifies a Coin.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CoinIdentifier {
    /// Identifier should be populated with a globally unique identifier of a Coin. In Bitcoin, this identifier would be transaction_hash:index.
    #[serde(rename = "identifier")]
    pub identifier: String,
}

impl CoinIdentifier {
    pub fn new(identifier: String) -> CoinIdentifier {
        CoinIdentifier {
            identifier: identifier,
        }
    }
}

/// Converts the CoinIdentifier value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CoinIdentifier {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("identifier".to_string());
        params.push(self.identifier.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CoinIdentifier value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CoinIdentifier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub identifier: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing CoinIdentifier".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "identifier" => intermediate_rep.identifier.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing CoinIdentifier".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CoinIdentifier {
            identifier: intermediate_rep
                .identifier
                .into_iter()
                .next()
                .ok_or("identifier missing in CoinIdentifier".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CoinIdentifier> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<CoinIdentifier>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<CoinIdentifier>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for CoinIdentifier - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CoinIdentifier> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <CoinIdentifier as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into CoinIdentifier - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// ConstructionCombineRequest is the input to the `/construction/combine` endpoint. It contains the unsigned transaction blob returned by `/construction/payloads` and all required signatures to create a network transaction.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionCombineRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "unsigned_transaction")]
    pub unsigned_transaction: String,

    #[serde(rename = "signatures")]
    pub signatures: Vec<models::Signature>,
}

impl ConstructionCombineRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        unsigned_transaction: String,
        signatures: Vec<models::Signature>,
    ) -> ConstructionCombineRequest {
        ConstructionCombineRequest {
            network_identifier: network_identifier,
            unsigned_transaction: unsigned_transaction,
            signatures: signatures,
        }
    }
}

/// Converts the ConstructionCombineRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionCombineRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        params.push("unsigned_transaction".to_string());
        params.push(self.unsigned_transaction.to_string());

        // Skipping signatures in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionCombineRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionCombineRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub unsigned_transaction: Vec<String>,
            pub signatures: Vec<Vec<models::Signature>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionCombineRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "unsigned_transaction" => intermediate_rep.unsigned_transaction.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "signatures" => return std::result::Result::Err("Parsing a container in this style is not supported in ConstructionCombineRequest".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing ConstructionCombineRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionCombineRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in ConstructionCombineRequest".to_string())?,
            unsigned_transaction: intermediate_rep
                .unsigned_transaction
                .into_iter()
                .next()
                .ok_or("unsigned_transaction missing in ConstructionCombineRequest".to_string())?,
            signatures: intermediate_rep
                .signatures
                .into_iter()
                .next()
                .ok_or("signatures missing in ConstructionCombineRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionCombineRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionCombineRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionCombineRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionCombineRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionCombineRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ConstructionCombineRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ConstructionCombineRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// ConstructionCombineResponse is returned by `/construction/combine`. The network payload will be sent directly to the `construction/submit` endpoint.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionCombineResponse {
    #[serde(rename = "signed_transaction")]
    pub signed_transaction: String,
}

impl ConstructionCombineResponse {
    pub fn new(signed_transaction: String) -> ConstructionCombineResponse {
        ConstructionCombineResponse {
            signed_transaction: signed_transaction,
        }
    }
}

/// Converts the ConstructionCombineResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionCombineResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("signed_transaction".to_string());
        params.push(self.signed_transaction.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionCombineResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionCombineResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub signed_transaction: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionCombineResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "signed_transaction" => intermediate_rep.signed_transaction.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ConstructionCombineResponse".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionCombineResponse {
            signed_transaction: intermediate_rep
                .signed_transaction
                .into_iter()
                .next()
                .ok_or("signed_transaction missing in ConstructionCombineResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionCombineResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionCombineResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionCombineResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionCombineResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionCombineResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ConstructionCombineResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ConstructionCombineResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// ConstructionDeriveRequest is passed to the `/construction/derive` endpoint. Network is provided in the request because some blockchains have different address formats for different networks. Metadata is provided in the request because some blockchains allow for multiple address types (i.e. different address for validators vs normal accounts).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionDeriveRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "public_key")]
    pub public_key: models::PublicKey,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl ConstructionDeriveRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        public_key: models::PublicKey,
    ) -> ConstructionDeriveRequest {
        ConstructionDeriveRequest {
            network_identifier: network_identifier,
            public_key: public_key,
            metadata: None,
        }
    }
}

/// Converts the ConstructionDeriveRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionDeriveRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping public_key in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionDeriveRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionDeriveRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub public_key: Vec<models::PublicKey>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionDeriveRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "public_key" => intermediate_rep.public_key.push(
                        <models::PublicKey as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ConstructionDeriveRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionDeriveRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in ConstructionDeriveRequest".to_string())?,
            public_key: intermediate_rep
                .public_key
                .into_iter()
                .next()
                .ok_or("public_key missing in ConstructionDeriveRequest".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionDeriveRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionDeriveRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionDeriveRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionDeriveRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionDeriveRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ConstructionDeriveRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ConstructionDeriveRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// ConstructionDeriveResponse is returned by the `/construction/derive` endpoint.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionDeriveResponse {
    /// [DEPRECATED by `account_identifier` in `v1.4.4`] Address in network-specific format.
    #[serde(rename = "address")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    #[serde(rename = "account_identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_identifier: Option<models::AccountIdentifier>,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl ConstructionDeriveResponse {
    pub fn new() -> ConstructionDeriveResponse {
        ConstructionDeriveResponse {
            address: None,
            account_identifier: None,
            metadata: None,
        }
    }
}

/// Converts the ConstructionDeriveResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionDeriveResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref address) = self.address {
            params.push("address".to_string());
            params.push(address.to_string());
        }

        // Skipping account_identifier in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionDeriveResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionDeriveResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub address: Vec<String>,
            pub account_identifier: Vec<models::AccountIdentifier>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionDeriveResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "address" => intermediate_rep.address.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "account_identifier" => intermediate_rep.account_identifier.push(
                        <models::AccountIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ConstructionDeriveResponse".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionDeriveResponse {
            address: intermediate_rep.address.into_iter().next(),
            account_identifier: intermediate_rep.account_identifier.into_iter().next(),
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionDeriveResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionDeriveResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionDeriveResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionDeriveResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionDeriveResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ConstructionDeriveResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ConstructionDeriveResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// ConstructionHashRequest is the input to the `/construction/hash` endpoint.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionHashRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "signed_transaction")]
    pub signed_transaction: String,
}

impl ConstructionHashRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        signed_transaction: String,
    ) -> ConstructionHashRequest {
        ConstructionHashRequest {
            network_identifier: network_identifier,
            signed_transaction: signed_transaction,
        }
    }
}

/// Converts the ConstructionHashRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionHashRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        params.push("signed_transaction".to_string());
        params.push(self.signed_transaction.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionHashRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionHashRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub signed_transaction: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionHashRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "signed_transaction" => intermediate_rep.signed_transaction.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ConstructionHashRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionHashRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in ConstructionHashRequest".to_string())?,
            signed_transaction: intermediate_rep
                .signed_transaction
                .into_iter()
                .next()
                .ok_or("signed_transaction missing in ConstructionHashRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionHashRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionHashRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionHashRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionHashRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionHashRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ConstructionHashRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ConstructionHashRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// A ConstructionMetadataRequest is utilized to get information required to construct a transaction.  The Options object used to specify which metadata to return is left purposely unstructured to allow flexibility for implementers. Options is not required in the case that there is network-wide metadata of interest.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionMetadataRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// Some blockchains require different metadata for different types of transaction construction (ex: delegation versus a transfer). Instead of requiring a blockchain node to return all possible types of metadata for construction (which may require multiple node fetches), the client can populate an options object to limit the metadata returned to only the subset required.
    #[serde(rename = "options")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
}

impl ConstructionMetadataRequest {
    pub fn new(network_identifier: models::NetworkIdentifier) -> ConstructionMetadataRequest {
        ConstructionMetadataRequest {
            network_identifier: network_identifier,
            options: None,
        }
    }
}

/// Converts the ConstructionMetadataRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionMetadataRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping options in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionMetadataRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionMetadataRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub options: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionMetadataRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "options" => intermediate_rep.options.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ConstructionMetadataRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionMetadataRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in ConstructionMetadataRequest".to_string())?,
            options: intermediate_rep.options.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionMetadataRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionMetadataRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionMetadataRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionMetadataRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionMetadataRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ConstructionMetadataRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ConstructionMetadataRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// The ConstructionMetadataResponse returns network-specific metadata used for transaction construction.  Optionally, the implementer can return the suggested fee associated with the transaction being constructed. The caller may use this info to adjust the intent of the transaction or to create a transaction with a different account that can pay the suggested fee. Suggested fee is an array in case fee payment must occur in multiple currencies.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionMetadataResponse {
    #[serde(rename = "metadata")]
    pub metadata: serde_json::Value,

    #[serde(rename = "suggested_fee")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_fee: Option<Vec<models::Amount>>,
}

impl ConstructionMetadataResponse {
    pub fn new(metadata: serde_json::Value) -> ConstructionMetadataResponse {
        ConstructionMetadataResponse {
            metadata: metadata,
            suggested_fee: None,
        }
    }
}

/// Converts the ConstructionMetadataResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionMetadataResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping metadata in query parameter serialization

        // Skipping suggested_fee in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionMetadataResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionMetadataResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub metadata: Vec<serde_json::Value>,
            pub suggested_fee: Vec<Vec<models::Amount>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionMetadataResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "metadata" => intermediate_rep.metadata.push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "suggested_fee" => return std::result::Result::Err("Parsing a container in this style is not supported in ConstructionMetadataResponse".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing ConstructionMetadataResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionMetadataResponse {
            metadata: intermediate_rep
                .metadata
                .into_iter()
                .next()
                .ok_or("metadata missing in ConstructionMetadataResponse".to_string())?,
            suggested_fee: intermediate_rep.suggested_fee.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionMetadataResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionMetadataResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionMetadataResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionMetadataResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionMetadataResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ConstructionMetadataResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into ConstructionMetadataResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

/// ConstructionParseRequest is the input to the `/construction/parse` endpoint. It allows the caller to parse either an unsigned or signed transaction.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionParseRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// Signed is a boolean indicating whether the transaction is signed.
    #[serde(rename = "signed")]
    pub signed: bool,

    /// This must be either the unsigned transaction blob returned by `/construction/payloads` or the signed transaction blob returned by `/construction/combine`.
    #[serde(rename = "transaction")]
    pub transaction: String,
}

impl ConstructionParseRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        signed: bool,
        transaction: String,
    ) -> ConstructionParseRequest {
        ConstructionParseRequest {
            network_identifier: network_identifier,
            signed: signed,
            transaction: transaction,
        }
    }
}

/// Converts the ConstructionParseRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionParseRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        params.push("signed".to_string());
        params.push(self.signed.to_string());

        params.push("transaction".to_string());
        params.push(self.transaction.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionParseRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionParseRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub signed: Vec<bool>,
            pub transaction: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionParseRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "signed" => intermediate_rep.signed.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "transaction" => intermediate_rep.transaction.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ConstructionParseRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionParseRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in ConstructionParseRequest".to_string())?,
            signed: intermediate_rep
                .signed
                .into_iter()
                .next()
                .ok_or("signed missing in ConstructionParseRequest".to_string())?,
            transaction: intermediate_rep
                .transaction
                .into_iter()
                .next()
                .ok_or("transaction missing in ConstructionParseRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionParseRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionParseRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionParseRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionParseRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionParseRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ConstructionParseRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ConstructionParseRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// ConstructionParseResponse contains an array of operations that occur in a transaction blob. This should match the array of operations provided to `/construction/preprocess` and `/construction/payloads`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionParseResponse {
    #[serde(rename = "operations")]
    pub operations: Vec<models::Operation>,

    /// [DEPRECATED by `account_identifier_signers` in `v1.4.4`] All signers (addresses) of a particular transaction. If the transaction is unsigned, it should be empty.
    #[serde(rename = "signers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signers: Option<Vec<String>>,

    #[serde(rename = "account_identifier_signers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_identifier_signers: Option<Vec<models::AccountIdentifier>>,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl ConstructionParseResponse {
    pub fn new(operations: Vec<models::Operation>) -> ConstructionParseResponse {
        ConstructionParseResponse {
            operations: operations,
            signers: None,
            account_identifier_signers: None,
            metadata: None,
        }
    }
}

/// Converts the ConstructionParseResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionParseResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping operations in query parameter serialization

        if let Some(ref signers) = self.signers {
            params.push("signers".to_string());
            params.push(
                signers
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_string(),
            );
        }

        // Skipping account_identifier_signers in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionParseResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionParseResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub operations: Vec<Vec<models::Operation>>,
            pub signers: Vec<Vec<String>>,
            pub account_identifier_signers: Vec<Vec<models::AccountIdentifier>>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionParseResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "operations" => return std::result::Result::Err("Parsing a container in this style is not supported in ConstructionParseResponse".to_string()),
                    "signers" => return std::result::Result::Err("Parsing a container in this style is not supported in ConstructionParseResponse".to_string()),
                    "account_identifier_signers" => return std::result::Result::Err("Parsing a container in this style is not supported in ConstructionParseResponse".to_string()),
                    "metadata" => intermediate_rep.metadata.push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ConstructionParseResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionParseResponse {
            operations: intermediate_rep
                .operations
                .into_iter()
                .next()
                .ok_or("operations missing in ConstructionParseResponse".to_string())?,
            signers: intermediate_rep.signers.into_iter().next(),
            account_identifier_signers: intermediate_rep
                .account_identifier_signers
                .into_iter()
                .next(),
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionParseResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionParseResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionParseResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionParseResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionParseResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ConstructionParseResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ConstructionParseResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// ConstructionPayloadsRequest is the request to `/construction/payloads`. It contains the network, a slice of operations, and arbitrary metadata that was returned by the call to `/construction/metadata`.  Optionally, the request can also include an array of PublicKeys associated with the AccountIdentifiers returned in ConstructionPreprocessResponse.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionPayloadsRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "operations")]
    pub operations: Vec<models::Operation>,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    #[serde(rename = "public_keys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_keys: Option<Vec<models::PublicKey>>,
}

impl ConstructionPayloadsRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        operations: Vec<models::Operation>,
    ) -> ConstructionPayloadsRequest {
        ConstructionPayloadsRequest {
            network_identifier: network_identifier,
            operations: operations,
            metadata: None,
            public_keys: None,
        }
    }
}

/// Converts the ConstructionPayloadsRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionPayloadsRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping operations in query parameter serialization

        // Skipping metadata in query parameter serialization

        // Skipping public_keys in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionPayloadsRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionPayloadsRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub operations: Vec<Vec<models::Operation>>,
            pub metadata: Vec<serde_json::Value>,
            pub public_keys: Vec<Vec<models::PublicKey>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionPayloadsRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "operations" => return std::result::Result::Err("Parsing a container in this style is not supported in ConstructionPayloadsRequest".to_string()),
                    "metadata" => intermediate_rep.metadata.push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "public_keys" => return std::result::Result::Err("Parsing a container in this style is not supported in ConstructionPayloadsRequest".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing ConstructionPayloadsRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionPayloadsRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in ConstructionPayloadsRequest".to_string())?,
            operations: intermediate_rep
                .operations
                .into_iter()
                .next()
                .ok_or("operations missing in ConstructionPayloadsRequest".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
            public_keys: intermediate_rep.public_keys.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionPayloadsRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionPayloadsRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionPayloadsRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionPayloadsRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionPayloadsRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ConstructionPayloadsRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ConstructionPayloadsRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// ConstructionTransactionResponse is returned by `/construction/payloads`. It contains an unsigned transaction blob (that is usually needed to construct the a network transaction from a collection of signatures) and an array of payloads that must be signed by the caller.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionPayloadsResponse {
    #[serde(rename = "unsigned_transaction")]
    pub unsigned_transaction: String,

    #[serde(rename = "payloads")]
    pub payloads: Vec<models::SigningPayload>,
}

impl ConstructionPayloadsResponse {
    pub fn new(
        unsigned_transaction: String,
        payloads: Vec<models::SigningPayload>,
    ) -> ConstructionPayloadsResponse {
        ConstructionPayloadsResponse {
            unsigned_transaction: unsigned_transaction,
            payloads: payloads,
        }
    }
}

/// Converts the ConstructionPayloadsResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionPayloadsResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("unsigned_transaction".to_string());
        params.push(self.unsigned_transaction.to_string());

        // Skipping payloads in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionPayloadsResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionPayloadsResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub unsigned_transaction: Vec<String>,
            pub payloads: Vec<Vec<models::SigningPayload>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionPayloadsResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "unsigned_transaction" => intermediate_rep.unsigned_transaction.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "payloads" => return std::result::Result::Err("Parsing a container in this style is not supported in ConstructionPayloadsResponse".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing ConstructionPayloadsResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionPayloadsResponse {
            unsigned_transaction: intermediate_rep
                .unsigned_transaction
                .into_iter()
                .next()
                .ok_or(
                    "unsigned_transaction missing in ConstructionPayloadsResponse".to_string(),
                )?,
            payloads: intermediate_rep
                .payloads
                .into_iter()
                .next()
                .ok_or("payloads missing in ConstructionPayloadsResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionPayloadsResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionPayloadsResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionPayloadsResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionPayloadsResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionPayloadsResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ConstructionPayloadsResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into ConstructionPayloadsResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

/// ConstructionPreprocessRequest is passed to the `/construction/preprocess` endpoint so that a Rosetta implementation can determine which metadata it needs to request for construction.  Metadata provided in this object should NEVER be a product of live data (i.e. the caller must follow some network-specific data fetching strategy outside of the Construction API to populate required Metadata). If live data is required for construction, it MUST be fetched in the call to `/construction/metadata`.  The caller can provide a max fee they are willing to pay for a transaction. This is an array in the case fees must be paid in multiple currencies.  The caller can also provide a suggested fee multiplier to indicate that the suggested fee should be scaled. This may be used to set higher fees for urgent transactions or to pay lower fees when there is less urgency. It is assumed that providing a very low multiplier (like 0.0001) will never lead to a transaction being created with a fee less than the minimum network fee (if applicable).  In the case that the caller provides both a max fee and a suggested fee multiplier, the max fee will set an upper bound on the suggested fee (regardless of the multiplier provided).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionPreprocessRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "operations")]
    pub operations: Vec<models::Operation>,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    #[serde(rename = "max_fee")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee: Option<Vec<models::Amount>>,

    #[serde(rename = "suggested_fee_multiplier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_fee_multiplier: Option<f64>,
}

impl ConstructionPreprocessRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        operations: Vec<models::Operation>,
    ) -> ConstructionPreprocessRequest {
        ConstructionPreprocessRequest {
            network_identifier: network_identifier,
            operations: operations,
            metadata: None,
            max_fee: None,
            suggested_fee_multiplier: None,
        }
    }
}

/// Converts the ConstructionPreprocessRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionPreprocessRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping operations in query parameter serialization

        // Skipping metadata in query parameter serialization

        // Skipping max_fee in query parameter serialization

        if let Some(ref suggested_fee_multiplier) = self.suggested_fee_multiplier {
            params.push("suggested_fee_multiplier".to_string());
            params.push(suggested_fee_multiplier.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionPreprocessRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionPreprocessRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub operations: Vec<Vec<models::Operation>>,
            pub metadata: Vec<serde_json::Value>,
            pub max_fee: Vec<Vec<models::Amount>>,
            pub suggested_fee_multiplier: Vec<f64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionPreprocessRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "operations" => return std::result::Result::Err("Parsing a container in this style is not supported in ConstructionPreprocessRequest".to_string()),
                    "metadata" => intermediate_rep.metadata.push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "max_fee" => return std::result::Result::Err("Parsing a container in this style is not supported in ConstructionPreprocessRequest".to_string()),
                    "suggested_fee_multiplier" => intermediate_rep.suggested_fee_multiplier.push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ConstructionPreprocessRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionPreprocessRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in ConstructionPreprocessRequest".to_string())?,
            operations: intermediate_rep
                .operations
                .into_iter()
                .next()
                .ok_or("operations missing in ConstructionPreprocessRequest".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
            max_fee: intermediate_rep.max_fee.into_iter().next(),
            suggested_fee_multiplier: intermediate_rep.suggested_fee_multiplier.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionPreprocessRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionPreprocessRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionPreprocessRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionPreprocessRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionPreprocessRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ConstructionPreprocessRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into ConstructionPreprocessRequest - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

/// ConstructionPreprocessResponse contains `options` that will be sent unmodified to `/construction/metadata`. If it is not necessary to make a request to `/construction/metadata`, `options` should be omitted.   Some blockchains require the PublicKey of particular AccountIdentifiers to construct a valid transaction. To fetch these PublicKeys, populate `required_public_keys` with the AccountIdentifiers associated with the desired PublicKeys. If it is not necessary to retrieve any PublicKeys for construction, `required_public_keys` should be omitted.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionPreprocessResponse {
    /// The options that will be sent directly to `/construction/metadata` by the caller.
    #[serde(rename = "options")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,

    #[serde(rename = "required_public_keys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_public_keys: Option<Vec<models::AccountIdentifier>>,
}

impl ConstructionPreprocessResponse {
    pub fn new() -> ConstructionPreprocessResponse {
        ConstructionPreprocessResponse {
            options: None,
            required_public_keys: None,
        }
    }
}

/// Converts the ConstructionPreprocessResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionPreprocessResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping options in query parameter serialization

        // Skipping required_public_keys in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionPreprocessResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionPreprocessResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub options: Vec<serde_json::Value>,
            pub required_public_keys: Vec<Vec<models::AccountIdentifier>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionPreprocessResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "options" => intermediate_rep.options.push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "required_public_keys" => return std::result::Result::Err("Parsing a container in this style is not supported in ConstructionPreprocessResponse".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing ConstructionPreprocessResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionPreprocessResponse {
            options: intermediate_rep.options.into_iter().next(),
            required_public_keys: intermediate_rep.required_public_keys.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionPreprocessResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionPreprocessResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionPreprocessResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionPreprocessResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionPreprocessResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ConstructionPreprocessResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into ConstructionPreprocessResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

/// The transaction submission request includes a signed transaction.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ConstructionSubmitRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "signed_transaction")]
    pub signed_transaction: String,
}

impl ConstructionSubmitRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        signed_transaction: String,
    ) -> ConstructionSubmitRequest {
        ConstructionSubmitRequest {
            network_identifier: network_identifier,
            signed_transaction: signed_transaction,
        }
    }
}

/// Converts the ConstructionSubmitRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConstructionSubmitRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        params.push("signed_transaction".to_string());
        params.push(self.signed_transaction.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConstructionSubmitRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConstructionSubmitRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub signed_transaction: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConstructionSubmitRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "signed_transaction" => intermediate_rep.signed_transaction.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ConstructionSubmitRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConstructionSubmitRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in ConstructionSubmitRequest".to_string())?,
            signed_transaction: intermediate_rep
                .signed_transaction
                .into_iter()
                .next()
                .ok_or("signed_transaction missing in ConstructionSubmitRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConstructionSubmitRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConstructionSubmitRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConstructionSubmitRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConstructionSubmitRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ConstructionSubmitRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ConstructionSubmitRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ConstructionSubmitRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Currency is composed of a canonical Symbol and Decimals. This Decimals value is used to convert an Amount.Value from atomic units (Satoshis) to standard units (Bitcoins).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Currency {
    /// Canonical symbol associated with a currency.
    #[serde(rename = "symbol")]
    pub symbol: String,

    /// Number of decimal places in the standard unit representation of the amount.  For example, BTC has 8 decimals. Note that it is not possible to represent the value of some currency in atomic units that is not base 10.
    #[serde(rename = "decimals")]
    pub decimals: u32,

    /// Any additional information related to the currency itself.  For example, it would be useful to populate this object with the contract address of an ERC-20 token.
    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Currency {
    pub fn new(symbol: String, decimals: u32) -> Currency {
        Currency {
            symbol: symbol,
            decimals: decimals,
            metadata: None,
        }
    }
}

/// Converts the Currency value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Currency {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("symbol".to_string());
        params.push(self.symbol.to_string());

        params.push("decimals".to_string());
        params.push(self.decimals.to_string());

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Currency value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub symbol: Vec<String>,
            pub decimals: Vec<u32>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Currency".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "symbol" => intermediate_rep.symbol.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "decimals" => intermediate_rep.decimals.push(
                        <u32 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Currency".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Currency {
            symbol: intermediate_rep
                .symbol
                .into_iter()
                .next()
                .ok_or("symbol missing in Currency".to_string())?,
            decimals: intermediate_rep
                .decimals
                .into_iter()
                .next()
                .ok_or("decimals missing in Currency".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Currency> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Currency>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Currency>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Currency - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Currency> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Currency as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Currency - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// CurveType is the type of cryptographic curve associated with a PublicKey.  * secp256k1: SEC compressed - `33 bytes` (https://secg.org/sec1-v2.pdf#subsubsection.2.3.3) * secp256r1: SEC compressed - `33 bytes` (https://secg.org/sec1-v2.pdf#subsubsection.2.3.3) * edwards25519: `y (255-bits) || x-sign-bit (1-bit)` - `32 bytes` (https://ed25519.cr.yp.to/ed25519-20110926.pdf) * tweedle: 1st pk : Fq.t (32 bytes) || 2nd pk : Fq.t (32 bytes) (https://github.com/CodaProtocol/coda/blob/develop/rfcs/0038-rosetta-construction-api.md#marshal-keys) * pallas: `x (255 bits) || y-parity-bit (1-bit) - 32 bytes` (https://github.com/zcash/pasta)
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum CurveType {
    #[serde(rename = "secp256k1")]
    Secp256k1,
    #[serde(rename = "secp256r1")]
    Secp256r1,
    #[serde(rename = "edwards25519")]
    Edwards25519,
    #[serde(rename = "tweedle")]
    Tweedle,
    #[serde(rename = "pallas")]
    Pallas,
}

impl std::fmt::Display for CurveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CurveType::Secp256k1 => write!(f, "{}", "secp256k1"),
            CurveType::Secp256r1 => write!(f, "{}", "secp256r1"),
            CurveType::Edwards25519 => write!(f, "{}", "edwards25519"),
            CurveType::Tweedle => write!(f, "{}", "tweedle"),
            CurveType::Pallas => write!(f, "{}", "pallas"),
        }
    }
}

impl std::str::FromStr for CurveType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "secp256k1" => std::result::Result::Ok(CurveType::Secp256k1),
            "secp256r1" => std::result::Result::Ok(CurveType::Secp256r1),
            "edwards25519" => std::result::Result::Ok(CurveType::Edwards25519),
            "tweedle" => std::result::Result::Ok(CurveType::Tweedle),
            "pallas" => std::result::Result::Ok(CurveType::Pallas),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

/// Used by RelatedTransaction to indicate the direction of the relation (i.e. cross-shard/cross-network sends may reference `backward` to an earlier transaction and async execution may reference `forward`). Can be used to indicate if a transaction relation is from child to parent or the reverse.
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum Direction {
    #[serde(rename = "forward")]
    Forward,
    #[serde(rename = "backward")]
    Backward,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Direction::Forward => write!(f, "{}", "forward"),
            Direction::Backward => write!(f, "{}", "backward"),
        }
    }
}

impl std::str::FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "forward" => std::result::Result::Ok(Direction::Forward),
            "backward" => std::result::Result::Ok(Direction::Backward),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

/// Instead of utilizing HTTP status codes to describe node errors (which often do not have a good analog), rich errors are returned using this object.  Both the code and message fields can be individually used to correctly identify an error. Implementations MUST use unique values for both fields.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Error {
    /// Code is a network-specific error code. If desired, this code can be equivalent to an HTTP status code.
    #[serde(rename = "code")]
    pub code: u32,

    /// Message is a network-specific error message.  The message MUST NOT change for a given code. In particular, this means that any contextual information should be included in the details field.
    #[serde(rename = "message")]
    pub message: String,

    /// Description allows the implementer to optionally provide additional information about an error. In many cases, the content of this field will be a copy-and-paste from existing developer documentation.  Description can ONLY be populated with generic information about a particular type of error. It MUST NOT be populated with information about a particular instantiation of an error (use `details` for this).  Whereas the content of Error.Message should stay stable across releases, the content of Error.Description will likely change across releases (as implementers improve error documentation). For this reason, the content in this field is not part of any type assertion (unlike Error.Message).
    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// An error is retriable if the same request may succeed if submitted again.
    #[serde(rename = "retriable")]
    pub retriable: bool,

    /// Often times it is useful to return context specific to the request that caused the error (i.e. a sample of the stack trace or impacted account) in addition to the standard error message.
    #[serde(rename = "details")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl Error {
    pub fn new(code: u32, message: String, retriable: bool) -> Error {
        Error {
            code: code,
            message: message,
            description: None,
            retriable: retriable,
            details: None,
        }
    }
}

/// Converts the Error value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Error {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("code".to_string());
        params.push(self.code.to_string());

        params.push("message".to_string());
        params.push(self.message.to_string());

        if let Some(ref description) = self.description {
            params.push("description".to_string());
            params.push(description.to_string());
        }

        params.push("retriable".to_string());
        params.push(self.retriable.to_string());

        // Skipping details in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Error value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Error {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub code: Vec<u32>,
            pub message: Vec<String>,
            pub description: Vec<String>,
            pub retriable: Vec<bool>,
            pub details: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Error".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "code" => intermediate_rep.code.push(
                        <u32 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "message" => intermediate_rep.message.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "description" => intermediate_rep.description.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "retriable" => intermediate_rep.retriable.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "details" => intermediate_rep.details.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Error".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Error {
            code: intermediate_rep
                .code
                .into_iter()
                .next()
                .ok_or("code missing in Error".to_string())?,
            message: intermediate_rep
                .message
                .into_iter()
                .next()
                .ok_or("message missing in Error".to_string())?,
            description: intermediate_rep.description.into_iter().next(),
            retriable: intermediate_rep
                .retriable
                .into_iter()
                .next()
                .ok_or("retriable missing in Error".to_string())?,
            details: intermediate_rep.details.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Error> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Error>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Error>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Error - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Error> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Error as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => {
                    std::result::Result::Ok(header::IntoHeaderValue(value))
                }
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Error - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// EventsBlocksRequest is utilized to fetch a sequence of BlockEvents indicating which blocks were added and removed from storage to reach the current state.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct EventsBlocksRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// offset is the offset into the event stream to sync events from. If this field is not populated, we return the limit events backwards from tip. If this is set to 0, we start from the beginning.
    #[serde(rename = "offset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,

    /// limit is the maximum number of events to fetch in one call. The implementation may return <= limit events.
    #[serde(rename = "limit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl EventsBlocksRequest {
    pub fn new(network_identifier: models::NetworkIdentifier) -> EventsBlocksRequest {
        EventsBlocksRequest {
            network_identifier: network_identifier,
            offset: None,
            limit: None,
        }
    }
}

/// Converts the EventsBlocksRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for EventsBlocksRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        if let Some(ref offset) = self.offset {
            params.push("offset".to_string());
            params.push(offset.to_string());
        }

        if let Some(ref limit) = self.limit {
            params.push("limit".to_string());
            params.push(limit.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a EventsBlocksRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for EventsBlocksRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub offset: Vec<i64>,
            pub limit: Vec<i64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing EventsBlocksRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "offset" => intermediate_rep.offset.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "limit" => intermediate_rep.limit.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing EventsBlocksRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(EventsBlocksRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in EventsBlocksRequest".to_string())?,
            offset: intermediate_rep.offset.into_iter().next(),
            limit: intermediate_rep.limit.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<EventsBlocksRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<EventsBlocksRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<EventsBlocksRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for EventsBlocksRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<EventsBlocksRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <EventsBlocksRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into EventsBlocksRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// EventsBlocksResponse contains an ordered collection of BlockEvents and the max retrievable sequence.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct EventsBlocksResponse {
    /// max_sequence is the maximum available sequence number to fetch.
    #[serde(rename = "max_sequence")]
    pub max_sequence: i64,

    /// events is an array of BlockEvents indicating the order to add and remove blocks to maintain a canonical view of blockchain state. Lightweight clients can use this event stream to update state without implementing their own block syncing logic.
    #[serde(rename = "events")]
    pub events: Vec<models::BlockEvent>,
}

impl EventsBlocksResponse {
    pub fn new(max_sequence: i64, events: Vec<models::BlockEvent>) -> EventsBlocksResponse {
        EventsBlocksResponse {
            max_sequence: max_sequence,
            events: events,
        }
    }
}

/// Converts the EventsBlocksResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for EventsBlocksResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("max_sequence".to_string());
        params.push(self.max_sequence.to_string());

        // Skipping events in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a EventsBlocksResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for EventsBlocksResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub max_sequence: Vec<i64>,
            pub events: Vec<Vec<models::BlockEvent>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing EventsBlocksResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "max_sequence" => intermediate_rep.max_sequence.push(<i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "events" => return std::result::Result::Err("Parsing a container in this style is not supported in EventsBlocksResponse".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing EventsBlocksResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(EventsBlocksResponse {
            max_sequence: intermediate_rep
                .max_sequence
                .into_iter()
                .next()
                .ok_or("max_sequence missing in EventsBlocksResponse".to_string())?,
            events: intermediate_rep
                .events
                .into_iter()
                .next()
                .ok_or("events missing in EventsBlocksResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<EventsBlocksResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<EventsBlocksResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<EventsBlocksResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for EventsBlocksResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<EventsBlocksResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <EventsBlocksResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into EventsBlocksResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// ExemptionType is used to indicate if the live balance for an account subject to a BalanceExemption could increase above, decrease below, or equal the computed balance.  * greater_or_equal: The live balance may increase above or equal the computed balance. This typically   occurs with staking rewards that accrue on each block. * less_or_equal: The live balance may decrease below or equal the computed balance. This typically   occurs as balance moves from locked to spendable on a vesting account. * dynamic: The live balance may increase above, decrease below, or equal the computed balance. This   typically occurs with tokens that have a dynamic supply.
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum ExemptionType {
    #[serde(rename = "greater_or_equal")]
    GreaterOrEqual,
    #[serde(rename = "less_or_equal")]
    LessOrEqual,
    #[serde(rename = "dynamic")]
    Dynamic,
}

impl std::fmt::Display for ExemptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ExemptionType::GreaterOrEqual => write!(f, "{}", "greater_or_equal"),
            ExemptionType::LessOrEqual => write!(f, "{}", "less_or_equal"),
            ExemptionType::Dynamic => write!(f, "{}", "dynamic"),
        }
    }
}

impl std::str::FromStr for ExemptionType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "greater_or_equal" => std::result::Result::Ok(ExemptionType::GreaterOrEqual),
            "less_or_equal" => std::result::Result::Ok(ExemptionType::LessOrEqual),
            "dynamic" => std::result::Result::Ok(ExemptionType::Dynamic),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

/// A MempoolResponse contains all transaction identifiers in the mempool for a particular network_identifier.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MempoolResponse {
    #[serde(rename = "transaction_identifiers")]
    pub transaction_identifiers: Vec<models::TransactionIdentifier>,
}

impl MempoolResponse {
    pub fn new(transaction_identifiers: Vec<models::TransactionIdentifier>) -> MempoolResponse {
        MempoolResponse {
            transaction_identifiers: transaction_identifiers,
        }
    }
}

/// Converts the MempoolResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for MempoolResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping transaction_identifiers in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MempoolResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MempoolResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub transaction_identifiers: Vec<Vec<models::TransactionIdentifier>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing MempoolResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "transaction_identifiers" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in MempoolResponse"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing MempoolResponse".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MempoolResponse {
            transaction_identifiers: intermediate_rep
                .transaction_identifiers
                .into_iter()
                .next()
                .ok_or("transaction_identifiers missing in MempoolResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MempoolResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<MempoolResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<MempoolResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for MempoolResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<MempoolResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <MempoolResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into MempoolResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// A MempoolTransactionRequest is utilized to retrieve a transaction from the mempool.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MempoolTransactionRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "transaction_identifier")]
    pub transaction_identifier: models::TransactionIdentifier,
}

impl MempoolTransactionRequest {
    pub fn new(
        network_identifier: models::NetworkIdentifier,
        transaction_identifier: models::TransactionIdentifier,
    ) -> MempoolTransactionRequest {
        MempoolTransactionRequest {
            network_identifier: network_identifier,
            transaction_identifier: transaction_identifier,
        }
    }
}

/// Converts the MempoolTransactionRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for MempoolTransactionRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping transaction_identifier in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MempoolTransactionRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MempoolTransactionRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub transaction_identifier: Vec<models::TransactionIdentifier>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing MempoolTransactionRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "transaction_identifier" => intermediate_rep.transaction_identifier.push(
                        <models::TransactionIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing MempoolTransactionRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MempoolTransactionRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in MempoolTransactionRequest".to_string())?,
            transaction_identifier: intermediate_rep
                .transaction_identifier
                .into_iter()
                .next()
                .ok_or("transaction_identifier missing in MempoolTransactionRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MempoolTransactionRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<MempoolTransactionRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<MempoolTransactionRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for MempoolTransactionRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<MempoolTransactionRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <MempoolTransactionRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into MempoolTransactionRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// A MempoolTransactionResponse contains an estimate of a mempool transaction. It may not be possible to know the full impact of a transaction in the mempool (ex: fee paid).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MempoolTransactionResponse {
    #[serde(rename = "transaction")]
    pub transaction: models::Transaction,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl MempoolTransactionResponse {
    pub fn new(transaction: models::Transaction) -> MempoolTransactionResponse {
        MempoolTransactionResponse {
            transaction: transaction,
            metadata: None,
        }
    }
}

/// Converts the MempoolTransactionResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for MempoolTransactionResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping transaction in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MempoolTransactionResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MempoolTransactionResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub transaction: Vec<models::Transaction>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing MempoolTransactionResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "transaction" => intermediate_rep.transaction.push(
                        <models::Transaction as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing MempoolTransactionResponse".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MempoolTransactionResponse {
            transaction: intermediate_rep
                .transaction
                .into_iter()
                .next()
                .ok_or("transaction missing in MempoolTransactionResponse".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MempoolTransactionResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<MempoolTransactionResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<MempoolTransactionResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for MempoolTransactionResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<MempoolTransactionResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <MempoolTransactionResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into MempoolTransactionResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// A MetadataRequest is utilized in any request where the only argument is optional metadata.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MetadataRequest {
    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl MetadataRequest {
    pub fn new() -> MetadataRequest {
        MetadataRequest { metadata: None }
    }
}

/// Converts the MetadataRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for MetadataRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MetadataRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MetadataRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing MetadataRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing MetadataRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MetadataRequest {
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MetadataRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<MetadataRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<MetadataRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for MetadataRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<MetadataRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <MetadataRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into MetadataRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// The network_identifier specifies which network a particular object is associated with.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkIdentifier {
    #[serde(rename = "blockchain")]
    pub blockchain: String,

    /// If a blockchain has a specific chain-id or network identifier, it should go in this field. It is up to the client to determine which network-specific identifier is mainnet or testnet.
    #[serde(rename = "network")]
    pub network: String,

    #[serde(rename = "sub_network_identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_network_identifier: Option<models::SubNetworkIdentifier>,
}

impl NetworkIdentifier {
    pub fn new(blockchain: String, network: String) -> NetworkIdentifier {
        NetworkIdentifier {
            blockchain: blockchain,
            network: network,
            sub_network_identifier: None,
        }
    }
}

/// Converts the NetworkIdentifier value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkIdentifier {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("blockchain".to_string());
        params.push(self.blockchain.to_string());

        params.push("network".to_string());
        params.push(self.network.to_string());

        // Skipping sub_network_identifier in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkIdentifier value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkIdentifier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub blockchain: Vec<String>,
            pub network: Vec<String>,
            pub sub_network_identifier: Vec<models::SubNetworkIdentifier>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing NetworkIdentifier".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "blockchain" => intermediate_rep.blockchain.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "network" => intermediate_rep.network.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "sub_network_identifier" => intermediate_rep.sub_network_identifier.push(
                        <models::SubNetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing NetworkIdentifier".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkIdentifier {
            blockchain: intermediate_rep
                .blockchain
                .into_iter()
                .next()
                .ok_or("blockchain missing in NetworkIdentifier".to_string())?,
            network: intermediate_rep
                .network
                .into_iter()
                .next()
                .ok_or("network missing in NetworkIdentifier".to_string())?,
            sub_network_identifier: intermediate_rep.sub_network_identifier.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkIdentifier> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<NetworkIdentifier>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<NetworkIdentifier>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for NetworkIdentifier - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<NetworkIdentifier>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <NetworkIdentifier as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into NetworkIdentifier - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// A NetworkListResponse contains all NetworkIdentifiers that the node can serve information for.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkListResponse {
    #[serde(rename = "network_identifiers")]
    pub network_identifiers: Vec<models::NetworkIdentifier>,
}

impl NetworkListResponse {
    pub fn new(network_identifiers: Vec<models::NetworkIdentifier>) -> NetworkListResponse {
        NetworkListResponse {
            network_identifiers: network_identifiers,
        }
    }
}

/// Converts the NetworkListResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkListResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifiers in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkListResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkListResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifiers: Vec<Vec<models::NetworkIdentifier>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing NetworkListResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifiers" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in NetworkListResponse"
                            .to_string(),
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing NetworkListResponse".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkListResponse {
            network_identifiers: intermediate_rep
                .network_identifiers
                .into_iter()
                .next()
                .ok_or("network_identifiers missing in NetworkListResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkListResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<NetworkListResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<NetworkListResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for NetworkListResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<NetworkListResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <NetworkListResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into NetworkListResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// NetworkOptionsResponse contains information about the versioning of the node and the allowed operation statuses, operation types, and errors.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkOptionsResponse {
    #[serde(rename = "version")]
    pub version: models::Version,

    #[serde(rename = "allow")]
    pub allow: models::Allow,
}

impl NetworkOptionsResponse {
    pub fn new(version: models::Version, allow: models::Allow) -> NetworkOptionsResponse {
        NetworkOptionsResponse {
            version: version,
            allow: allow,
        }
    }
}

/// Converts the NetworkOptionsResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkOptionsResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping version in query parameter serialization

        // Skipping allow in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkOptionsResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkOptionsResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub version: Vec<models::Version>,
            pub allow: Vec<models::Allow>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing NetworkOptionsResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "version" => intermediate_rep.version.push(
                        <models::Version as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "allow" => intermediate_rep.allow.push(
                        <models::Allow as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing NetworkOptionsResponse".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkOptionsResponse {
            version: intermediate_rep
                .version
                .into_iter()
                .next()
                .ok_or("version missing in NetworkOptionsResponse".to_string())?,
            allow: intermediate_rep
                .allow
                .into_iter()
                .next()
                .ok_or("allow missing in NetworkOptionsResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkOptionsResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<NetworkOptionsResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<NetworkOptionsResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for NetworkOptionsResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<NetworkOptionsResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <NetworkOptionsResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into NetworkOptionsResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// A NetworkRequest is utilized to retrieve some data specific exclusively to a NetworkIdentifier.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl NetworkRequest {
    pub fn new(network_identifier: models::NetworkIdentifier) -> NetworkRequest {
        NetworkRequest {
            network_identifier: network_identifier,
            metadata: None,
        }
    }
}

/// Converts the NetworkRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing NetworkRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing NetworkRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in NetworkRequest".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<NetworkRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<NetworkRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for NetworkRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <NetworkRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into NetworkRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// NetworkStatusResponse contains basic information about the node's view of a blockchain network. It is assumed that any BlockIdentifier.Index less than or equal to CurrentBlockIdentifier.Index can be queried.  If a Rosetta implementation prunes historical state, it should populate the optional `oldest_block_identifier` field with the oldest block available to query. If this is not populated, it is assumed that the `genesis_block_identifier` is the oldest queryable block.  If a Rosetta implementation performs some pre-sync before it is possible to query blocks, sync_status should be populated so that clients can still monitor healthiness. Without this field, it may appear that the implementation is stuck syncing and needs to be terminated.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkStatusResponse {
    #[serde(rename = "current_block_identifier")]
    pub current_block_identifier: models::BlockIdentifier,

    /// The timestamp of the block in milliseconds since the Unix Epoch. The timestamp is stored in milliseconds because some blockchains produce blocks more often than once a second.
    #[serde(rename = "current_block_timestamp")]
    pub current_block_timestamp: i64,

    #[serde(rename = "genesis_block_identifier")]
    pub genesis_block_identifier: models::BlockIdentifier,

    #[serde(rename = "oldest_block_identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_block_identifier: Option<models::BlockIdentifier>,

    #[serde(rename = "sync_status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_status: Option<models::SyncStatus>,

    #[serde(rename = "peers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peers: Option<Vec<models::Peer>>,
}

impl NetworkStatusResponse {
    pub fn new(
        current_block_identifier: models::BlockIdentifier,
        current_block_timestamp: i64,
        genesis_block_identifier: models::BlockIdentifier,
    ) -> NetworkStatusResponse {
        NetworkStatusResponse {
            current_block_identifier: current_block_identifier,
            current_block_timestamp: current_block_timestamp,
            genesis_block_identifier: genesis_block_identifier,
            oldest_block_identifier: None,
            sync_status: None,
            peers: None,
        }
    }
}

/// Converts the NetworkStatusResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkStatusResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping current_block_identifier in query parameter serialization

        params.push("current_block_timestamp".to_string());
        params.push(self.current_block_timestamp.to_string());

        // Skipping genesis_block_identifier in query parameter serialization

        // Skipping oldest_block_identifier in query parameter serialization

        // Skipping sync_status in query parameter serialization

        // Skipping peers in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkStatusResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkStatusResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub current_block_identifier: Vec<models::BlockIdentifier>,
            pub current_block_timestamp: Vec<i64>,
            pub genesis_block_identifier: Vec<models::BlockIdentifier>,
            pub oldest_block_identifier: Vec<models::BlockIdentifier>,
            pub sync_status: Vec<models::SyncStatus>,
            pub peers: Vec<Vec<models::Peer>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing NetworkStatusResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "current_block_identifier" => intermediate_rep.current_block_identifier.push(<models::BlockIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "current_block_timestamp" => intermediate_rep.current_block_timestamp.push(<i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "genesis_block_identifier" => intermediate_rep.genesis_block_identifier.push(<models::BlockIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "oldest_block_identifier" => intermediate_rep.oldest_block_identifier.push(<models::BlockIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "sync_status" => intermediate_rep.sync_status.push(<models::SyncStatus as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "peers" => return std::result::Result::Err("Parsing a container in this style is not supported in NetworkStatusResponse".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkStatusResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkStatusResponse {
            current_block_identifier: intermediate_rep
                .current_block_identifier
                .into_iter()
                .next()
                .ok_or("current_block_identifier missing in NetworkStatusResponse".to_string())?,
            current_block_timestamp: intermediate_rep
                .current_block_timestamp
                .into_iter()
                .next()
                .ok_or("current_block_timestamp missing in NetworkStatusResponse".to_string())?,
            genesis_block_identifier: intermediate_rep
                .genesis_block_identifier
                .into_iter()
                .next()
                .ok_or("genesis_block_identifier missing in NetworkStatusResponse".to_string())?,
            oldest_block_identifier: intermediate_rep.oldest_block_identifier.into_iter().next(),
            sync_status: intermediate_rep.sync_status.into_iter().next(),
            peers: intermediate_rep.peers.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkStatusResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<NetworkStatusResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<NetworkStatusResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for NetworkStatusResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<NetworkStatusResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <NetworkStatusResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into NetworkStatusResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Operations contain all balance-changing information within a transaction. They are always one-sided (only affect 1 AccountIdentifier) and can succeed or fail independently from a Transaction.  Operations are used both to represent on-chain data (Data API) and to construct new transactions (Construction API), creating a standard interface for reading and writing to blockchains.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Operation {
    #[serde(rename = "operation_identifier")]
    pub operation_identifier: models::OperationIdentifier,

    /// Restrict referenced related_operations to identifier indices < the current operation_identifier.index. This ensures there exists a clear DAG-structure of relations.  Since operations are one-sided, one could imagine relating operations in a single transfer or linking operations in a call tree.
    #[serde(rename = "related_operations")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_operations: Option<Vec<models::OperationIdentifier>>,

    /// Type is the network-specific type of the operation. Ensure that any type that can be returned here is also specified in the NetworkOptionsResponse. This can be very useful to downstream consumers that parse all block data.
    #[serde(rename = "type")]
    pub r#type: String,

    /// Status is the network-specific status of the operation. Status is not defined on the transaction object because blockchains with smart contracts may have transactions that partially apply (some operations are successful and some are not). Blockchains with atomic transactions (all operations succeed or all operations fail) will have the same status for each operation.  On-chain operations (operations retrieved in the `/block` and `/block/transaction` endpoints) MUST have a populated status field (anything on-chain must have succeeded or failed). However, operations provided during transaction construction (often times called \"intent\" in the documentation) MUST NOT have a populated status field (operations yet to be included on-chain have not yet succeeded or failed).
    #[serde(rename = "status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(rename = "account")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<models::AccountIdentifier>,

    #[serde(rename = "amount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<models::Amount>,

    #[serde(rename = "coin_change")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coin_change: Option<models::CoinChange>,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Operation {
    pub fn new(operation_identifier: models::OperationIdentifier, r#type: String) -> Operation {
        Operation {
            operation_identifier: operation_identifier,
            related_operations: None,
            r#type: r#type,
            status: None,
            account: None,
            amount: None,
            coin_change: None,
            metadata: None,
        }
    }
}

/// Converts the Operation value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Operation {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping operation_identifier in query parameter serialization

        // Skipping related_operations in query parameter serialization

        params.push("type".to_string());
        params.push(self.r#type.to_string());

        if let Some(ref status) = self.status {
            params.push("status".to_string());
            params.push(status.to_string());
        }

        // Skipping account in query parameter serialization

        // Skipping amount in query parameter serialization

        // Skipping coin_change in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Operation value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Operation {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub operation_identifier: Vec<models::OperationIdentifier>,
            pub related_operations: Vec<Vec<models::OperationIdentifier>>,
            pub r#type: Vec<String>,
            pub status: Vec<String>,
            pub account: Vec<models::AccountIdentifier>,
            pub amount: Vec<models::Amount>,
            pub coin_change: Vec<models::CoinChange>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Operation".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "operation_identifier" => intermediate_rep.operation_identifier.push(
                        <models::OperationIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "related_operations" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Operation"
                                .to_string(),
                        )
                    }
                    "type" => intermediate_rep.r#type.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "status" => intermediate_rep.status.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "account" => intermediate_rep.account.push(
                        <models::AccountIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "amount" => intermediate_rep.amount.push(
                        <models::Amount as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "coin_change" => intermediate_rep.coin_change.push(
                        <models::CoinChange as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Operation".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Operation {
            operation_identifier: intermediate_rep
                .operation_identifier
                .into_iter()
                .next()
                .ok_or("operation_identifier missing in Operation".to_string())?,
            related_operations: intermediate_rep.related_operations.into_iter().next(),
            r#type: intermediate_rep
                .r#type
                .into_iter()
                .next()
                .ok_or("type missing in Operation".to_string())?,
            status: intermediate_rep.status.into_iter().next(),
            account: intermediate_rep.account.into_iter().next(),
            amount: intermediate_rep.amount.into_iter().next(),
            coin_change: intermediate_rep.coin_change.into_iter().next(),
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Operation> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Operation>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Operation>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Operation - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Operation> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Operation as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Operation - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// The operation_identifier uniquely identifies an operation within a transaction.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct OperationIdentifier {
    /// The operation index is used to ensure each operation has a unique identifier within a transaction. This index is only relative to the transaction and NOT GLOBAL. The operations in each transaction should start from index 0.  To clarify, there may not be any notion of an operation index in the blockchain being described.
    #[serde(rename = "index")]
    pub index: i64,

    /// Some blockchains specify an operation index that is essential for client use. For example, Bitcoin uses a network_index to identify which UTXO was used in a transaction.  network_index should not be populated if there is no notion of an operation index in a blockchain (typically most account-based blockchains).
    #[serde(rename = "network_index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_index: Option<i64>,
}

impl OperationIdentifier {
    pub fn new(index: i64) -> OperationIdentifier {
        OperationIdentifier {
            index: index,
            network_index: None,
        }
    }
}

/// Converts the OperationIdentifier value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for OperationIdentifier {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("index".to_string());
        params.push(self.index.to_string());

        if let Some(ref network_index) = self.network_index {
            params.push("network_index".to_string());
            params.push(network_index.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a OperationIdentifier value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for OperationIdentifier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub index: Vec<i64>,
            pub network_index: Vec<i64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing OperationIdentifier".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "index" => intermediate_rep.index.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "network_index" => intermediate_rep.network_index.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing OperationIdentifier".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(OperationIdentifier {
            index: intermediate_rep
                .index
                .into_iter()
                .next()
                .ok_or("index missing in OperationIdentifier".to_string())?,
            network_index: intermediate_rep.network_index.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<OperationIdentifier> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<OperationIdentifier>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<OperationIdentifier>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for OperationIdentifier - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<OperationIdentifier>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <OperationIdentifier as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into OperationIdentifier - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// OperationStatus is utilized to indicate which Operation status are considered successful.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct OperationStatus {
    /// The status is the network-specific status of the operation.
    #[serde(rename = "status")]
    pub status: String,

    /// An Operation is considered successful if the Operation.Amount should affect the Operation.Account. Some blockchains (like Bitcoin) only include successful operations in blocks but other blockchains (like Ethereum) include unsuccessful operations that incur a fee.  To reconcile the computed balance from the stream of Operations, it is critical to understand which Operation.Status indicate an Operation is successful and should affect an Account.
    #[serde(rename = "successful")]
    pub successful: bool,
}

impl OperationStatus {
    pub fn new(status: String, successful: bool) -> OperationStatus {
        OperationStatus {
            status: status,
            successful: successful,
        }
    }
}

/// Converts the OperationStatus value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for OperationStatus {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("status".to_string());
        params.push(self.status.to_string());

        params.push("successful".to_string());
        params.push(self.successful.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a OperationStatus value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for OperationStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub status: Vec<String>,
            pub successful: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing OperationStatus".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "status" => intermediate_rep.status.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "successful" => intermediate_rep.successful.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing OperationStatus".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(OperationStatus {
            status: intermediate_rep
                .status
                .into_iter()
                .next()
                .ok_or("status missing in OperationStatus".to_string())?,
            successful: intermediate_rep
                .successful
                .into_iter()
                .next()
                .ok_or("successful missing in OperationStatus".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<OperationStatus> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<OperationStatus>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<OperationStatus>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for OperationStatus - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<OperationStatus>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <OperationStatus as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into OperationStatus - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Operator is used by query-related endpoints to determine how to apply conditions.  If this field is not populated, the default `and` value will be used.
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum Operator {
    #[serde(rename = "or")]
    Or,
    #[serde(rename = "and")]
    And,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Operator::Or => write!(f, "{}", "or"),
            Operator::And => write!(f, "{}", "and"),
        }
    }
}

impl std::str::FromStr for Operator {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "or" => std::result::Result::Ok(Operator::Or),
            "and" => std::result::Result::Ok(Operator::And),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

/// When fetching data by BlockIdentifier, it may be possible to only specify the index or hash. If neither property is specified, it is assumed that the client is making a request at the current block.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct PartialBlockIdentifier {
    #[serde(rename = "index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

    #[serde(rename = "hash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl PartialBlockIdentifier {
    pub fn new() -> PartialBlockIdentifier {
        PartialBlockIdentifier {
            index: None,
            hash: None,
        }
    }
}

/// Converts the PartialBlockIdentifier value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for PartialBlockIdentifier {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref index) = self.index {
            params.push("index".to_string());
            params.push(index.to_string());
        }

        if let Some(ref hash) = self.hash {
            params.push("hash".to_string());
            params.push(hash.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a PartialBlockIdentifier value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for PartialBlockIdentifier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub index: Vec<i64>,
            pub hash: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing PartialBlockIdentifier".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "index" => intermediate_rep.index.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "hash" => intermediate_rep.hash.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing PartialBlockIdentifier".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(PartialBlockIdentifier {
            index: intermediate_rep.index.into_iter().next(),
            hash: intermediate_rep.hash.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<PartialBlockIdentifier> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<PartialBlockIdentifier>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<PartialBlockIdentifier>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for PartialBlockIdentifier - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<PartialBlockIdentifier>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <PartialBlockIdentifier as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into PartialBlockIdentifier - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// A Peer is a representation of a node's peer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Peer {
    #[serde(rename = "peer_id")]
    pub peer_id: String,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Peer {
    pub fn new(peer_id: String) -> Peer {
        Peer {
            peer_id: peer_id,
            metadata: None,
        }
    }
}

/// Converts the Peer value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Peer {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("peer_id".to_string());
        params.push(self.peer_id.to_string());

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Peer value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Peer {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub peer_id: Vec<String>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing Peer".to_string())
                }
            };

            if let Some(key) = key_result {
                match key {
                    "peer_id" => intermediate_rep.peer_id.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Peer".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Peer {
            peer_id: intermediate_rep
                .peer_id
                .into_iter()
                .next()
                .ok_or("peer_id missing in Peer".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Peer> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Peer>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Peer>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Peer - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Peer> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Peer as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => {
                    std::result::Result::Ok(header::IntoHeaderValue(value))
                }
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Peer - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// PublicKey contains a public key byte array for a particular CurveType encoded in hex.  Note that there is no PrivateKey struct as this is NEVER the concern of an implementation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct PublicKey {
    /// Hex-encoded public key bytes in the format specified by the CurveType.
    #[serde(rename = "hex_bytes")]
    pub hex_bytes: String,

    #[serde(rename = "curve_type")]
    pub curve_type: models::CurveType,
}

impl PublicKey {
    pub fn new(hex_bytes: String, curve_type: models::CurveType) -> PublicKey {
        PublicKey {
            hex_bytes: hex_bytes,
            curve_type: curve_type,
        }
    }
}

/// Converts the PublicKey value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for PublicKey {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("hex_bytes".to_string());
        params.push(self.hex_bytes.to_string());

        // Skipping curve_type in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a PublicKey value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for PublicKey {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub hex_bytes: Vec<String>,
            pub curve_type: Vec<models::CurveType>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing PublicKey".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "hex_bytes" => intermediate_rep.hex_bytes.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "curve_type" => intermediate_rep.curve_type.push(
                        <models::CurveType as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing PublicKey".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(PublicKey {
            hex_bytes: intermediate_rep
                .hex_bytes
                .into_iter()
                .next()
                .ok_or("hex_bytes missing in PublicKey".to_string())?,
            curve_type: intermediate_rep
                .curve_type
                .into_iter()
                .next()
                .ok_or("curve_type missing in PublicKey".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<PublicKey> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<PublicKey>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<PublicKey>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for PublicKey - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<PublicKey> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <PublicKey as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into PublicKey - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// The related_transaction allows implementations to link together multiple transactions. An unpopulated network identifier indicates that the related transaction is on the same network.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct RelatedTransaction {
    #[serde(rename = "network_identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_identifier: Option<models::NetworkIdentifier>,

    #[serde(rename = "transaction_identifier")]
    pub transaction_identifier: models::TransactionIdentifier,

    #[serde(rename = "direction")]
    pub direction: models::Direction,
}

impl RelatedTransaction {
    pub fn new(
        transaction_identifier: models::TransactionIdentifier,
        direction: models::Direction,
    ) -> RelatedTransaction {
        RelatedTransaction {
            network_identifier: None,
            transaction_identifier: transaction_identifier,
            direction: direction,
        }
    }
}

/// Converts the RelatedTransaction value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for RelatedTransaction {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping transaction_identifier in query parameter serialization

        // Skipping direction in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a RelatedTransaction value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for RelatedTransaction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub transaction_identifier: Vec<models::TransactionIdentifier>,
            pub direction: Vec<models::Direction>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing RelatedTransaction".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "transaction_identifier" => intermediate_rep.transaction_identifier.push(
                        <models::TransactionIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "direction" => intermediate_rep.direction.push(
                        <models::Direction as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing RelatedTransaction".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(RelatedTransaction {
            network_identifier: intermediate_rep.network_identifier.into_iter().next(),
            transaction_identifier: intermediate_rep
                .transaction_identifier
                .into_iter()
                .next()
                .ok_or("transaction_identifier missing in RelatedTransaction".to_string())?,
            direction: intermediate_rep
                .direction
                .into_iter()
                .next()
                .ok_or("direction missing in RelatedTransaction".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<RelatedTransaction> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<RelatedTransaction>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<RelatedTransaction>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for RelatedTransaction - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<RelatedTransaction>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <RelatedTransaction as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into RelatedTransaction - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// SearchTransactionsRequest is used to search for transactions matching a set of provided conditions in canonical blocks.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SearchTransactionsRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    #[serde(rename = "operator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<models::Operator>,

    /// max_block is the largest block index to consider when searching for transactions. If this field is not populated, the current block is considered the max_block.  If you do not specify a max_block, it is possible a newly synced block will interfere with paginated transaction queries (as the offset could become invalid with newly added rows).
    #[serde(rename = "max_block")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block: Option<i64>,

    /// offset is the offset into the query result to start returning transactions.  If any search conditions are changed, the query offset will change and you must restart your search iteration.
    #[serde(rename = "offset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,

    /// limit is the maximum number of transactions to return in one call. The implementation may return <= limit transactions.
    #[serde(rename = "limit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    #[serde(rename = "transaction_identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_identifier: Option<models::TransactionIdentifier>,

    #[serde(rename = "account_identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_identifier: Option<models::AccountIdentifier>,

    #[serde(rename = "coin_identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coin_identifier: Option<models::CoinIdentifier>,

    #[serde(rename = "currency")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<models::Currency>,

    /// status is the network-specific operation type.
    #[serde(rename = "status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// type is the network-specific operation type.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// address is AccountIdentifier.Address. This is used to get all transactions related to an AccountIdentifier.Address, regardless of SubAccountIdentifier.
    #[serde(rename = "address")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// success is a synthetic condition populated by parsing network-specific operation statuses (using the mapping provided in `/network/options`).
    #[serde(rename = "success")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
}

impl SearchTransactionsRequest {
    pub fn new(network_identifier: models::NetworkIdentifier) -> SearchTransactionsRequest {
        SearchTransactionsRequest {
            network_identifier: network_identifier,
            operator: None,
            max_block: None,
            offset: None,
            limit: None,
            transaction_identifier: None,
            account_identifier: None,
            coin_identifier: None,
            currency: None,
            status: None,
            r#type: None,
            address: None,
            success: None,
        }
    }
}

/// Converts the SearchTransactionsRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for SearchTransactionsRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        // Skipping operator in query parameter serialization

        if let Some(ref max_block) = self.max_block {
            params.push("max_block".to_string());
            params.push(max_block.to_string());
        }

        if let Some(ref offset) = self.offset {
            params.push("offset".to_string());
            params.push(offset.to_string());
        }

        if let Some(ref limit) = self.limit {
            params.push("limit".to_string());
            params.push(limit.to_string());
        }

        // Skipping transaction_identifier in query parameter serialization

        // Skipping account_identifier in query parameter serialization

        // Skipping coin_identifier in query parameter serialization

        // Skipping currency in query parameter serialization

        if let Some(ref status) = self.status {
            params.push("status".to_string());
            params.push(status.to_string());
        }

        if let Some(ref r#type) = self.r#type {
            params.push("type".to_string());
            params.push(r#type.to_string());
        }

        if let Some(ref address) = self.address {
            params.push("address".to_string());
            params.push(address.to_string());
        }

        if let Some(ref success) = self.success {
            params.push("success".to_string());
            params.push(success.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SearchTransactionsRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SearchTransactionsRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub operator: Vec<models::Operator>,
            pub max_block: Vec<i64>,
            pub offset: Vec<i64>,
            pub limit: Vec<i64>,
            pub transaction_identifier: Vec<models::TransactionIdentifier>,
            pub account_identifier: Vec<models::AccountIdentifier>,
            pub coin_identifier: Vec<models::CoinIdentifier>,
            pub currency: Vec<models::Currency>,
            pub status: Vec<String>,
            pub r#type: Vec<String>,
            pub address: Vec<String>,
            pub success: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SearchTransactionsRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(
                        <models::NetworkIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "operator" => intermediate_rep.operator.push(
                        <models::Operator as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "max_block" => intermediate_rep.max_block.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "offset" => intermediate_rep.offset.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "limit" => intermediate_rep.limit.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "transaction_identifier" => intermediate_rep.transaction_identifier.push(
                        <models::TransactionIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "account_identifier" => intermediate_rep.account_identifier.push(
                        <models::AccountIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "coin_identifier" => intermediate_rep.coin_identifier.push(
                        <models::CoinIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "currency" => intermediate_rep.currency.push(
                        <models::Currency as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "status" => intermediate_rep.status.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "type" => intermediate_rep.r#type.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "address" => intermediate_rep.address.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "success" => intermediate_rep.success.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing SearchTransactionsRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SearchTransactionsRequest {
            network_identifier: intermediate_rep
                .network_identifier
                .into_iter()
                .next()
                .ok_or("network_identifier missing in SearchTransactionsRequest".to_string())?,
            operator: intermediate_rep.operator.into_iter().next(),
            max_block: intermediate_rep.max_block.into_iter().next(),
            offset: intermediate_rep.offset.into_iter().next(),
            limit: intermediate_rep.limit.into_iter().next(),
            transaction_identifier: intermediate_rep.transaction_identifier.into_iter().next(),
            account_identifier: intermediate_rep.account_identifier.into_iter().next(),
            coin_identifier: intermediate_rep.coin_identifier.into_iter().next(),
            currency: intermediate_rep.currency.into_iter().next(),
            status: intermediate_rep.status.into_iter().next(),
            r#type: intermediate_rep.r#type.into_iter().next(),
            address: intermediate_rep.address.into_iter().next(),
            success: intermediate_rep.success.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SearchTransactionsRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<SearchTransactionsRequest>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SearchTransactionsRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SearchTransactionsRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<SearchTransactionsRequest>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SearchTransactionsRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SearchTransactionsRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// SearchTransactionsResponse contains an ordered collection of BlockTransactions that match the query in SearchTransactionsRequest. These BlockTransactions are sorted from most recent block to oldest block.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SearchTransactionsResponse {
    /// transactions is an array of BlockTransactions sorted by most recent BlockIdentifier (meaning that transactions in recent blocks appear first).  If there are many transactions for a particular search, transactions may not contain all matching transactions. It is up to the caller to paginate these transactions using the max_block field.
    #[serde(rename = "transactions")]
    pub transactions: Vec<models::BlockTransaction>,

    /// total_count is the number of results for a given search. Callers typically use this value to concurrently fetch results by offset or to display a virtual page number associated with results.
    #[serde(rename = "total_count")]
    pub total_count: i64,

    /// next_offset is the next offset to use when paginating through transaction results. If this field is not populated, there are no more transactions to query.
    #[serde(rename = "next_offset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
}

impl SearchTransactionsResponse {
    pub fn new(
        transactions: Vec<models::BlockTransaction>,
        total_count: i64,
    ) -> SearchTransactionsResponse {
        SearchTransactionsResponse {
            transactions: transactions,
            total_count: total_count,
            next_offset: None,
        }
    }
}

/// Converts the SearchTransactionsResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for SearchTransactionsResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping transactions in query parameter serialization

        params.push("total_count".to_string());
        params.push(self.total_count.to_string());

        if let Some(ref next_offset) = self.next_offset {
            params.push("next_offset".to_string());
            params.push(next_offset.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SearchTransactionsResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SearchTransactionsResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub transactions: Vec<Vec<models::BlockTransaction>>,
            pub total_count: Vec<i64>,
            pub next_offset: Vec<i64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SearchTransactionsResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "transactions" => return std::result::Result::Err("Parsing a container in this style is not supported in SearchTransactionsResponse".to_string()),
                    "total_count" => intermediate_rep.total_count.push(<i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "next_offset" => intermediate_rep.next_offset.push(<i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing SearchTransactionsResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SearchTransactionsResponse {
            transactions: intermediate_rep
                .transactions
                .into_iter()
                .next()
                .ok_or("transactions missing in SearchTransactionsResponse".to_string())?,
            total_count: intermediate_rep
                .total_count
                .into_iter()
                .next()
                .ok_or("total_count missing in SearchTransactionsResponse".to_string())?,
            next_offset: intermediate_rep.next_offset.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SearchTransactionsResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<SearchTransactionsResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SearchTransactionsResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SearchTransactionsResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<SearchTransactionsResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SearchTransactionsResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SearchTransactionsResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Signature contains the payload that was signed, the public keys of the keypairs used to produce the signature, the signature (encoded in hex), and the SignatureType.  PublicKey is often times not known during construction of the signing payloads but may be needed to combine signatures properly.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Signature {
    #[serde(rename = "signing_payload")]
    pub signing_payload: models::SigningPayload,

    #[serde(rename = "public_key")]
    pub public_key: models::PublicKey,

    #[serde(rename = "signature_type")]
    pub signature_type: models::SignatureType,

    #[serde(rename = "hex_bytes")]
    pub hex_bytes: String,
}

impl Signature {
    pub fn new(
        signing_payload: models::SigningPayload,
        public_key: models::PublicKey,
        signature_type: models::SignatureType,
        hex_bytes: String,
    ) -> Signature {
        Signature {
            signing_payload: signing_payload,
            public_key: public_key,
            signature_type: signature_type,
            hex_bytes: hex_bytes,
        }
    }
}

/// Converts the Signature value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Signature {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping signing_payload in query parameter serialization

        // Skipping public_key in query parameter serialization

        // Skipping signature_type in query parameter serialization

        params.push("hex_bytes".to_string());
        params.push(self.hex_bytes.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Signature value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Signature {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub signing_payload: Vec<models::SigningPayload>,
            pub public_key: Vec<models::PublicKey>,
            pub signature_type: Vec<models::SignatureType>,
            pub hex_bytes: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Signature".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "signing_payload" => intermediate_rep.signing_payload.push(
                        <models::SigningPayload as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "public_key" => intermediate_rep.public_key.push(
                        <models::PublicKey as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "signature_type" => intermediate_rep.signature_type.push(
                        <models::SignatureType as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "hex_bytes" => intermediate_rep.hex_bytes.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Signature".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Signature {
            signing_payload: intermediate_rep
                .signing_payload
                .into_iter()
                .next()
                .ok_or("signing_payload missing in Signature".to_string())?,
            public_key: intermediate_rep
                .public_key
                .into_iter()
                .next()
                .ok_or("public_key missing in Signature".to_string())?,
            signature_type: intermediate_rep
                .signature_type
                .into_iter()
                .next()
                .ok_or("signature_type missing in Signature".to_string())?,
            hex_bytes: intermediate_rep
                .hex_bytes
                .into_iter()
                .next()
                .ok_or("hex_bytes missing in Signature".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Signature> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Signature>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Signature>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Signature - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Signature> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Signature as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Signature - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// SignatureType is the type of a cryptographic signature.  * ecdsa: `r (32-bytes) || s (32-bytes)` - `64 bytes` * ecdsa_recovery: `r (32-bytes) || s (32-bytes) || v (1-byte)` - `65 bytes` * ed25519: `R (32-byte) || s (32-bytes)` - `64 bytes` * schnorr_1: `r (32-bytes) || s (32-bytes)` - `64 bytes`  (schnorr signature implemented by Zilliqa where both `r` and `s` are scalars encoded as `32-bytes` values, most significant byte first.) * schnorr_poseidon: `r (32-bytes) || s (32-bytes)` where s = Hash(1st pk || 2nd pk || r) - `64 bytes`  (schnorr signature w/ Poseidon hash function implemented by O(1) Labs where both `r` and `s` are scalars encoded as `32-bytes` values, least significant byte first. https://github.com/CodaProtocol/signer-reference/blob/master/schnorr.ml )
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum SignatureType {
    #[serde(rename = "ecdsa")]
    Ecdsa,
    #[serde(rename = "ecdsa_recovery")]
    EcdsaRecovery,
    #[serde(rename = "ed25519")]
    Ed25519,
    #[serde(rename = "schnorr_1")]
    Schnorr1,
    #[serde(rename = "schnorr_poseidon")]
    SchnorrPoseidon,
}

impl std::fmt::Display for SignatureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SignatureType::Ecdsa => write!(f, "{}", "ecdsa"),
            SignatureType::EcdsaRecovery => write!(f, "{}", "ecdsa_recovery"),
            SignatureType::Ed25519 => write!(f, "{}", "ed25519"),
            SignatureType::Schnorr1 => write!(f, "{}", "schnorr_1"),
            SignatureType::SchnorrPoseidon => write!(f, "{}", "schnorr_poseidon"),
        }
    }
}

impl std::str::FromStr for SignatureType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "ecdsa" => std::result::Result::Ok(SignatureType::Ecdsa),
            "ecdsa_recovery" => std::result::Result::Ok(SignatureType::EcdsaRecovery),
            "ed25519" => std::result::Result::Ok(SignatureType::Ed25519),
            "schnorr_1" => std::result::Result::Ok(SignatureType::Schnorr1),
            "schnorr_poseidon" => std::result::Result::Ok(SignatureType::SchnorrPoseidon),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

/// SigningPayload is signed by the client with the keypair associated with an AccountIdentifier using the specified SignatureType.  SignatureType can be optionally populated if there is a restriction on the signature scheme that can be used to sign the payload.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SigningPayload {
    /// [DEPRECATED by `account_identifier` in `v1.4.4`] The network-specific address of the account that should sign the payload.
    #[serde(rename = "address")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    #[serde(rename = "account_identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_identifier: Option<models::AccountIdentifier>,

    /// Hex-encoded string of the payload bytes.
    #[serde(rename = "hex_bytes")]
    pub hex_bytes: String,

    #[serde(rename = "signature_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_type: Option<models::SignatureType>,
}

impl SigningPayload {
    pub fn new(hex_bytes: String) -> SigningPayload {
        SigningPayload {
            address: None,
            account_identifier: None,
            hex_bytes: hex_bytes,
            signature_type: None,
        }
    }
}

/// Converts the SigningPayload value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for SigningPayload {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref address) = self.address {
            params.push("address".to_string());
            params.push(address.to_string());
        }

        // Skipping account_identifier in query parameter serialization

        params.push("hex_bytes".to_string());
        params.push(self.hex_bytes.to_string());

        // Skipping signature_type in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SigningPayload value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SigningPayload {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub address: Vec<String>,
            pub account_identifier: Vec<models::AccountIdentifier>,
            pub hex_bytes: Vec<String>,
            pub signature_type: Vec<models::SignatureType>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SigningPayload".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "address" => intermediate_rep.address.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "account_identifier" => intermediate_rep.account_identifier.push(
                        <models::AccountIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "hex_bytes" => intermediate_rep.hex_bytes.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "signature_type" => intermediate_rep.signature_type.push(
                        <models::SignatureType as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing SigningPayload".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SigningPayload {
            address: intermediate_rep.address.into_iter().next(),
            account_identifier: intermediate_rep.account_identifier.into_iter().next(),
            hex_bytes: intermediate_rep
                .hex_bytes
                .into_iter()
                .next()
                .ok_or("hex_bytes missing in SigningPayload".to_string())?,
            signature_type: intermediate_rep.signature_type.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SigningPayload> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<SigningPayload>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SigningPayload>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SigningPayload - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<SigningPayload> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SigningPayload as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SigningPayload - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// An account may have state specific to a contract address (ERC-20 token) and/or a stake (delegated balance). The sub_account_identifier should specify which state (if applicable) an account instantiation refers to.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SubAccountIdentifier {
    /// The SubAccount address may be a cryptographic value or some other identifier (ex: bonded) that uniquely specifies a SubAccount.
    #[serde(rename = "address")]
    pub address: String,

    /// If the SubAccount address is not sufficient to uniquely specify a SubAccount, any other identifying information can be stored here.  It is important to note that two SubAccounts with identical addresses but differing metadata will not be considered equal by clients.
    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl SubAccountIdentifier {
    pub fn new(address: String) -> SubAccountIdentifier {
        SubAccountIdentifier {
            address: address,
            metadata: None,
        }
    }
}

/// Converts the SubAccountIdentifier value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for SubAccountIdentifier {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("address".to_string());
        params.push(self.address.to_string());

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SubAccountIdentifier value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SubAccountIdentifier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub address: Vec<String>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SubAccountIdentifier".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "address" => intermediate_rep.address.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing SubAccountIdentifier".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SubAccountIdentifier {
            address: intermediate_rep
                .address
                .into_iter()
                .next()
                .ok_or("address missing in SubAccountIdentifier".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SubAccountIdentifier> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<SubAccountIdentifier>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SubAccountIdentifier>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SubAccountIdentifier - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<SubAccountIdentifier>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SubAccountIdentifier as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SubAccountIdentifier - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// In blockchains with sharded state, the SubNetworkIdentifier is required to query some object on a specific shard. This identifier is optional for all non-sharded blockchains.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SubNetworkIdentifier {
    #[serde(rename = "network")]
    pub network: String,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl SubNetworkIdentifier {
    pub fn new(network: String) -> SubNetworkIdentifier {
        SubNetworkIdentifier {
            network: network,
            metadata: None,
        }
    }
}

/// Converts the SubNetworkIdentifier value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for SubNetworkIdentifier {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("network".to_string());
        params.push(self.network.to_string());

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SubNetworkIdentifier value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SubNetworkIdentifier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network: Vec<String>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SubNetworkIdentifier".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "network" => intermediate_rep.network.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing SubNetworkIdentifier".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SubNetworkIdentifier {
            network: intermediate_rep
                .network
                .into_iter()
                .next()
                .ok_or("network missing in SubNetworkIdentifier".to_string())?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SubNetworkIdentifier> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<SubNetworkIdentifier>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SubNetworkIdentifier>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SubNetworkIdentifier - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<SubNetworkIdentifier>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SubNetworkIdentifier as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SubNetworkIdentifier - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// SyncStatus is used to provide additional context about an implementation's sync status.  This object is often used by implementations to indicate healthiness when block data cannot be queried until some sync phase completes or cannot be determined by comparing the timestamp of the most recent block with the current time.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SyncStatus {
    /// CurrentIndex is the index of the last synced block in the current stage.  This is a separate field from current_block_identifier in NetworkStatusResponse because blocks with indices up to and including the current_index may not yet be queryable by the caller. To reiterate, all indices up to and including current_block_identifier in NetworkStatusResponse must be queryable via the /block endpoint (excluding indices less than oldest_block_identifier).
    #[serde(rename = "current_index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_index: Option<i64>,

    /// TargetIndex is the index of the block that the implementation is attempting to sync to in the current stage.
    #[serde(rename = "target_index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_index: Option<i64>,

    /// Stage is the phase of the sync process.
    #[serde(rename = "stage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,

    /// synced is a boolean that indicates if an implementation has synced up to the most recent block. If this field is not populated, the caller should rely on a traditional tip timestamp comparison to determine if an implementation is synced.  This field is particularly useful for quiescent blockchains (blocks only produced when there are pending transactions). In these blockchains, the most recent block could have a timestamp far behind the current time but the node could be healthy and at tip.
    #[serde(rename = "synced")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synced: Option<bool>,
}

impl SyncStatus {
    pub fn new() -> SyncStatus {
        SyncStatus {
            current_index: None,
            target_index: None,
            stage: None,
            synced: None,
        }
    }
}

/// Converts the SyncStatus value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for SyncStatus {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref current_index) = self.current_index {
            params.push("current_index".to_string());
            params.push(current_index.to_string());
        }

        if let Some(ref target_index) = self.target_index {
            params.push("target_index".to_string());
            params.push(target_index.to_string());
        }

        if let Some(ref stage) = self.stage {
            params.push("stage".to_string());
            params.push(stage.to_string());
        }

        if let Some(ref synced) = self.synced {
            params.push("synced".to_string());
            params.push(synced.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SyncStatus value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SyncStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub current_index: Vec<i64>,
            pub target_index: Vec<i64>,
            pub stage: Vec<String>,
            pub synced: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SyncStatus".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "current_index" => intermediate_rep.current_index.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "target_index" => intermediate_rep.target_index.push(
                        <i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "stage" => intermediate_rep.stage.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "synced" => intermediate_rep.synced.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing SyncStatus".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SyncStatus {
            current_index: intermediate_rep.current_index.into_iter().next(),
            target_index: intermediate_rep.target_index.into_iter().next(),
            stage: intermediate_rep.stage.into_iter().next(),
            synced: intermediate_rep.synced.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SyncStatus> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<SyncStatus>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SyncStatus>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SyncStatus - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<SyncStatus> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SyncStatus as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SyncStatus - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// The timestamp of the block in milliseconds since the Unix Epoch. The timestamp is stored in milliseconds because some blockchains produce blocks more often than once a second.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Timestamp(i64);

impl std::convert::From<i64> for Timestamp {
    fn from(x: i64) -> Self {
        Timestamp(x)
    }
}

impl std::convert::From<Timestamp> for i64 {
    fn from(x: Timestamp) -> Self {
        x.0
    }
}

impl std::ops::Deref for Timestamp {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}

impl std::ops::DerefMut for Timestamp {
    fn deref_mut(&mut self) -> &mut i64 {
        &mut self.0
    }
}

/// Transactions contain an array of Operations that are attributable to the same TransactionIdentifier.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Transaction {
    #[serde(rename = "transaction_identifier")]
    pub transaction_identifier: models::TransactionIdentifier,

    #[serde(rename = "operations")]
    pub operations: Vec<models::Operation>,

    #[serde(rename = "related_transactions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_transactions: Option<Vec<models::RelatedTransaction>>,

    /// Transactions that are related to other transactions (like a cross-shard transaction) should include the tranaction_identifier of these transactions in the metadata.
    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Transaction {
    pub fn new(
        transaction_identifier: models::TransactionIdentifier,
        operations: Vec<models::Operation>,
    ) -> Transaction {
        Transaction {
            transaction_identifier: transaction_identifier,
            operations: operations,
            related_transactions: None,
            metadata: None,
        }
    }
}

/// Converts the Transaction value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Transaction {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping transaction_identifier in query parameter serialization

        // Skipping operations in query parameter serialization

        // Skipping related_transactions in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Transaction value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Transaction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub transaction_identifier: Vec<models::TransactionIdentifier>,
            pub operations: Vec<Vec<models::Operation>>,
            pub related_transactions: Vec<Vec<models::RelatedTransaction>>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Transaction".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "transaction_identifier" => intermediate_rep.transaction_identifier.push(
                        <models::TransactionIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "operations" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Transaction"
                                .to_string(),
                        )
                    }
                    "related_transactions" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Transaction"
                                .to_string(),
                        )
                    }
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Transaction".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Transaction {
            transaction_identifier: intermediate_rep
                .transaction_identifier
                .into_iter()
                .next()
                .ok_or("transaction_identifier missing in Transaction".to_string())?,
            operations: intermediate_rep
                .operations
                .into_iter()
                .next()
                .ok_or("operations missing in Transaction".to_string())?,
            related_transactions: intermediate_rep.related_transactions.into_iter().next(),
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Transaction> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Transaction>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Transaction>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Transaction - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Transaction> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Transaction as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Transaction - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// The transaction_identifier uniquely identifies a transaction in a particular network and block or in the mempool.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionIdentifier {
    /// Any transactions that are attributable only to a block (ex: a block event) should use the hash of the block as the identifier.  This should be normalized according to the case specified in the transaction_hash_case in network options.
    #[serde(rename = "hash")]
    pub hash: String,
}

impl TransactionIdentifier {
    pub fn new(hash: String) -> TransactionIdentifier {
        TransactionIdentifier { hash: hash }
    }
}

/// Converts the TransactionIdentifier value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionIdentifier {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("hash".to_string());
        params.push(self.hash.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionIdentifier value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionIdentifier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub hash: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing TransactionIdentifier".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "hash" => intermediate_rep.hash.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing TransactionIdentifier".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionIdentifier {
            hash: intermediate_rep
                .hash
                .into_iter()
                .next()
                .ok_or("hash missing in TransactionIdentifier".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionIdentifier> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<TransactionIdentifier>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<TransactionIdentifier>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for TransactionIdentifier - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<TransactionIdentifier>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <TransactionIdentifier as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into TransactionIdentifier - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// TransactionIdentifierResponse contains the transaction_identifier of a transaction that was submitted to either `/construction/hash` or `/construction/submit`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionIdentifierResponse {
    #[serde(rename = "transaction_identifier")]
    pub transaction_identifier: models::TransactionIdentifier,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl TransactionIdentifierResponse {
    pub fn new(
        transaction_identifier: models::TransactionIdentifier,
    ) -> TransactionIdentifierResponse {
        TransactionIdentifierResponse {
            transaction_identifier: transaction_identifier,
            metadata: None,
        }
    }
}

/// Converts the TransactionIdentifierResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionIdentifierResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping transaction_identifier in query parameter serialization

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionIdentifierResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionIdentifierResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub transaction_identifier: Vec<models::TransactionIdentifier>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing TransactionIdentifierResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "transaction_identifier" => intermediate_rep.transaction_identifier.push(
                        <models::TransactionIdentifier as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing TransactionIdentifierResponse"
                                .to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionIdentifierResponse {
            transaction_identifier: intermediate_rep
                .transaction_identifier
                .into_iter()
                .next()
                .ok_or(
                    "transaction_identifier missing in TransactionIdentifierResponse".to_string(),
                )?,
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionIdentifierResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<TransactionIdentifierResponse>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<TransactionIdentifierResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for TransactionIdentifierResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<TransactionIdentifierResponse>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionIdentifierResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionIdentifierResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

/// The Version object is utilized to inform the client of the versions of different components of the Rosetta implementation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Version {
    /// The rosetta_version is the version of the Rosetta interface the implementation adheres to. This can be useful for clients looking to reliably parse responses.
    #[serde(rename = "rosetta_version")]
    pub rosetta_version: String,

    /// The node_version is the canonical version of the node runtime. This can help clients manage deployments.
    #[serde(rename = "node_version")]
    pub node_version: String,

    /// When a middleware server is used to adhere to the Rosetta interface, it should return its version here. This can help clients manage deployments.
    #[serde(rename = "middleware_version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middleware_version: Option<String>,

    /// Any other information that may be useful about versioning of dependent services should be returned here.
    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Version {
    pub fn new(rosetta_version: String, node_version: String) -> Version {
        Version {
            rosetta_version: rosetta_version,
            node_version: node_version,
            middleware_version: None,
            metadata: None,
        }
    }
}

/// Converts the Version value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Version {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("rosetta_version".to_string());
        params.push(self.rosetta_version.to_string());

        params.push("node_version".to_string());
        params.push(self.node_version.to_string());

        if let Some(ref middleware_version) = self.middleware_version {
            params.push("middleware_version".to_string());
            params.push(middleware_version.to_string());
        }

        // Skipping metadata in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Version value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub rosetta_version: Vec<String>,
            pub node_version: Vec<String>,
            pub middleware_version: Vec<String>,
            pub metadata: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Version".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "rosetta_version" => intermediate_rep.rosetta_version.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "node_version" => intermediate_rep.node_version.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "middleware_version" => intermediate_rep.middleware_version.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "metadata" => intermediate_rep.metadata.push(
                        <serde_json::Value as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Version".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Version {
            rosetta_version: intermediate_rep
                .rosetta_version
                .into_iter()
                .next()
                .ok_or("rosetta_version missing in Version".to_string())?,
            node_version: intermediate_rep
                .node_version
                .into_iter()
                .next()
                .ok_or("node_version missing in Version".to_string())?,
            middleware_version: intermediate_rep.middleware_version.into_iter().next(),
            metadata: intermediate_rep.metadata.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Version> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Version>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Version>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Version - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Version> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Version as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Version - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}
