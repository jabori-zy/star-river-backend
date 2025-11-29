use star_river_core::custom_type::*;

use super::VirtualTradingSystemContext;
use crate::types::VirtualTransaction;

impl<E> VirtualTradingSystemContext<E> where E: Clone + Send + Sync + 'static {}
