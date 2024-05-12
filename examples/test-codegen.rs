use rust_async_queue::task;

#[rust_async_queue::task]
fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn main() {}
