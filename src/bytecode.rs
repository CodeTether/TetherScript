//! Bytecode representation: instructions, chunks, function prototypes.
//!
//! The bytecode VM shares the `Value` representation and `Env` model with the
//! tree-walking interpreter, so ownership semantics (move / Copy / tombstone)
//! transfer over unchanged. Variable lookup is still by name against `Env` —
//! name resolution on every `GetName` is slower than register/slot IR but
//! keeps semantics one-to-one with the reference interpreter. Fast local
//! slots are a later optimization.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::{Env, Value};

#[derive(Debug, Clone)]
pub enum Instr {
    // Stack
    Pop,

    // Constants
    Const(u16),
    Nil,
    True,
    False,

    // Variables (by name, looked up in the frame's `Env`)
    GetName(u16),
    GetMove(u16),          // `move x` — take from env, leave tombstone
    DefLet(u16, bool),     // name_idx, mutable
    Assign(u16),

    // Unary
    Neg,
    Not,

    // Binary
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,

    // Control flow (offsets are relative to the *next* instruction).
    Jump(i32),
    JumpIfFalse(i32),
    JumpIfFalseKeep(i32),  // leaves value on stack for short-circuit `&&`
    JumpIfTrueKeep(i32),   // leaves value on stack for short-circuit `||`

    // Aggregates
    BuildList(u16),
    Index,
    SetIndex,              // stack: target, index, value -> value
    GetField(u16),
    SetField(u16),         // stack: target, value -> value

    // Method invocation: stack: target, arg1..argN -> result
    Method(u16, u8),

    // Calls: stack: callee, arg1..argN -> result
    Call(u8),
    Return,

    // Function literals / declarations (emits a VmFn wrapping proto + env)
    MakeFn(u16),

    // Scope markers used by if / while / block expressions
    PushScope,
    PopScope,

    // Unconditional runtime halt with the stack-top value as the message
    Panic,
}

#[derive(Debug, Default)]
pub struct Chunk {
    pub code: Vec<Instr>,
    pub consts: Vec<Value>,
    pub names: Vec<String>,
    pub protos: Vec<Rc<FnProto>>,
}

#[derive(Debug)]
pub struct FnProto {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub chunk: Chunk,
}

/// A bytecode function value: prototype + captured closure env.
pub struct VmFnObj {
    pub proto: Rc<FnProto>,
    pub closure: Rc<RefCell<Env>>,
    pub name: Option<String>,
}
