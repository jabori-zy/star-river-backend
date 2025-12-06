use std::sync::atomic::AtomicI32;

pub static TRANSACTION_ID_COUNTER: AtomicI32 = AtomicI32::new(0);
pub static ORDER_ID_COUNTER: AtomicI32 = AtomicI32::new(0);
pub static POSITION_ID_COUNTER: AtomicI32 = AtomicI32::new(0);
