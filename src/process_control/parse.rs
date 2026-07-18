#[cfg(windows)]
mod csv;
mod values;

pub(super) use values::value;

pub(super) fn processes(output: String) -> Result<Vec<(i64, String)>, String> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(process)
        .collect()
}

#[cfg(windows)]
fn process(line: &str) -> Result<(i64, String), String> {
    let fields = csv::parse(line)?;
    if fields.len() < 2 {
        return Err(format!("process_list: malformed tasklist row `{line}`"));
    }
    pid_name(&fields[1], fields[0].clone(), line)
}

#[cfg(not(windows))]
fn process(line: &str) -> Result<(i64, String), String> {
    let mut fields = line.trim().splitn(2, char::is_whitespace);
    let pid = fields.next().unwrap_or_default();
    let name = fields.next().unwrap_or_default().trim().to_string();
    pid_name(pid, name, line)
}

fn pid_name(pid: &str, name: String, line: &str) -> Result<(i64, String), String> {
    let pid = pid
        .parse()
        .map_err(|_| format!("process_list: invalid PID row `{line}`"))?;
    Ok((pid, name))
}
