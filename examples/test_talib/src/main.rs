mod cycle_test;
mod momentum_test;
mod overlap_test;
mod pattern_recognition_test;
mod price_transform_test;
mod value;
mod volatility_test;
mod volume_test;

fn main() {
    println!("=== 技术指标测试套件 ===");

    // 运行所有分类的指标测试
    cycle_test::test_cycle_indicators();
    // momentum_test::test_momentum_indicators();
    overlap_test::test_overlap_indicators();
    pattern_recognition_test::test_pattern_recognition_indicators();
    price_transform_test::test_price_transform_indicators();
    volatility_test::test_volatility_indicators();
    volume_test::test_volume_indicators();
}
