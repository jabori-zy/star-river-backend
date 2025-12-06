use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use deepsize::DeepSizeOf;
use serde::{Deserialize, Serialize, ser::Serializer};
use utoipa::ToSchema;

use crate::core_error::{CoreError, ParseExchangeFailedSnafu};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum MarketType {
    Spot,
    Futures(MarginType),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum MarginType {
    U,
    Coin,
}

pub type MT5Server = String;

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash, DeepSizeOf, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Exchange {
    #[serde(rename = "binance")]
    Binance,

    #[serde(rename = "huobi")]
    Huobi,

    #[serde(rename = "okx")]
    Okx,

    #[serde(rename = "metatrader5")]
    Metatrader5(MT5Server),
}

impl Display for Exchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exchange::Binance => write!(f, "binance"),
            Exchange::Huobi => write!(f, "huobi"),
            Exchange::Okx => write!(f, "okx"),
            Exchange::Metatrader5(server) => {
                if server.is_empty() {
                    write!(f, "metatrader5")
                } else {
                    write!(f, "metatrader5({})", server)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExchangeStatus {
    NotRegist,      // Not registered
    Registing,      // Registering
    Connected,      // Connected
    RegisterFailed, // Registration failed
    Error,          // Error
}

// impl ToString for Exchange {
//     fn to_string(&self) -> String {
//         match self {
//             Exchange::Binance => "binance".to_string(),
//             Exchange::Huobi => "huobi".to_string(),
//             Exchange::Okx => "okx".to_string(),
//             Exchange::Metatrader5(server) => {
//                 if server.is_empty() {
//                     "metatrader5".to_string()
//                 } else {
//                     format!("metatrader5({})", server)
//                 }
//             }
//         }
//     }
// }

impl Serialize for Exchange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Exchange::Binance => serializer.serialize_str("binance"),
            Exchange::Huobi => serializer.serialize_str("huobi"),
            Exchange::Okx => serializer.serialize_str("okx"),
            Exchange::Metatrader5(server) => {
                if server.is_empty() {
                    serializer.serialize_str("metatrader5")
                } else {
                    serializer.serialize_str(&format!("metatrader5({})", server))
                }
            }
        }
    }
}

impl FromStr for Exchange {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "binance" => Ok(Exchange::Binance),
            "huobi" => Ok(Exchange::Huobi),
            "okx" => Ok(Exchange::Okx),
            _ => {
                // If it's metatrader5, parse out the server
                if s.starts_with("metatrader5") {
                    // Check if using parenthesis format: metatrader5(server)
                    if s.contains("(") && s.ends_with(")") {
                        let start = s.find("(").unwrap() + 1;
                        let end = s.len() - 1;
                        let server = &s[start..end];
                        Ok(Exchange::Metatrader5(server.to_string()))
                    }
                    // Compatible with legacy colon format: metatrader5:server
                    else if s.contains(":") {
                        let parts = s.split(":").collect::<Vec<&str>>();
                        if parts.len() > 1 {
                            Ok(Exchange::Metatrader5(parts[1].to_string()))
                        } else {
                            Ok(Exchange::Metatrader5(String::new()))
                        }
                    }
                    // Case without server information
                    else {
                        Ok(Exchange::Metatrader5(String::new()))
                    }
                } else {
                    Err(ParseExchangeFailedSnafu { exchange: s.to_string() }.build())
                }
            }
        }
    }
}

pub fn deserialize_exchange<'de, D>(deserializer: D) -> Result<Exchange, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // First try regular deserialization
    let exchange_str = String::deserialize(deserializer)?;
    Exchange::from_str(&exchange_str).map_err(serde::de::Error::custom)
}
