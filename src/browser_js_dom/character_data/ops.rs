use super::*;

pub(super) fn call(handle: &DomHandle, method: &str, args: &[JsValue]) -> Result<JsValue, String> {
    let old = state::read(handle);
    if method == "substringData" {
        return units::slice(
            &old,
            args::unsigned(args, 0),
            args::unsigned(args, 1),
            method,
        )
        .map(JsValue::String);
    }
    let next = match method {
        "appendData" => format!("{}{}", old, args::text(args, 0)),
        "deleteData" => units::replace(
            &old,
            args::unsigned(args, 0),
            args::unsigned(args, 1),
            "",
            method,
        )?,
        "insertData" => units::replace(
            &old,
            args::unsigned(args, 0),
            0,
            &args::text(args, 1),
            method,
        )?,
        "replaceData" => units::replace(
            &old,
            args::unsigned(args, 0),
            args::unsigned(args, 1),
            &args::text(args, 2),
            method,
        )?,
        _ => return Err(format!("CharacterData.{method}: unsupported method")),
    };
    state::write(handle, old, next);
    Ok(JsValue::Undefined)
}
