// Test basic concurrent_bikeshed usage with shared mutable access and return values.
//@ edition: 2021
//@ check-pass

#![feature(concurrent_bikeshed)]

async fn test_basic() {
    let mut x = 0i32;
    let (a, b) = concurrent_bikeshed {
        { x += 1; 42 },
        { x += 2; 99 },
    };
    assert_eq!(a, 42);
    assert_eq!(b, 99);
}

async fn test_unit_branches() {
    let mut x = 0i32;
    // Branches returning () should still work
    let ((), ()) = concurrent_bikeshed {
        { x += 1; },
        { x += 2; },
    };
}

async fn test_single_branch() {
    let mut x = 0i32;
    // Single branch returns value directly, not a 1-tuple
    let val: i32 = concurrent_bikeshed {
        { x += 1; 42 },
    };
    assert_eq!(val, 42);
}

fn main() {
    let _ = async {
        test_basic().await;
        test_unit_branches().await;
        test_single_branch().await;
    };
}
