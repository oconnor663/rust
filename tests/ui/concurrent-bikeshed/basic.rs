// Test basic concurrent_bikeshed usage with shared mutable access.
//@ edition: 2021
//@ check-pass

#![feature(concurrent_bikeshed)]

async fn test_basic() {
    let mut x = 0i32;
    concurrent_bikeshed {
        { x += 1; },
        { x += 2; },
    };
}

fn main() {
    let _ = async {
        test_basic().await;
    };
}
