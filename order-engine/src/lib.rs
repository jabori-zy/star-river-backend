use exchange_client::Trading;
use types::order::Order;
use types::market::Exchange;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use exchange_client::metatrader5::MetaTrader5;
use event_center::EventPublisher;


pub struct OrderEngineState {
    pub exchanges: HashMap<Exchange, Box<dyn Trading>>,
    pub orders: HashMap<Exchange, Order>,

}

pub struct OrderEngine {
    pub state: Arc<RwLock<OrderEngineState>>,
    // 事件相关
}

impl OrderEngine {
    pub fn new() -> Self {
        OrderEngine {
            state: Arc::new(RwLock::new(OrderEngineState {
                exchanges: HashMap::new(),
                orders: HashMap::new(),
            })),
        }
    }

    pub fn register_exchange(exchange: Exchange, state: Arc<RwLock<OrderEngineState>>, event_publisher: EventPublisher) -> Result<(), String> {
        match exchange {
            Exchange::Metatrader5 => {
                let mt5 = MetaTrader5::new(event_publisher);
                state.write().unwrap().exchanges.insert(exchange.clone(), Box::new(mt5));
                tracing::info!("{}交易所注册成功!", exchange);
                Ok(())
            }
            _ => {
                panic!("不支持的交易所");
            }
        }
    }
}






