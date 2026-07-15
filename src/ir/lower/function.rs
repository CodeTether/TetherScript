use crate::ast::Block as AstBlock;
use crate::ir::{Block, Function, Terminator};

use super::{block, builder::Builder, LowerError};

pub(super) fn lower(
    name: &str,
    params: &[String],
    body: &AstBlock,
) -> Result<Function, LowerError> {
    let mut builder = Builder::new(name, params);
    let returned = block::lower(&mut builder, body)?;
    Ok(Function {
        name: name.into(),
        params: builder.params,
        blocks: vec![Block {
            label: "entry".into(),
            instructions: builder.instructions,
            terminator: Terminator::Return(returned),
        }],
    })
}
