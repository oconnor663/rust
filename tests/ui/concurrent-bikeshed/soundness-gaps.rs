// Test cases that demonstrate genuine unsoundness in the current implementation.
// These programs compile today but exhibit undefined behavior: the raw-pointer
// desugaring bypasses borrowck, allowing aliasing &mut references to the same
// memory across .await suspension points.
//
// A proper borrowck-integrated solution would reject all of these.
//@ edition: 2021
//@ check-pass

#![feature(concurrent_bikeshed)]
#![allow(unused, dangerous_implicit_autorefs)]

async fn sleep() {}

// 1. Both branches hold &mut to the same tuple field across .await.
//    The analysis checks for `&mut pair` but not `&mut pair.0`, so
//    these aliasing borrows slip through.
async fn tuple_field_aliasing() {
    let mut pair = (1i32, 2i32);
    concurrent_bikeshed {
        {
            let r = &mut pair.0;
            sleep().await;
            *r += 10;
        },
        {
            let r = &mut pair.0;
            sleep().await;
            *r += 20;
        },
    };
}

// 2. Both branches hold &mut to the same array element across .await.
//    Same pattern as above — `&mut arr[0]` is not caught by the analysis.
async fn array_element_aliasing() {
    let mut arr = [1i32, 2];
    concurrent_bikeshed {
        {
            let r = &mut arr[0];
            sleep().await;
            *r += 10;
        },
        {
            let r = &mut arr[0];
            sleep().await;
            *r += 20;
        },
    };
}

// 3. Both branches call .first_mut() on a Vec, holding &mut across .await.
//    The method call creates an implicit &mut borrow that the analysis
//    doesn't see (it only looks for ExprKind::AddrOf).
async fn vec_first_mut_aliasing() {
    let mut v = vec![0];
    concurrent_bikeshed {
        {
            let x = v.first_mut().unwrap();
            sleep().await;
            *x += 1;
        },
        {
            let x = v.first_mut().unwrap();
            sleep().await;
            *x += 1;
        },
    };
}

fn main() {}
