pub mod node;
pub mod strategy;

use star_river_core::custom_type::{NodeId, NodeName};

pub trait NodeCommandTrait: Send + Sync + 'static {
    fn node_id(&self) -> &NodeId;
    fn node_name(&self) -> &NodeName;
}

pub trait NodeResponseTrait: Send + Sync + 'static {}

pub trait StrategyCommandTrait: Send + Sync + 'static {}

pub trait StrategyResponseTrait: Send + Sync + 'static {}
