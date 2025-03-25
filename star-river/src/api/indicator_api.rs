use types::indicator::Indicators;
use crate::star_river::StarRiver;
use axum::http::StatusCode;
use axum::extract::State;
use types::market::{Exchange, KlineInterval};
use axum::response::IntoResponse;
use serde::Deserialize;
use axum::extract::Json;
use std::str::FromStr;

