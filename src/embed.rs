//! Embedded script trailer support for standalone CLI launchers.

use std::fs;
use std::io::Write;
use std::path::Path;

const MAGIC: &[u8] = b"\nTETHERSCRIPT_EMBEDDED_SOURCE_V1\0";
const LEN_BYTES: usize = 8;

pub(crate) fn read_current() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    read_from(&exe).ok().flatten()
}

pub(crate) fn write_launcher(source: &str, output: &Path) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("find current exe: {e}"))?;
    let mut bytes = fs::read(&exe).map_err(|e| format!("read {}: {e}", exe.display()))?;
    bytes.extend_from_slice(source.as_bytes());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&(source.len() as u64).to_le_bytes());
    let mut file =
        fs::File::create(output).map_err(|e| format!("create {}: {e}", output.display()))?;
    file.write_all(&bytes)
        .map_err(|e| format!("write {}: {e}", output.display()))?;
    crate::embed_perm::make_executable(output)?;
    Ok(())
}

fn read_from(path: &Path) -> Result<Option<String>, String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if bytes.len() < MAGIC.len() + LEN_BYTES {
        return Ok(None);
    }
    let len_at = bytes.len() - LEN_BYTES;
    let len = u64::from_le_bytes(bytes[len_at..].try_into().unwrap()) as usize;
    let magic_at = len_at
        .checked_sub(MAGIC.len())
        .ok_or("bad embedded trailer")?;
    if &bytes[magic_at..len_at] != MAGIC {
        return Ok(None);
    }
    let source_at = magic_at
        .checked_sub(len)
        .ok_or("bad embedded source length")?;
    String::from_utf8(bytes[source_at..magic_at].to_vec())
        .map(Some)
        .map_err(|e| format!("embedded source is not utf-8: {e}"))
}
