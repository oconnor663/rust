// Test that exclusive variables (used in only one branch) work fine.
//@ edition: 2021
//@ check-pass

#![feature(concurrent_bikeshed)]

async fn test_exclusive() {
    let mut x = 0i32;
    let mut y = 0i32;
    concurrent_bikeshed {
        { x += 1; },
        { y += 1; },
    };
}

fn main() {
    let _ = async {
        test_exclusive().await;
    };
}
