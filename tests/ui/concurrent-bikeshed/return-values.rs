// Test that concurrent_bikeshed branches return values correctly.
//@ edition: 2021
//@ check-pass

#![feature(concurrent_bikeshed)]

async fn sleep() {}

async fn test_different_types() {
    // Branches can return different types in the tuple
    let (a, b): (i32, bool) = concurrent_bikeshed {
        { 42 },
        { true },
    };
    assert_eq!(a, 42);
    assert_eq!(b, true);
}

async fn test_with_await() {
    // Return values work when branches contain .await
    let (a, b) = concurrent_bikeshed {
        { sleep().await; 10 },
        { 20 },
    };
    assert_eq!(a, 10);
    assert_eq!(b, 20);
}

async fn test_three_branches() {
    let (a, b, c) = concurrent_bikeshed {
        { 1u8 },
        { 2u8 },
        { 3u8 },
    };
    assert_eq!(a, 1);
    assert_eq!(b, 2);
    assert_eq!(c, 3);
}

async fn test_string_values() {
    let (a, b) = concurrent_bikeshed {
        { String::from("hello") },
        { String::from("world") },
    };
    assert_eq!(a, "hello");
    assert_eq!(b, "world");
}

fn main() {
    let _ = async {
        test_different_types().await;
        test_with_await().await;
        test_three_branches().await;
        test_string_values().await;
    };
}
