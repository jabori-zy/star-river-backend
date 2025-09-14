use chrono::{DateTime, Utc};
use chrono_tz::{Asia::Shanghai, America::New_York};
use std::io;

fn user_input() -> i64 {
    println!("请输入时间戳(秒):");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().parse().unwrap()
}

pub fn main() {

    loop {
    // 这里改为用户输入
        let timestamp = user_input();
        let utc_datetime = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap();
        let shanghai_datetime = utc_datetime.with_timezone(&Shanghai);
        let newyork_datetime = utc_datetime.with_timezone(&New_York);


        println!("utc_datetime: {}", utc_datetime);
        println!("shanghai_datetime: {}", shanghai_datetime);
        println!("newyork_datetime: {}", newyork_datetime);
    }

}