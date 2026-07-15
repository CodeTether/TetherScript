use crate::ir::{self, Module};

use super::fixture::empty_function;

#[test]
fn rejects_duplicate_function_names() {
    let function = empty_function("same");
    let module = Module {
        functions: vec![function.clone(), function],
    };
    assert_eq!(
        ir::verify(&module).unwrap_err().to_string(),
        "invalid module: duplicate function `same`"
    );
}
