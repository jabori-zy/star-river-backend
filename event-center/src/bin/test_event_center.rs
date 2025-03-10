use event_center::{Channel, Event};
use types::market::{Kline, Exchange, KlineInterval};
use event_center::market_event::MarketEvent;
use event_center::market_event::KlineEventInfo;
use utils::get_utc8_timestamp_millis;
use event_center::EventCenter;

fn main() {
    // 获取所有通道

    let channels = Channel::get_all_channels();
    println!("{:?}", channels);

    let kline = Kline {
        timestamp: 1717977600000,
        open: 10000.0,
        high: 10000.0,
        low: 10000.0,
        close: 10000.0,
        volume: 10000.0,
    };

    let kline_update_event_config = KlineEventInfo {
        exchange: Exchange::Binance,
        symbol: "BTCUSDT".to_string(),
        interval: KlineInterval::Minutes1,
        kline,
        event_timestamp: get_utc8_timestamp_millis(),
    };

    let kline_update_event = MarketEvent::KlineUpdate(kline_update_event_config);

    let event = Event::Market(kline_update_event);

    let event_center = EventCenter::new();
    event_center.publish(event.clone()).unwrap();

    println!("{:?}", event);
}

