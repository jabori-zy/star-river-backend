use star_river_core::{
    custom_type::{Balance, Leverage, Margin, MarginRatio, Price},
    position::PositionSide,
};

pub struct Formula;

impl Formula {
    // Calculate margin
    pub fn calculate_margin(leverage: Leverage, price: Price, quantity: f64) -> Margin {
        // Calculate required initial margin
        // Margin = open price * position quantity / leverage
        let margin = price * quantity / leverage as f64;
        margin
    }

    // Calculate margin ratio
    pub fn calculate_margin_ratio(current_balance: Balance, leverage: Leverage, price: Price, quantity: f64) -> MarginRatio {
        // Margin ratio = margin / margin balance
        let margin_ratio = Self::calculate_margin(leverage, price, quantity) / current_balance;
        margin_ratio
    }

    // Calculate liquidation price
    pub fn calculate_force_price(position_side: &PositionSide, leverage: Leverage, price: Price, quantity: f64) -> Price {
        // Calculate liquidation price
        // Liquidation price = open price - margin / position quantity
        let force_price = match position_side {
            PositionSide::Long => price - Self::calculate_margin(leverage, price, quantity) / quantity, // Long liquidation price: open price - margin / position quantity
            PositionSide::Short => price + Self::calculate_margin(leverage, price, quantity) / quantity, // Short liquidation price: open price + margin / position quantity
        };
        force_price
    }
}
