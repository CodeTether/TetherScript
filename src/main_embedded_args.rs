//! Argument parsing for embedded launchers.

#[derive(Clone, Default)]
pub(crate) struct EmbeddedArgs {
    pub fs_grant: Option<String>,
    pub full_access: bool,
    pub provider_grant: Option<String>,
    pub provider_key: Option<String>,
    pub provider_vault: Option<String>,
    pub rpc_grant: Option<String>,
    pub reload_source: Option<String>,
    pub script_args: Vec<String>,
}

pub(crate) fn parse(args: &[String]) -> Result<EmbeddedArgs, String> {
    let mut out = EmbeddedArgs::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--" => {
                out.script_args.extend(args[i + 1..].iter().cloned());
                break;
            }
            "--access-mode" => out.full_access = value(args, &mut i)? == "full",
            "--grant-fs" => out.fs_grant = Some(value(args, &mut i)?),
            "--grant-provider" => out.provider_grant = Some(endpoint(args, &mut i)?),
            "--grant-provider-key" => out.provider_key = Some(value(args, &mut i)?),
            "--grant-provider-vault" => out.provider_vault = Some(value(args, &mut i)?),
            "--grant-rpc" => out.rpc_grant = Some(value(args, &mut i)?),
            "--reload-source" => out.reload_source = Some(value(args, &mut i)?),
            other => out.script_args.push(other.to_string()),
        }
        i += 1;
    }
    Ok(out)
}

fn value(args: &[String], i: &mut usize) -> Result<String, String> {
    *i += 1;
    args.get(*i)
        .cloned()
        .ok_or_else(|| format!("{} requires an argument", args[*i - 1]))
}

fn endpoint(args: &[String], i: &mut usize) -> Result<String, String> {
    let url = value(args, i)?;
    if url.starts_with("http://") || url.starts_with("https://") {
        return Ok(url);
    }
    Err("--grant-provider endpoint must start with http:// or https://".into())
}
