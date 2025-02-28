// tests/test_solution.rs

use solution::run;

#[test]
fn test_hello_output() {
    let result = run();
    assert_eq!(result, "Hello Rust Lab PJATK");
}
