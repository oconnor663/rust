// Test that concurrent_bikeshed branches return values correctly.
//@ edition: 2021
//@ check-pass

#![feature(concurrent_bikeshed)]

async fn sleep() {}

async fn test_different_types() {
    // Branches can return different types in the tuple
    // Also tests brace-free expression syntax
    let (a, b): (i32, bool) = concurrent_bikeshed {
        42,
        true,
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
    // Brace-free with function calls
    let (a, b) = concurrent_bikeshed {
        String::from("hello"),
        String::from("world"),
    };
    assert_eq!(a, "hello");
    assert_eq!(b, "world");
}

async fn test_mixed_syntax() {
    let mut x = 0i32;
    // Mix of brace-free expressions and block expressions
    let (a, b) = concurrent_bikeshed {
        { x += 1; 42 },
        sleep().await,
    };
    assert_eq!(a, 42);
    assert_eq!(b, ());
}

fn main() {
    let _ = async {
        test_different_types().await;
        test_with_await().await;
        test_three_branches().await;
        test_string_values().await;
        test_mixed_syntax().await;
    };
}
