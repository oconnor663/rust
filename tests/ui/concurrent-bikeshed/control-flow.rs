// Test that return, ?, break, and continue work inside concurrent_bikeshed branches.
//@ edition: 2021
//@ check-pass

#![feature(concurrent_bikeshed)]
#![allow(unused_mut)]

async fn sleep() {}

// Test `return` from inside a branch
async fn test_early_return(mut flag: bool) -> i32 {
    let (_a, _b) = concurrent_bikeshed {
        {
            if flag {
                return 42;
            }
            sleep().await;
        },
        sleep().await,
    };
    99
}

// Test `return` with a value
async fn test_return_value() -> String {
    let _ = concurrent_bikeshed {
        {
            return String::from("early");
        },
        sleep().await,
    };
    String::from("late")
}

// Test `?` operator (try) from inside a branch
async fn test_try_operator(mut fail: bool) -> Result<i32, String> {
    let (_a, _b) = concurrent_bikeshed {
        {
            if fail {
                Err("oops".to_string())?;
            }
            10
        },
        20,
    };
    Ok(100)
}

// Test `break` from inside a branch (in an enclosing loop)
async fn test_break_from_branch() -> i32 {
    let mut sum = 0;
    loop {
        let (a, _b) = concurrent_bikeshed {
            {
                sum += 1;
                if sum >= 3 {
                    break;
                }
                sleep().await;
            },
            sleep().await,
        };
        let _ = a;
    }
    sum
}

// Test `break` with a value from inside a branch
async fn test_break_with_value() -> i32 {
    let result = loop {
        let (_a, _b) = concurrent_bikeshed {
            {
                sleep().await;
                break 42;
            },
            sleep().await,
        };
    };
    result
}

// Test `continue` from inside a branch
async fn test_continue_from_branch() -> i32 {
    let mut sum = 0;
    let mut iterations = 0;
    loop {
        iterations += 1;
        if iterations > 5 {
            break;
        }
        let _ = concurrent_bikeshed {
            {
                if iterations % 2 == 0 {
                    continue;
                }
                sum += iterations;
                sleep().await;
            },
            sleep().await,
        };
    }
    sum // 1 + 3 + 5 = 9
}

// Test that control flow works without other branches needing control flow
async fn test_return_only_in_one_branch(mut flag: bool) -> i32 {
    let (a, b) = concurrent_bikeshed {
        {
            if flag {
                return 0;
            }
            1
        },
        2,
    };
    a + b
}

// Test no control flow at all (regression: should still work)
async fn test_no_control_flow() -> (i32, i32) {
    concurrent_bikeshed {
        { sleep().await; 10 },
        20,
    }
}

fn main() {
    let _ = async {
        assert_eq!(test_early_return(true).await, 42);
        assert_eq!(test_early_return(false).await, 99);
        assert_eq!(test_return_value().await, "early");
        assert_eq!(test_try_operator(false).await, Ok(100));
        assert!(test_try_operator(true).await.is_err());
        assert_eq!(test_break_from_branch().await, 3);
        assert_eq!(test_break_with_value().await, 42);
        assert_eq!(test_continue_from_branch().await, 9);
        assert_eq!(test_return_only_in_one_branch(true).await, 0);
        assert_eq!(test_return_only_in_one_branch(false).await, 3);
        assert_eq!(test_no_control_flow().await, (10, 20));
    };
}
