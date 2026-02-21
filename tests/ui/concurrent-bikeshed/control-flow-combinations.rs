// Test comprehensive valid control flow combinations inside concurrent_bikeshed.
//@ edition: 2021
//@ check-pass

#![feature(concurrent_bikeshed)]
#![allow(unused_mut, unreachable_code)]

async fn sleep() {}

// --- Inner loops: break/continue target the loop INSIDE the branch, not intercepted ---

async fn break_inner_loop() {
    let (a, b) = concurrent_bikeshed {
        {
            let mut sum = 0;
            for i in 0..10 {
                if i >= 3 { break; }
                sum += i;
            }
            sum // 0 + 1 + 2 = 3
        },
        {
            let mut v = Vec::new();
            let mut i = 0;
            loop {
                if i >= 3 { break; }
                v.push(i);
                i += 1;
            }
            v.len()
        },
    };
    assert_eq!(a, 3);
    assert_eq!(b, 3);
}

async fn continue_inner_loop() {
    let (a, _b) = concurrent_bikeshed {
        {
            let mut sum = 0;
            for i in 0..10 {
                if i % 2 == 0 { continue; }
                sum += i;
            }
            sum // 1+3+5+7+9 = 25
        },
        sleep().await,
    };
    assert_eq!(a, 25);
}

async fn break_with_value_inner_loop() {
    let (a, _b) = concurrent_bikeshed {
        {
            let result = loop {
                break 42;
            };
            result
        },
        sleep().await,
    };
    assert_eq!(a, 42);
}

async fn while_loop_in_branch() {
    let (a, _b) = concurrent_bikeshed {
        {
            let mut x = 0;
            while x < 5 {
                x += 1;
            }
            x
        },
        sleep().await,
    };
    assert_eq!(a, 5);
}

// --- Closures inside branches: return targets the closure, not the function ---

async fn return_from_closure_in_branch() {
    let (a, _b) = concurrent_bikeshed {
        {
            let f = || -> i32 {
                return 42;  // returns from closure, not function
            };
            f()
        },
        sleep().await,
    };
    assert_eq!(a, 42);
}

// --- Mixing different control flow types across branches ---

async fn return_and_break_different_branches() -> i32 {
    let mut count = 0;
    loop {
        count += 1;
        let (a, b) = concurrent_bikeshed {
            {
                if count >= 3 {
                    return 100;  // return from function
                }
                count
            },
            {
                if count >= 2 {
                    break;  // break from loop
                }
                sleep().await;
                0
            },
        };
        let _ = (a, b);
    }
    count
}

async fn return_and_continue_different_branches() -> i32 {
    let mut sum = 0;
    let mut i = 0;
    loop {
        i += 1;
        if i > 5 { break; }
        let (a, _b) = concurrent_bikeshed {
            {
                if sum > 10 {
                    return sum;  // return from function
                }
                i
            },
            {
                if i % 2 == 0 {
                    continue;  // continue the loop
                }
                sleep().await;
            },
        };
        sum += a;
    }
    sum
}

// --- ? operator combinations ---

async fn try_in_multiple_branches() -> Result<(i32, i32), String> {
    let (a, b) = concurrent_bikeshed {
        {
            let x: Result<i32, String> = Ok(10);
            x?
        },
        {
            let y: Result<i32, String> = Ok(20);
            y?
        },
    };
    Ok((a, b))
}

async fn try_and_return_mixed() -> Result<i32, String> {
    let _ = concurrent_bikeshed {
        {
            let x: Result<(), String> = Err("fail".into());
            x?;  // propagate error via ?
        },
        sleep().await,
    };
    Ok(42)
}

// --- Three branches with control flow ---

async fn three_branches_with_return() -> i32 {
    let _ = concurrent_bikeshed {
        {
            return 1;
        },
        sleep().await,
        sleep().await,
    };
    0
}

async fn three_branches_mixed_control_flow() -> i32 {
    let mut iterations = 0;
    loop {
        iterations += 1;
        if iterations > 3 { break; }
        let _ = concurrent_bikeshed {
            {
                if iterations == 2 {
                    continue;  // skip this iteration
                }
                sleep().await;
            },
            {
                if iterations == 3 {
                    return 99;  // return from function
                }
                sleep().await;
            },
            sleep().await,
        };
    }
    0
}

// --- Control flow only in some branches (others are pure values) ---

async fn return_in_one_of_three(mut cond: bool) -> i32 {
    let (a, b, c) = concurrent_bikeshed {
        {
            if cond { return 0; }
            1
        },
        2,
        3,
    };
    a + b + c
}

async fn break_in_one_of_three() -> i32 {
    let mut total = 0;
    loop {
        let (a, b, c) = concurrent_bikeshed {
            1,
            {
                total += 1;
                if total >= 2 { break; }
                sleep().await;
                2
            },
            3,
        };
        total += a + b + c;
    }
    total
}

// --- Nested concurrent_bikeshed ---

async fn nested_concurrent_bikeshed() -> i32 {
    let (a, b) = concurrent_bikeshed {
        {
            let (x, y) = concurrent_bikeshed {
                1,
                2,
            };
            x + y
        },
        {
            let (x, y) = concurrent_bikeshed {
                10,
                20,
            };
            x + y
        },
    };
    a + b // 3 + 30 = 33
}

async fn nested_with_return() -> i32 {
    let _ = concurrent_bikeshed {
        {
            let _ = concurrent_bikeshed {
                {
                    return 42;  // return from outermost function
                },
                sleep().await,
            };
        },
        sleep().await,
    };
    0
}

// --- No control flow at all (regression) ---

async fn pure_values() -> (i32, i32, i32) {
    concurrent_bikeshed {
        1,
        2,
        3,
    }
}

async fn pure_values_with_await() -> (i32, i32) {
    concurrent_bikeshed {
        { sleep().await; 10 },
        { sleep().await; 20 },
    }
}

// --- Single branch with control flow ---

async fn single_branch_return() -> i32 {
    let _ = concurrent_bikeshed {
        { return 42; },
    };
    0
}

async fn single_branch_break() -> i32 {
    let mut count = 0;
    loop {
        let _ = concurrent_bikeshed {
            {
                count += 1;
                if count >= 3 { break; }
                sleep().await;
            },
        };
    }
    count
}

fn main() {
    let _ = async {
        break_inner_loop().await;
        continue_inner_loop().await;
        break_with_value_inner_loop().await;
        while_loop_in_branch().await;
        return_from_closure_in_branch().await;
        assert_eq!(return_and_break_different_branches().await, 2);
        try_in_multiple_branches().await.unwrap();
        assert!(try_and_return_mixed().await.is_err());
        assert_eq!(three_branches_with_return().await, 1);
        assert_eq!(return_in_one_of_three(true).await, 0);
        assert_eq!(return_in_one_of_three(false).await, 6);
        nested_concurrent_bikeshed().await;
        assert_eq!(nested_with_return().await, 42);
        assert_eq!(pure_values().await, (1, 2, 3));
        pure_values_with_await().await;
        assert_eq!(single_branch_return().await, 42);
        assert_eq!(single_branch_break().await, 3);
    };
}
