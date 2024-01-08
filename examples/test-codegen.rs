use rust_async_queue::codegen::my_custom_attribute;

#[my_custom_attribute(a, b)]
fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn main() {}
