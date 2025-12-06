use std::{fmt::Debug, sync::Arc};

// third-party
use tokio::sync::RwLock;

use super::{context_trait::NodeMetaDataExt, node_trait::NodeContextAccessor};

#[derive(Debug)]
pub struct NodeBase<C>
where
    C: NodeMetaDataExt,
{
    pub context: Arc<RwLock<C>>,
}

impl<C> NodeBase<C>
where
    C: NodeMetaDataExt,
{
    /// Create new node base instance
    pub fn new(context: C) -> Self {
        Self {
            context: Arc::new(RwLock::new(context)),
        }
    }
}

impl<C> Clone for NodeBase<C>
where
    C: NodeMetaDataExt,
{
    fn clone(&self) -> Self {
        Self {
            context: Arc::clone(&self.context), // Clone Arc (ref count +1), share underlying data
        }
    }
}

// Implement NodeContextAccessor trait for NodeBase
impl<C> NodeContextAccessor for NodeBase<C>
where
    C: NodeMetaDataExt,
{
    type Context = C;

    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        &self.context
    }
}
