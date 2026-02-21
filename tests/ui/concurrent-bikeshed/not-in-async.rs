// Test that concurrent_bikeshed errors when used outside async context.

#![feature(concurrent_bikeshed)]

fn main() {
    concurrent_bikeshed { //~ ERROR `concurrent_bikeshed` can only be used inside `async` functions and blocks
        { },
        { },
    };
}
