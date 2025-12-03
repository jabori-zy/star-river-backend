use super::VirtualTradingSystemContext;

impl<E> VirtualTradingSystemContext<E> where E: Clone + Send + Sync + 'static {}
