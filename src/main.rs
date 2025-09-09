fn main() {
    let result = star_river::start();
    if result.is_err() {
        println!("Error: {}", result.err().unwrap());
    }
}
