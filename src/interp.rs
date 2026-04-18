//! Tree-walking interpreter.
//!
//! Evaluates the AST directly. Slow compared to a bytecode VM, but a great
//! reference implementation: easy to reason about, easy to debug, and gives
//! us a runnable language *today*. The bytecode VM will port these semantics
//! one-to-one.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::value::{Env, FnObj, NativeFn, Slot, Value};

/// Non-local control flow. Wrapped in Result::Err so we can `?` it through
/// the evaluator without polluting the happy path.
pub enum Unwind {
    Error(String),
    Return(Value),
    Panic(String),
}

impl From<String> for Unwind {
    fn from(s: String) -> Self { Unwind::Error(s) }
}

pub type EvalResult = Result<Value, Unwind>;

pub struct Interpreter {
    pub globals: Rc<RefCell<Env>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Env::new_global();
        install_builtins(&globals);
        Self { globals }
    }

    pub fn run(&mut self, program: &Program) -> Result<(), String> {
        // Two-pass: hoist top-level fn declarations so forward references
        // (e.g. `main` calling `fib` defined below) work.
        for stmt in &program.stmts {
            if let Stmt::FnDecl { name, params, body } = stmt {
                let func = Value::Fn(Rc::new(FnObj {
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.globals.clone(),
                    name: Some(name.clone()),
                }));
                self.globals.borrow_mut().define(name, func, false);
            }
        }

        for stmt in &program.stmts {
            if matches!(stmt, Stmt::FnDecl { .. }) { continue; }
            match self.exec_stmt(stmt, &self.globals.clone()) {
                Ok(_) => {}
                Err(Unwind::Error(e)) | Err(Unwind::Panic(e)) => return Err(e),
                Err(Unwind::Return(_)) => return Err("`return` outside of function".into()),
            }
        }

        // Conventional entry point: if `main` is defined, call it.
        let has_main = self.globals.borrow().slots.contains_key("main");
        if has_main {
            let main = self.globals.borrow().get("main").map_err(|e| e)?;
            match self.call(&main, &[]) {
                Ok(_) => Ok(()),
                Err(Unwind::Error(e)) | Err(Unwind::Panic(e)) => Err(e),
                Err(Unwind::Return(_)) => Err("`return` unwound out of main".into()),
            }
        } else {
            Ok(())
        }
    }

    // ---------- statements ----------

    fn exec_stmt(&self, stmt: &Stmt, env: &Rc<RefCell<Env>>) -> EvalResult {
        match stmt {
            Stmt::Let { name, mutable, value } => {
                let v = self.eval(value, env)?;
                env.borrow_mut().define(name, v, *mutable);
                Ok(Value::Nil)
            }
            Stmt::Expr { expr, .. } => self.eval(expr, env),
            Stmt::FnDecl { name, params, body } => {
                let func = Value::Fn(Rc::new(FnObj {
                    params: params.clone(),
                    body: body.clone(),
                    closure: env.clone(),
                    name: Some(name.clone()),
                }));
                env.borrow_mut().define(name, func, false);
                Ok(Value::Nil)
            }
        }
    }

    fn exec_block(&self, block: &Block, env: &Rc<RefCell<Env>>) -> EvalResult {
        let mut last = Value::Nil;
        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_last = i == block.stmts.len() - 1;
            match stmt {
                Stmt::Expr { expr, terminated } => {
                    let v = self.eval(expr, env)?;
                    if is_last && !terminated { last = v; } else { last = Value::Nil; }
                }
                _ => { self.exec_stmt(stmt, env)?; last = Value::Nil; }
            }
        }
        Ok(last)
    }

    // ---------- expressions ----------

    fn eval(&self, expr: &Expr, env: &Rc<RefCell<Env>>) -> EvalResult {
        match expr {
            Expr::Int(n)   => Ok(Value::Int(*n)),
            Expr::Float(n) => Ok(Value::Float(*n)),
            Expr::Str(s)   => Ok(Value::Str(Rc::new(s.clone()))),
            Expr::Bool(b)  => Ok(Value::Bool(*b)),
            Expr::Nil      => Ok(Value::Nil),

            Expr::Ident(name) => Ok(env.borrow().get(name)?),

            Expr::Move(inner) => {
                // `move x` — only meaningful for plain identifiers.
                match inner.as_ref() {
                    Expr::Ident(name) => Ok(env.borrow_mut().take(name)?),
                    other => {
                        // `move <expr>` on a non-identifier is just the value.
                        // Nothing to tombstone.
                        self.eval(other, env)
                    }
                }
            }

            // v0: borrows are implicit. `&x` and `&mut x` just evaluate to x.
            // The distinction will matter once we implement borrow tracking
            // on aggregate values (list mutation during iteration, etc.).
            Expr::Borrow(inner) | Expr::BorrowMut(inner) => self.eval(inner, env),

            Expr::Unary { op, rhs } => {
                let v = self.eval(rhs, env)?;
                apply_unary(*op, v).map_err(Unwind::Error)
            }

            Expr::Binary { op, lhs, rhs } => {
                if *op == BinOp::Assign {
                    return self.eval_assign(lhs, rhs, env);
                }
                if *op == BinOp::And {
                    let l = self.eval(lhs, env)?;
                    if !l.truthy() { return Ok(l); }
                    return self.eval(rhs, env);
                }
                if *op == BinOp::Or {
                    let l = self.eval(lhs, env)?;
                    if l.truthy() { return Ok(l); }
                    return self.eval(rhs, env);
                }
                let l = self.eval(lhs, env)?;
                let r = self.eval(rhs, env)?;
                apply_binary(*op, l, r).map_err(Unwind::Error)
            }

            Expr::List(items) => {
                let mut xs = Vec::with_capacity(items.len());
                for it in items { xs.push(self.eval(it, env)?); }
                Ok(Value::List(Rc::new(RefCell::new(xs))))
            }

            Expr::Call { callee, args } => {
                let callee = self.eval(callee, env)?;
                let mut arg_vals = Vec::with_capacity(args.len());
                for a in args { arg_vals.push(self.eval(a, env)?); }
                self.call(&callee, &arg_vals)
            }

            Expr::Index { target, index } => {
                let t = self.eval(target, env)?;
                let i = self.eval(index, env)?;
                index_value(&t, &i).map_err(Unwind::Error)
            }

            Expr::Field { target, name } => {
                let t = self.eval(target, env)?;
                field_value(&t, name).map_err(Unwind::Error)
            }

            Expr::Method { target, name, args } => {
                let t = self.eval(target, env)?;
                let mut arg_vals = Vec::with_capacity(args.len());
                for a in args { arg_vals.push(self.eval(a, env)?); }
                call_method(&t, name, &arg_vals).map_err(Unwind::Error)
            }

            Expr::If { cond, then_branch, else_branch } => {
                let c = self.eval(cond, env)?;
                if c.truthy() {
                    let scope = Env::child(env);
                    self.exec_block(then_branch, &scope)
                } else if let Some(else_branch) = else_branch {
                    let scope = Env::child(env);
                    self.exec_block(else_branch, &scope)
                } else {
                    Ok(Value::Nil)
                }
            }

            Expr::While { cond, body } => {
                loop {
                    let c = self.eval(cond, env)?;
                    if !c.truthy() { break; }
                    let scope = Env::child(env);
                    self.exec_block(body, &scope)?;
                }
                Ok(Value::Nil)
            }

            Expr::Block(block) => {
                let scope = Env::child(env);
                self.exec_block(block, &scope)
            }

            Expr::Fn { params, body } => {
                Ok(Value::Fn(Rc::new(FnObj {
                    params: params.clone(),
                    body: body.clone(),
                    closure: env.clone(),
                    name: None,
                })))
            }

            Expr::Return(inner) => {
                let v = match inner {
                    Some(e) => self.eval(e, env)?,
                    None => Value::Nil,
                };
                Err(Unwind::Return(v))
            }

            Expr::Panic(msg) => {
                let v = self.eval(msg, env)?;
                Err(Unwind::Panic(format!("panic: {}", v)))
            }
        }
    }

    fn eval_assign(&self, lhs: &Expr, rhs: &Expr, env: &Rc<RefCell<Env>>) -> EvalResult {
        let value = self.eval(rhs, env)?;
        match lhs {
            Expr::Ident(name) => {
                env.borrow_mut().assign(name, value.clone())?;
                Ok(value)
            }
            Expr::Index { target, index } => {
                let t = self.eval(target, env)?;
                let i = self.eval(index, env)?;
                match (&t, &i) {
                    (Value::List(xs), Value::Int(idx)) => {
                        let mut xs = xs.borrow_mut();
                        let len = xs.len() as i64;
                        let idx = if *idx < 0 { idx + len } else { *idx };
                        if idx < 0 || idx >= len {
                            return Err(Unwind::Error(format!("index {} out of bounds (len {})", idx, len)));
                        }
                        xs[idx as usize] = value.clone();
                        Ok(value)
                    }
                    (Value::Map(m), Value::Str(k)) => {
                        m.borrow_mut().insert((**k).clone(), value.clone());
                        Ok(value)
                    }
                    _ => Err(Unwind::Error(format!(
                        "cannot index-assign into {} with {}", t.type_name(), i.type_name()
                    ))),
                }
            }
            Expr::Field { target, name } => {
                let t = self.eval(target, env)?;
                match t {
                    Value::Map(m) => {
                        m.borrow_mut().insert(name.clone(), value.clone());
                        Ok(value)
                    }
                    _ => Err(Unwind::Error(format!(
                        "cannot set field `{}` on {}", name, t.type_name()
                    ))),
                }
            }
            _ => Err(Unwind::Error("invalid assignment target".into())),
        }
    }

    pub fn call(&self, callee: &Value, args: &[Value]) -> EvalResult {
        match callee {
            Value::Fn(f) => {
                if args.len() != f.params.len() {
                    return Err(Unwind::Error(format!(
                        "{} expected {} args, got {}",
                        f.name.as_deref().unwrap_or("<fn>"), f.params.len(), args.len()
                    )));
                }
                let scope = Env::child(&f.closure);
                {
                    let mut s = scope.borrow_mut();
                    for (name, val) in f.params.iter().zip(args.iter()) {
                        s.slots.insert(name.clone(), Slot::Live {
                            value: val.clone(), mutable: true,
                        });
                    }
                }
                match self.exec_block(&f.body, &scope) {
                    Ok(v) => Ok(v),
                    Err(Unwind::Return(v)) => Ok(v),
                    Err(other) => Err(other),
                }
            }
            Value::Native(n) => {
                if let Some(arity) = n.arity {
                    if args.len() != arity {
                        return Err(Unwind::Error(format!(
                            "{} expected {} args, got {}", n.name, arity, args.len()
                        )));
                    }
                }
                (n.func)(args).map_err(Unwind::Error)
            }
            other => Err(Unwind::Error(format!("{} is not callable", other.type_name()))),
        }
    }
}

// ---------- operators ----------

pub(crate) fn apply_unary(op: UnOp, v: Value) -> Result<Value, String> {
    match (op, v) {
        (UnOp::Neg, Value::Int(n))   => Ok(Value::Int(-n)),
        (UnOp::Neg, Value::Float(n)) => Ok(Value::Float(-n)),
        (UnOp::Not, v)               => Ok(Value::Bool(!v.truthy())),
        (op, v) => Err(format!("cannot apply {:?} to {}", op, v.type_name())),
    }
}

pub(crate) fn apply_binary(op: BinOp, l: Value, r: Value) -> Result<Value, String> {
    use BinOp::*;
    use Value::*;
    match (op, &l, &r) {
        // Numeric
        (Add, Int(a), Int(b))     => Ok(Int(a + b)),
        (Sub, Int(a), Int(b))     => Ok(Int(a - b)),
        (Mul, Int(a), Int(b))     => Ok(Int(a * b)),
        (Div, Int(_), Int(0))     => Err("integer division by zero".into()),
        (Div, Int(a), Int(b))     => Ok(Int(a / b)),
        (Mod, Int(_), Int(0))     => Err("integer modulo by zero".into()),
        (Mod, Int(a), Int(b))     => Ok(Int(a % b)),

        (Add, Float(a), Float(b)) => Ok(Float(a + b)),
        (Sub, Float(a), Float(b)) => Ok(Float(a - b)),
        (Mul, Float(a), Float(b)) => Ok(Float(a * b)),
        (Div, Float(a), Float(b)) => Ok(Float(a / b)),

        // String concatenation
        (Add, Str(a), Str(b))     => Ok(Str(Rc::new(format!("{}{}", a, b)))),
        (Add, Str(a), b)          => Ok(Str(Rc::new(format!("{}{}", a, b)))),
        (Add, a, Str(b))          => Ok(Str(Rc::new(format!("{}{}", a, b)))),

        // Comparisons
        (Eq, a, b)    => Ok(Bool(values_eq(a, b))),
        (NotEq, a, b) => Ok(Bool(!values_eq(a, b))),
        (Lt, Int(a), Int(b))     => Ok(Bool(a < b)),
        (Gt, Int(a), Int(b))     => Ok(Bool(a > b)),
        (LtEq, Int(a), Int(b))   => Ok(Bool(a <= b)),
        (GtEq, Int(a), Int(b))   => Ok(Bool(a >= b)),
        (Lt, Float(a), Float(b)) => Ok(Bool(a < b)),
        (Gt, Float(a), Float(b)) => Ok(Bool(a > b)),
        (LtEq, Float(a), Float(b)) => Ok(Bool(a <= b)),
        (GtEq, Float(a), Float(b)) => Ok(Bool(a >= b)),

        (op, a, b) => Err(format!(
            "cannot apply {:?} to {} and {}", op, a.type_name(), b.type_name()
        )),
    }
}

fn values_eq(a: &Value, b: &Value) -> bool {
    use Value::*;
    match (a, b) {
        (Nil, Nil)             => true,
        (Int(a), Int(b))       => a == b,
        (Float(a), Float(b))   => a == b,
        (Int(a), Float(b))     => (*a as f64) == *b,
        (Float(a), Int(b))     => *a == (*b as f64),
        (Bool(a), Bool(b))     => a == b,
        (Str(a), Str(b))       => a == b,
        _ => false,
    }
}

pub(crate) fn index_value(target: &Value, index: &Value) -> Result<Value, String> {
    match (target, index) {
        (Value::List(xs), Value::Int(i)) => {
            let xs = xs.borrow();
            let len = xs.len() as i64;
            let idx = if *i < 0 { i + len } else { *i };
            if idx < 0 || idx >= len {
                return Err(format!("index {} out of bounds (len {})", i, len));
            }
            Ok(xs[idx as usize].clone())
        }
        (Value::Map(m), Value::Str(k)) => {
            Ok(m.borrow().get(&**k).cloned().unwrap_or(Value::Nil))
        }
        (Value::Str(s), Value::Int(i)) => {
            let bytes = s.as_bytes();
            let len = bytes.len() as i64;
            let idx = if *i < 0 { i + len } else { *i };
            if idx < 0 || idx >= len {
                return Err(format!("index {} out of bounds (len {})", i, len));
            }
            Ok(Value::Str(Rc::new((bytes[idx as usize] as char).to_string())))
        }
        (t, i) => Err(format!("cannot index {} with {}", t.type_name(), i.type_name())),
    }
}

pub(crate) fn field_value(target: &Value, name: &str) -> Result<Value, String> {
    match target {
        Value::Map(m) => Ok(m.borrow().get(name).cloned().unwrap_or(Value::Nil)),
        _ => Err(format!("cannot access field `{}` on {}", name, target.type_name())),
    }
}

pub(crate) fn call_method(target: &Value, name: &str, args: &[Value]) -> Result<Value, String> {
    match (target, name, args) {
        (Value::List(xs), "len", []) => Ok(Value::Int(xs.borrow().len() as i64)),
        (Value::List(xs), "push", [v]) => {
            xs.borrow_mut().push(v.clone());
            Ok(Value::Nil)
        }
        (Value::List(xs), "pop", []) => {
            Ok(xs.borrow_mut().pop().unwrap_or(Value::Nil))
        }
        (Value::Str(s), "len", []) => Ok(Value::Int(s.len() as i64)),
        (Value::Str(s), "upper", []) => Ok(Value::Str(Rc::new(s.to_uppercase()))),
        (Value::Str(s), "lower", []) => Ok(Value::Str(Rc::new(s.to_lowercase()))),
        (Value::Map(m), "len", []) => Ok(Value::Int(m.borrow().len() as i64)),
        (Value::Map(m), "keys", []) => {
            let keys: Vec<Value> = m.borrow().keys()
                .map(|k| Value::Str(Rc::new(k.clone())))
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(keys))))
        }
        (t, n, _) => Err(format!("no method `{}` on {}", n, t.type_name())),
    }
}

// ---------- built-ins ----------

pub(crate) fn install_builtins(env: &Rc<RefCell<Env>>) {
    let mut e = env.borrow_mut();

    let println_fn = Value::Native(Rc::new(NativeFn {
        name: "println".into(),
        arity: None,
        func: Box::new(|args| {
            let s: Vec<String> = args.iter().map(|v| v.to_string()).collect();
            println!("{}", s.join(" "));
            Ok(Value::Nil)
        }),
    }));
    e.define("println", println_fn, false);

    let print_fn = Value::Native(Rc::new(NativeFn {
        name: "print".into(),
        arity: None,
        func: Box::new(|args| {
            let s: Vec<String> = args.iter().map(|v| v.to_string()).collect();
            print!("{}", s.join(" "));
            Ok(Value::Nil)
        }),
    }));
    e.define("print", print_fn, false);

    let len_fn = Value::Native(Rc::new(NativeFn {
        name: "len".into(),
        arity: Some(1),
        func: Box::new(|args| match &args[0] {
            Value::Str(s)  => Ok(Value::Int(s.len() as i64)),
            Value::List(x) => Ok(Value::Int(x.borrow().len() as i64)),
            Value::Map(m)  => Ok(Value::Int(m.borrow().len() as i64)),
            v => Err(format!("len: {} has no length", v.type_name())),
        }),
    }));
    e.define("len", len_fn, false);

    let type_of_fn = Value::Native(Rc::new(NativeFn {
        name: "type_of".into(),
        arity: Some(1),
        func: Box::new(|args| Ok(Value::Str(Rc::new(args[0].type_name().into())))),
    }));
    e.define("type_of", type_of_fn, false);

    let map_fn = Value::Native(Rc::new(NativeFn {
        name: "map".into(),
        arity: Some(0),
        func: Box::new(|_args| {
            Ok(Value::Map(Rc::new(RefCell::new(HashMap::new()))))
        }),
    }));
    e.define("map", map_fn, false);
}
