use crate::ir::Constant;

pub(super) fn render(value: &Constant) -> String {
    match value {
        Constant::Int(value) => format!("const.int {value}"),
        Constant::Float(value) => format!("const.float {value}"),
        Constant::Bool(value) => format!("const.bool {value}"),
        Constant::Str(value) => format!("const.str {:?}", value),
        Constant::Nil => "const.nil".into(),
    }
}
