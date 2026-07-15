#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Mode {
    Tokens,
    Ast,
    Bytecode,
    BytecodeVisual,
    Ir,
}

pub(super) enum Parsed {
    Help,
    Run { mode: Mode, path: String },
}

pub(super) fn parse(args: &[String]) -> Result<Parsed, String> {
    let mut mode = None;
    let mut path = None;
    for argument in args {
        match argument.as_str() {
            "--help" | "-h" => return Ok(Parsed::Help),
            "--tokens" => mode = Some(Mode::Tokens),
            "--ast" => mode = Some(Mode::Ast),
            "--bytecode" => mode = Some(Mode::Bytecode),
            "--bytecode-visual" | "--bytecode-viz" | "--visual" => {
                mode = Some(Mode::BytecodeVisual);
            }
            "--ir" => mode = Some(Mode::Ir),
            option if option.starts_with('-') => {
                return Err(format!("unknown option '{option}'"));
            }
            value if path.is_none() => path = Some(value.to_string()),
            value => return Err(format!("unexpected argument '{value}'")),
        }
    }
    let path = path.ok_or_else(|| "missing source file".to_string())?;
    let mode = mode.ok_or_else(|| {
        "specify one of --tokens, --ast, --bytecode, --bytecode-visual, --ir".to_string()
    })?;
    Ok(Parsed::Run { mode, path })
}
