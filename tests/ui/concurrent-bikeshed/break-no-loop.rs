// Test that `break` inside concurrent_bikeshed without an enclosing loop is an error.
//@ edition: 2021

#![feature(concurrent_bikeshed)]

async fn sleep() {}

async fn bad_break() {
    concurrent_bikeshed {
        { break; },  //~ ERROR
        sleep().await,
    };
}

fn main() {}
