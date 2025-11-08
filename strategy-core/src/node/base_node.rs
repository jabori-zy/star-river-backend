use super::context_trait::NodeMetaDataExt;
use super::node_trait::NodeContextAccessor;

use std::{
    fmt::Debug,
    sync::Arc,
};

// third-party
use tokio::sync::RwLock;


#[derive(Debug)]
pub struct NodeBase<C>
where C: NodeMetaDataExt,
{
    pub context: Arc<RwLock<C>>,
}

impl<C> NodeBase<C>
where
    C: NodeMetaDataExt,
{
    /// 创建新的节点基础实例
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
            context: Arc::clone(&self.context), // 克隆 Arc（引用计数+1），共享底层数据
        }
    }
}

// 为 NodeBase 实现 NodeContextAccessor trait
impl<C> NodeContextAccessor for NodeBase<C>
where
    C: NodeMetaDataExt,
{
    type Context = C;

    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        &self.context
    }
}