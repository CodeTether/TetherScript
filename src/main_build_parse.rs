//! Argument parser for `tetherscript build`.

pub(crate) struct BuildOpts {
    pub(crate) path: String,
    pub(crate) output: String,
}

pub(crate) fn parse(args: &[String]) -> Result<BuildOpts, String> {
    let mut path = None;
    let mut output = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err("__help__".into()),
            "-o" | "--output" => read_output(args, &mut i, &mut output)?,
            other => read_path(other, &mut path)?,
        }
        i += 1;
    }
    Ok(BuildOpts {
        path: path.ok_or("missing source file")?,
        output: output.ok_or("missing -o <output>")?,
    })
}

fn read_output(args: &[String], i: &mut usize, output: &mut Option<String>) -> Result<(), String> {
    *i += 1;
    let value = args.get(*i).ok_or("-o requires an output path")?;
    *output = Some(value.clone());
    Ok(())
}

fn read_path(arg: &str, path: &mut Option<String>) -> Result<(), String> {
    if arg.starts_with('-') {
        return Err(format!("unknown option '{}'", arg));
    }
    if path.replace(arg.to_string()).is_some() {
        return Err(format!("unexpected argument '{}'", arg));
    }
    Ok(())
}
