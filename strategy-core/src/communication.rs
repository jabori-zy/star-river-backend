pub mod node;
pub mod strategy;

use star_river_core::custom_type::NodeId;


pub trait NodeCommandTrait: Send + Sync + 'static {
    fn node_id(&self) -> &NodeId;
}


pub trait NodeResponseTrait: Send + Sync + 'static {}



pub trait StrategyCommandTrait: Send + Sync + 'static {}



pub trait StrategyResponseTrait: Send + Sync + 'static {}