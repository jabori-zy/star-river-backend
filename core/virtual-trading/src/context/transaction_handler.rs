use super::VtsContext;

impl<E> VtsContext<E> where E: Clone + Send + Sync + 'static {}
