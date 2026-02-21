// Test cases where the current analysis is overly restrictive.
// These programs are rejected by the pre-lowering AST check, but would
// actually be safe because the borrows don't span .await points.
// A proper borrowck-integrated solution would accept these.
//@ edition: 2021

#![feature(concurrent_bikeshed)]
#![allow(unused)]

async fn sleep() {}

// 1. Explicit &mut that does NOT span an .await point.
//    The borrow of `x` is created and dropped before the .await,
//    so there's no aliasing danger. But the current analysis rejects
//    ANY explicit borrow of a shared var in a branch with .await.
async fn borrow_before_await() {
    let mut x = 0i32;
    concurrent_bikeshed {
        {
            let r = &mut x; //~ ERROR shared variable `x` is explicitly borrowed
            *r += 1;
            // r is dropped here
            sleep().await;
        },
        { x += 1; },
    };
}

// 2. Shared reference (not &mut) before .await.
//    A shared `&x` that doesn't span .await is always safe.
async fn shared_ref_before_await() {
    let mut x = 0i32;
    concurrent_bikeshed {
        {
            let r = &x; //~ ERROR shared variable `x` is explicitly borrowed
            let _copy = *r;
            // r is dropped here
            sleep().await;
        },
        { x += 1; },
    };
}

// 3. Borrow inside a block scope that ends before .await.
//    The inner scope constrains the borrow's lifetime.
async fn scoped_borrow_before_await() {
    let mut x = 0i32;
    concurrent_bikeshed {
        {
            {
                let r = &mut x; //~ ERROR shared variable `x` is explicitly borrowed
                *r += 1;
            } // r dropped here, well before .await
            sleep().await;
        },
        { x += 1; },
    };
}

fn main() {}
