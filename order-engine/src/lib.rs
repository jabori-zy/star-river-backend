use exchange_client::ExchangeClient;
use types::order::Order;
use types::market::Exchange;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use event_center::EventPublisher;
use event_center::Event;
use exchange_client::ExchangeManager;
use event_center::command_event::CommandEvent;
use event_center::command_event::OrderEngineCommand;
use event_center::command_event::CreateOrderParams;
use tokio::sync::mpsc;
use types::order::OrderType;
use types::order::OrderSide;

pub struct OrderEngineState {
    pub orders: HashMap<Exchange, Order>,
    

}

pub struct OrderEngine {
    pub state: Arc<RwLock<OrderEngineState>>,
    exchange_manager: Arc<Mutex<ExchangeManager>>,
    // 事件相关
    event_publisher: EventPublisher,
    command_event_receiver: broadcast::Receiver<Event>,
    response_event_receiver: broadcast::Receiver<Event>,
}

impl OrderEngine {
    pub fn new(
        command_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
        event_publisher: EventPublisher,
        exchange_manager: Arc<Mutex<ExchangeManager>>,
    ) -> Self {
        OrderEngine {
            exchange_manager: exchange_manager,
            event_publisher: event_publisher,
            command_event_receiver: command_event_receiver,
            response_event_receiver: response_event_receiver,
            state: Arc::new(RwLock::new(OrderEngineState {
                orders: HashMap::new(),
            })),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
        // 监听事件
        self.listen(internal_tx).await?;
        // 处理事件
        self.handle_events(internal_rx, self.state.clone(), self.exchange_manager.clone()).await?;
        Ok(())
    }

    async fn listen(&self, internal_tx: mpsc::Sender<Event>) -> Result<(), String> {
        tracing::info!("订单引擎启动成功, 开始监听...");
        let mut response_receiver = self.response_event_receiver.resubscribe();
        let mut command_receiver = self.command_event_receiver.resubscribe();
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(event) = response_receiver.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                    Ok(event) = command_receiver.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                }
            }
        });
        Ok(())
    }

    async fn handle_events(&self, mut internal_rx: mpsc::Receiver<Event>, state: Arc<RwLock<OrderEngineState>>, exchange_manager: Arc<Mutex<ExchangeManager>>) -> Result<(), String> {
        let event_publisher = self.event_publisher.clone();
        tokio::spawn(async move {
            loop {
                let event = internal_rx.recv().await.unwrap();
                match event {
                    Event::Command(command_event) => {
                        match command_event {
                            CommandEvent::OrderEngine(OrderEngineCommand::CreateOrder(params)) => {
                                Self::create_order(state.clone(), params, event_publisher.clone(), exchange_manager.clone()).await.unwrap();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        });
        Ok(())
    }

    async fn create_order(state: Arc<RwLock<OrderEngineState>>, params: CreateOrderParams, event_publisher: EventPublisher, exchange_manager: Arc<Mutex<ExchangeManager>>) -> Result<(), String> {
        tracing::info!("订单引擎收到创建订单命令: {:?}", params);
        let exchange = exchange_manager.lock().await.get_exchange(&params.order_request.exchange).await.unwrap();
        let order_request = params.order_request;
        let order = exchange.send_order(order_request).await.unwrap();
        Ok(())
    }
}





