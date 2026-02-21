// Test that invalid control flow inside concurrent_bikeshed branches produces errors.
//@ edition: 2021

#![feature(concurrent_bikeshed)]

async fn sleep() {}

// --- break without enclosing loop ---

async fn break_no_loop() {
    concurrent_bikeshed {
        { break; },     //~ ERROR `break` inside `async` block
        sleep().await,
    };
}

async fn break_value_no_loop() {
    concurrent_bikeshed {
        { break 42; },  //~ ERROR `break` inside `async` block
        sleep().await,
    };
}

// --- continue without enclosing loop ---

async fn continue_no_loop() {
    concurrent_bikeshed {
        { continue; },  //~ ERROR `continue` inside `async` block
        sleep().await,
    };
}

// --- labeled break targeting outer loop across async boundary ---
// (Labels can't cross async block boundaries)

async fn labeled_break() {
    'outer: loop {
        concurrent_bikeshed {
            { break 'outer; },  //~ ERROR `break` inside `async` block
            sleep().await,
        };
    }
}

// NOTE: `continue 'outer` across an async boundary triggers a pre-existing
// compiler ICE in rustc_mir_build, so we don't test it here.

// --- break/continue in both branches, no loop ---

async fn both_branches_break() {
    concurrent_bikeshed {
        { break; },     //~ ERROR `break` inside `async` block
        { break; },     //~ ERROR `break` inside `async` block
    };
}

async fn both_branches_continue() {
    concurrent_bikeshed {
        { continue; },  //~ ERROR `continue` inside `async` block
        { continue; },  //~ ERROR `continue` inside `async` block
    };
}

fn main() {}
