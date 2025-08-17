/// 虚拟交易系统测试模块
/// 
/// 本模块包含对虚拟交易系统订单和仓位相关计算的全面测试
/// 
/// 测试覆盖范围：
/// 1. 订单相关计算：保证金、保证金率、强平价格
/// 2. 仓位相关计算：盈亏计算、多仓位管理
/// 3. 风险管理：保证金不足检查、强平场景
/// 4. 综合场景：完整交易流程、账户权益计算

pub mod order_calculation_test;
pub mod position_calculation_test; 
pub mod comprehensive_test;