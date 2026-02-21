// gate-test-concurrent_bikeshed
// Test that concurrent_bikeshed requires the feature gate.
//@ edition: 2021

fn main() {
    let _ = async {
        concurrent_bikeshed { //~ ERROR `concurrent_bikeshed` expression is experimental
            { },
            { },
        };
    };
}
