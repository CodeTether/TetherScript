/// Identifies an SSA value within one function.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::ValueId;
/// assert_eq!(ValueId(3).0, 3);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ValueId(pub u32);

/// Stores a literal embedded in Tether IR.
///
/// Variants represent the primitive constants accepted by the first lowering
/// stage. Heap allocation remains a runtime concern.
///
/// # Examples
///
/// ```
/// use tetherscript::ir::Constant;
/// assert_eq!(Constant::Int(42), Constant::Int(42));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Nil,
}
