// Test that shared variables cannot be explicitly borrowed in branches with .await.
//@ edition: 2021

#![feature(concurrent_bikeshed)]

async fn sleep() {}

async fn test_shared_borrow() {
    let mut x = 0i32;
    concurrent_bikeshed {
        { let r = &mut x; sleep().await; *r += 1; }, //~ ERROR shared variable `x` is explicitly borrowed
        { x += 1; },
    };
}

fn main() {
    let _ = async {
        test_shared_borrow().await;
    };
}
