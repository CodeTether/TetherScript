//! Scope attenuation helpers for computer authorities.

use std::collections::HashSet;

use crate::value::Value;

use super::authority::ComputerAuthority;

pub(crate) fn scopes(
    current: &ComputerAuthority,
    next: &mut ComputerAuthority,
    value: &Value,
) -> Result<(), String> {
    let requested = match value {
        Value::Str(s) => vec![(**s).clone()],
        Value::List(xs) => xs
            .borrow()
            .iter()
            .map(|v| match v {
                Value::Str(s) => Ok((**s).clone()),
                other => Err(format!(
                    "computer.narrow scopes: expected str, got {}",
                    other.type_name()
                )),
            })
            .collect::<Result<Vec<_>, _>>()?,
        other => {
            return Err(format!(
                "computer.narrow scopes: expected str/list, got {}",
                other.type_name()
            ))
        }
    };
    let allowed: HashSet<String> = requested.into_iter().collect();
    if !allowed.is_subset(&current.allowed_scopes) {
        return Err("computer.narrow: scopes must be subset of current scopes".into());
    }
    next.allowed_scopes = allowed;
    Ok(())
}
