use std::ops::{Deref, DerefMut};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::{StreamExt, stream::select_all};
use key::KeyTrait;
use star_river_event::backtest_strategy::node_event::KlineNodeEvent;
use tokio::sync::watch;
use tokio_stream::wrappers::BroadcastStream;
use virtual_trading::{
    Vts, VtsContext,
    vts_trait::{VTSEventHandler, VTSEventListener, VtsCtxAccessor},
};

use crate::node::node_event::BacktestNodeEvent;

#[derive(Debug)]
pub struct BacktestVtsContext {
    inner: VtsContext<BacktestNodeEvent>,
}

impl Deref for BacktestVtsContext {
    type Target = VtsContext<BacktestNodeEvent>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for BacktestVtsContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug)]
pub struct BacktestVts {
    inner: Vts<BacktestVtsContext>,
}

impl Deref for BacktestVts {
    type Target = Vts<BacktestVtsContext>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for BacktestVts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl BacktestVts {
    pub fn new(context: BacktestVtsContext) -> Self {
        Self { inner: Vts::new(context) }
    }

    pub async fn start(&self) {
        tracing::info!("[BacktestVts] ready to start virtual trading system...");
        // 1. listen kline node events
        self.listen_kline_node_events().await;
        // 2. listen vts command
        self.listen_vts_command().await;
        tracing::info!("[BacktestVts] virtual trading system started");
    }
}

impl BacktestVtsContext {
    pub fn new(strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>) -> Self {
        Self {
            inner: VtsContext::new(strategy_time_watch_rx),
        }
    }
}

// ============================================================================
// VTSAccessor implementation for BacktestVts
// ============================================================================

impl VtsCtxAccessor for BacktestVts {
    type Context = BacktestVtsContext;

    fn context(&self) -> &std::sync::Arc<tokio::sync::RwLock<Self::Context>> {
        self.inner.context()
    }
}

// ============================================================================
// VTSEventHandler implementation for BacktestVtsContext
// ============================================================================

#[async_trait]
impl VTSEventHandler for BacktestVtsContext {
    type KlineEvent = BacktestNodeEvent;

    async fn handle_kline_event(&mut self, event: BacktestNodeEvent) {
        match event {
            BacktestNodeEvent::KlineNode(kline_event) => match kline_event {
                KlineNodeEvent::KlineUpdate(event) => {
                    if event.is_min_interval {
                        self.handle_kline_update(
                            event.kline_key.exchange().clone(),
                            event.kline_key.symbol().clone(),
                            event.kline.clone(),
                        );
                    }
                }
            },
            _ => {}
        }
    }
}

// ============================================================================
// VTSEventListener implementation for BacktestVts
// ============================================================================

#[async_trait]
impl VTSEventListener for BacktestVts {
    async fn listen_kline_node_events(&self) {
        let (cancel_token, streams) = self
            .with_ctx_read(|ctx| {
                if ctx.kline_node_event_receiver().is_empty() {
                    tracing::warn!("[BacktestVts] no kline node event receiver");
                    return (ctx.cancel_token(), vec![]);
                }

                // Create streams for receiving K-line node events
                let streams: Vec<_> = ctx
                    .kline_node_event_receiver()
                    .iter()
                    .map(|receiver| BroadcastStream::new(receiver.resubscribe()))
                    .collect();

                let cancel_token = ctx.cancel_token();

                (cancel_token, streams)
            })
            .await;

        if streams.is_empty() {
            return;
        }

        let mut combined_stream = select_all(streams);
        let context = self.context().clone();

        // Listen to K-line node events
        tokio::spawn(async move {
            loop {
                if cancel_token.is_cancelled() {
                    tracing::info!("[BacktestVts] kline node event listen task cancelled");
                    break;
                }

                // Receive messages
                match combined_stream.next().await {
                    Some(Ok(event)) => {
                        // Handle K-line node event
                        let mut context_guard = context.write().await;
                        context_guard.handle_kline_event(event).await;
                    }
                    Some(Err(e)) => {
                        tracing::error!("[BacktestVts] receive kline node event error: {}", e);
                    }
                    None => {
                        tracing::warn!("[BacktestVts] all kline node event streams closed");
                        break;
                    }
                }
            }
        });
    }

    async fn listen_vts_command(&self) {
        let (command_receiver, cancel_token) = self
            .with_ctx_read(|ctx| {
                let receiver = ctx.get_command_receiver();
                let cancel_token = ctx.cancel_token();
                (receiver, cancel_token)
            })
            .await;

        let context = self.context().clone();

        // Listen to VTS commands
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // If cancel signal is triggered, abort task
                    _ = cancel_token.cancelled() => {
                        tracing::info!("[BacktestVts] VTS command listen task cancelled");
                        break;
                    }

                    _ = async {
                        let mut command_receiver_guard = command_receiver.lock().await;

                        if let Some(received_command) = command_receiver_guard.recv().await {
                            let mut context_guard = context.write().await;
                            context_guard.handle_command(received_command).await;
                        }
                    } => {}
                }
            }
        });
    }
}
