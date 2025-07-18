use crate::indicator_engine::talib_bindings::*;
use super::TALib;
use crate::indicator_engine::talib_error::TalibError;

impl TALib {
    /// RSI - 相对强弱指数
    /// 这个方法保留用于向后兼容，新代码建议使用通用接口
    pub fn relative_strength_index(data: &[f64], period: i32) -> Result<Vec<f64>, TalibError> {
        use super::indicators::calculate_rsi;
        calculate_rsi(data, period)
    }
}