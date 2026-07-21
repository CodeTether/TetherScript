//! Decode package metadata from the in-tree JSON value model.

use std::path::PathBuf;

use crate::value::Value;

pub(super) fn manifest(value: &Value) -> Result<super::Manifest, String> {
    let root = super::decode_value::object(value, "manifest root")?;
    super::decode_value::reject_keys(&root, &["schema", "package"], "manifest")?;
    match root.get("schema") {
        Some(Value::Int(1)) => {}
        Some(Value::Int(version)) => return Err(format!("unsupported schema {version}")),
        _ => return Err("schema must be the integer 1".into()),
    }
    let package_value = root.get("package").ok_or("missing package object")?;
    let package = super::decode_value::object(package_value, "package")?;
    super::decode_value::reject_keys(&package, &["name", "version", "entry"], "package")?;
    let name = super::decode_value::string(&package, "name")?;
    let version = super::decode_value::string(&package, "version")?;
    let entry = PathBuf::from(super::decode_value::string(&package, "entry")?);
    super::validate::metadata(&name, &version, &entry)?;
    Ok(super::Manifest {
        name,
        version,
        entry,
    })
}
