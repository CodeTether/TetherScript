//! Builtin function registration.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::{Env, Value};

use super::super::super::pure_native;
use super::decode::value_to_rules;
use super::encode::rules_to_value;

use self::arg::str_arg;
use self::matchers::{block_value, cosmetic_list};

#[path = "register/arg.rs"]
mod arg;
#[path = "register/matchers.rs"]
mod matchers;

pub(super) fn all(env: &Rc<RefCell<Env>>) {
    let mut e = env.borrow_mut();
    e.define("adblock_parse", parse_builtin(), false);
    e.define("adblock_should_block", block_builtin(), false);
    e.define("adblock_cosmetic_selectors", cosmetic_builtin(), false);
    e.define("adblock_rule_count", count_builtin(), false);
}

fn parse_builtin() -> Value {
    pure_native("adblock_parse", Some(1), |args| {
        Ok(rules_to_value(&super::super::adblock::parse::parse_list(
            &str_arg(&args[0], "text")?,
        )))
    })
}

fn block_builtin() -> Value {
    pure_native("adblock_should_block", Some(3), |args| {
        let rules = value_to_rules(&args[0])?;
        Ok(Value::Bool(block_value(
            &rules,
            &str_arg(&args[1], "url")?,
            &str_arg(&args[2], "source_domain")?,
        )))
    })
}

fn cosmetic_builtin() -> Value {
    pure_native("adblock_cosmetic_selectors", Some(2), |args| {
        Ok(cosmetic_list(
            &value_to_rules(&args[0])?,
            &str_arg(&args[1], "domain")?,
        ))
    })
}

fn count_builtin() -> Value {
    pure_native("adblock_rule_count", Some(1), |args| {
        Ok(Value::Int(value_to_rules(&args[0])?.len() as i64))
    })
}
