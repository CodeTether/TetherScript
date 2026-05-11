//! Small self-contained JavaScript engine.
//!
//! This module intentionally uses only the Rust standard library. It is not a
//! web-compatible ECMAScript implementation yet, but it provides a real lexer,
//! parser, lexical environments, user functions, native functions, control flow,
//! arithmetic/comparison/logical operators, arrays, objects, and property access.
//! It is meant to be the JavaScript execution core that the experimental browser
//! module can grow around without introducing external crates.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::value::Value;

#[derive(Clone)]
pub enum JsValue {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Rc<RefCell<Vec<JsValue>>>),
    Object(Rc<RefCell<HashMap<String, JsValue>>>),
    Function(Rc<JsFunction>),
    BoundFunction(Rc<BoundFunction>),
    Native(Rc<NativeFunction>),
}

#[derive(Clone)]
pub struct JsFunction {
    name: Option<String>,
    params: Vec<String>,
    body: Vec<Stmt>,
    env: EnvRef,
}

#[derive(Clone)]
pub struct BoundFunction {
    pub function: JsValue,
    pub this_value: JsValue,
}

pub struct NativeFunction {
    name: String,
    arity: Option<usize>,
    func: Box<dyn Fn(&[JsValue]) -> Result<JsValue, String>>,
    properties: HashMap<String, JsValue>,
}

impl NativeFunction {
    pub fn new(
        name: impl Into<String>,
        arity: Option<usize>,
        func: impl Fn(&[JsValue]) -> Result<JsValue, String> + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            arity,
            func: Box::new(func),
            properties: HashMap::new(),
        }
    }

    pub fn with_property(mut self, name: impl Into<String>, value: JsValue) -> Self {
        self.properties.insert(name.into(), value);
        self
    }

    fn property(&self, name: &str) -> Option<JsValue> {
        self.properties.get(name).cloned()
    }
}

impl fmt::Debug for JsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl PartialEq for JsValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsValue::Undefined, JsValue::Undefined) => true,
            (JsValue::Null, JsValue::Null) => true,
            (JsValue::Bool(a), JsValue::Bool(b)) => a == b,
            (JsValue::Number(a), JsValue::Number(b)) => (*a - *b).abs() < f64::EPSILON,
            (JsValue::String(a), JsValue::String(b)) => a == b,
            (JsValue::Array(a), JsValue::Array(b)) => Rc::ptr_eq(a, b),
            (JsValue::Object(a), JsValue::Object(b)) => Rc::ptr_eq(a, b),
            (JsValue::Function(a), JsValue::Function(b)) => Rc::ptr_eq(a, b),
            (JsValue::BoundFunction(a), JsValue::BoundFunction(b)) => Rc::ptr_eq(a, b),
            (JsValue::Native(a), JsValue::Native(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl JsValue {
    pub fn truthy(&self) -> bool {
        match self {
            JsValue::Undefined | JsValue::Null => false,
            JsValue::Bool(b) => *b,
            JsValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JsValue::String(s) => !s.is_empty(),
            JsValue::Array(_)
            | JsValue::Object(_)
            | JsValue::Function(_)
            | JsValue::BoundFunction(_)
            | JsValue::Native(_) => true,
        }
    }

    pub fn display(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".into(),
            JsValue::Null => "null".into(),
            JsValue::Bool(b) => b.to_string(),
            JsValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            JsValue::String(s) => s.clone(),
            JsValue::Array(items) => items
                .borrow()
                .iter()
                .map(|v| v.display())
                .collect::<Vec<_>>()
                .join(","),
            JsValue::Object(_) => "[object Object]".into(),
            JsValue::Function(fun) => {
                format!("function {}", fun.name.as_deref().unwrap_or("<anonymous>"))
            }
            JsValue::BoundFunction(fun) => format!("bound {}", fun.function.display()),
            JsValue::Native(fun) => format!("function {}", fun.name),
        }
    }

    fn number(&self) -> f64 {
        match self {
            JsValue::Undefined => f64::NAN,
            JsValue::Null => 0.0,
            JsValue::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            JsValue::Number(n) => *n,
            JsValue::String(s) => s.trim().parse().unwrap_or(f64::NAN),
            _ => f64::NAN,
        }
    }
}

pub fn js_to_tether(value: &JsValue) -> Value {
    match value {
        JsValue::Undefined | JsValue::Null => Value::Nil,
        JsValue::Bool(b) => Value::Bool(*b),
        JsValue::Number(n) if n.fract() == 0.0 => Value::Int(*n as i64),
        JsValue::Number(n) => Value::Float(*n),
        JsValue::String(s) => Value::Str(Rc::new(s.clone())),
        JsValue::Array(items) => Value::List(Rc::new(RefCell::new(
            items.borrow().iter().map(js_to_tether).collect(),
        ))),
        JsValue::Object(obj) => {
            let map = obj
                .borrow()
                .iter()
                .map(|(k, v)| (k.clone(), js_to_tether(v)))
                .collect();
            Value::Map(Rc::new(RefCell::new(map)))
        }
        JsValue::Function(_) | JsValue::BoundFunction(_) | JsValue::Native(_) => {
            Value::Str(Rc::new(value.display()))
        }
    }
}

pub fn eval_to_value(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("js_eval: expected 1 arg, got {}", args.len()));
    }
    let Value::Str(source) = &args[0] else {
        return Err(format!(
            "js_eval: expected str, got {}",
            args[0].type_name()
        ));
    };
    eval(source).map(|v| js_to_tether(&v))
}

pub fn eval(source: &str) -> Result<JsValue, String> {
    let mut engine = JsEngine::new();
    engine.eval(source)
}

pub struct JsEngine {
    globals: EnvRef,
    last_console: Rc<RefCell<Vec<String>>>,
}

impl JsEngine {
    pub fn with_globals(globals: HashMap<String, JsValue>) -> Self {
        let engine = Self::new();
        {
            let mut env = engine.globals.borrow_mut();
            for (name, value) in globals {
                env.define(&name, value);
            }
        }
        engine
    }

    pub fn set_global(&mut self, name: &str, value: JsValue) {
        self.globals.borrow_mut().define(name, value);
    }

    pub fn new() -> Self {
        let globals = Env::new(None);
        let last_console = Rc::new(RefCell::new(Vec::new()));
        install_globals(&globals, last_console.clone());
        Self {
            globals,
            last_console,
        }
    }

    pub fn eval(&mut self, source: &str) -> Result<JsValue, String> {
        let tokens = Lexer::new(source).tokenize()?;
        let program = Parser::new(tokens).parse_program()?;
        match execute_block(&program, self.globals.clone())? {
            Flow::Value(v) => Ok(v),
            Flow::Return(v) => Ok(v),
            Flow::Break | Flow::Continue => Err("break/continue outside loop".into()),
        }
    }

    pub fn console_output(&self) -> Vec<String> {
        self.last_console.borrow().clone()
    }
}

impl Default for JsEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn install_globals(env: &EnvRef, console_log: Rc<RefCell<Vec<String>>>) {
    env.borrow_mut().define("undefined", JsValue::Undefined);
    env.borrow_mut().define("NaN", JsValue::Number(f64::NAN));
    env.borrow_mut()
        .define("Infinity", JsValue::Number(f64::INFINITY));

    let log_capture = console_log.clone();
    let log = JsValue::Native(Rc::new(NativeFunction::new("log", None, move |args| {
        let line = args
            .iter()
            .map(|v| v.display())
            .collect::<Vec<_>>()
            .join(" ");
        log_capture.borrow_mut().push(line);
        Ok(JsValue::Undefined)
    })));
    let mut console = HashMap::new();
    console.insert("log".into(), log);
    env.borrow_mut()
        .define("console", JsValue::Object(Rc::new(RefCell::new(console))));

    env.borrow_mut().define(
        "Number",
        JsValue::Native(Rc::new(NativeFunction::new("Number", Some(1), |args| {
            Ok(JsValue::Number(
                args.first().unwrap_or(&JsValue::Undefined).number(),
            ))
        }))),
    );
    env.borrow_mut().define(
        "String",
        JsValue::Native(Rc::new(NativeFunction::new("String", Some(1), |args| {
            Ok(JsValue::String(
                args.first().unwrap_or(&JsValue::Undefined).display(),
            ))
        }))),
    );
    env.borrow_mut().define(
        "Boolean",
        JsValue::Native(Rc::new(NativeFunction::new("Boolean", Some(1), |args| {
            Ok(JsValue::Bool(
                args.first().unwrap_or(&JsValue::Undefined).truthy(),
            ))
        }))),
    );

    for (name, func) in array_global_functions() {
        env.borrow_mut().define(name, func);
    }

    // ── Promise ──────────────────────────────────────────────────────────
    // Promise(executor) — returns a thenable object with synchronous resolution.
    // Promise.resolve / Promise.reject are also separate globals for older callers.
    env.borrow_mut().define(
        "Promise",
        JsValue::Native(Rc::new(NativeFunction::new("Promise", Some(1), |args| {
            let executor = args.first().cloned().unwrap_or(JsValue::Undefined);
            make_promise(executor)
        }))),
    );
    env.borrow_mut().define(
        "Promise_resolve",
        JsValue::Native(Rc::new(NativeFunction::new(
            "Promise_resolve",
            Some(1),
            |args| promise_resolve(args),
        ))),
    );
    env.borrow_mut().define(
        "Promise_reject",
        JsValue::Native(Rc::new(NativeFunction::new(
            "Promise_reject",
            Some(1),
            |args| promise_reject(args),
        ))),
    );
}

fn promise_resolve(args: &[JsValue]) -> Result<JsValue, String> {
    let value = args.first().cloned().unwrap_or(JsValue::Undefined);
    let mut obj = HashMap::new();
    obj.insert(
        "__promise_state".into(),
        JsValue::String("fulfilled".into()),
    );
    obj.insert("value".into(), value.clone());
    install_then_catch(
        &mut obj,
        Rc::new(RefCell::new(PromiseState::Fulfilled(value))),
    );
    Ok(JsValue::Object(Rc::new(RefCell::new(obj))))
}

fn promise_reject(args: &[JsValue]) -> Result<JsValue, String> {
    let reason = args.first().cloned().unwrap_or(JsValue::Undefined);
    let mut obj = HashMap::new();
    obj.insert("__promise_state".into(), JsValue::String("rejected".into()));
    obj.insert("reason".into(), reason.clone());
    install_then_catch(
        &mut obj,
        Rc::new(RefCell::new(PromiseState::Rejected(reason))),
    );
    Ok(JsValue::Object(Rc::new(RefCell::new(obj))))
}

/// Internal promise state.
#[derive(Clone)]
enum PromiseState {
    Pending,
    Fulfilled(JsValue),
    Rejected(JsValue),
}

/// Create a new promise object from an executor function.
fn make_promise(executor: JsValue) -> Result<JsValue, String> {
    let state = Rc::new(RefCell::new(PromiseState::Pending));
    let obj_rc: Rc<RefCell<HashMap<String, JsValue>>> = Rc::new(RefCell::new(HashMap::new()));

    // resolve callback
    let s = state.clone();
    let o = obj_rc.clone();
    let resolve_fn = JsValue::Native(Rc::new(NativeFunction::new(
        "resolve",
        Some(1),
        move |args| {
            let val = args.first().cloned().unwrap_or(JsValue::Undefined);
            *s.borrow_mut() = PromiseState::Fulfilled(val.clone());
            o.borrow_mut().insert(
                "__promise_state".into(),
                JsValue::String("fulfilled".into()),
            );
            o.borrow_mut().insert("value".into(), val);
            Ok(JsValue::Undefined)
        },
    )));

    // reject callback
    let s2 = state.clone();
    let o2 = obj_rc.clone();
    let reject_fn = JsValue::Native(Rc::new(NativeFunction::new(
        "reject",
        Some(1),
        move |args| {
            let reason = args.first().cloned().unwrap_or(JsValue::Undefined);
            *s2.borrow_mut() = PromiseState::Rejected(reason.clone());
            o2.borrow_mut()
                .insert("__promise_state".into(), JsValue::String("rejected".into()));
            o2.borrow_mut().insert("reason".into(), reason);
            Ok(JsValue::Undefined)
        },
    )));

    // Install .then / .catch on the shared object
    {
        let mut obj = obj_rc.borrow_mut();
        obj.insert("__promise_state".into(), JsValue::String("pending".into()));
        install_then_catch(&mut obj, state.clone());
    }

    // Execute the executor synchronously
    let _ = call_value(executor, &[resolve_fn, reject_fn]);

    // Return a snapshot of the object (after executor has run, state is known)
    let snapshot: HashMap<String, JsValue> = obj_rc.borrow().clone();
    Ok(JsValue::Object(Rc::new(RefCell::new(snapshot))))
}

/// Install `.then()` and `.catch()` methods on a promise object map.
fn install_then_catch(obj: &mut HashMap<String, JsValue>, state: Rc<RefCell<PromiseState>>) {
    let st = state.clone();
    obj.insert(
        "then".into(),
        JsValue::Native(Rc::new(NativeFunction::new("then", None, move |args| {
            let on_ok = args.first().cloned().unwrap_or(JsValue::Undefined);
            let on_err = args.get(1).cloned().unwrap_or(JsValue::Undefined);
            let current = st.borrow().clone();
            let mut next = HashMap::new();

            match &current {
                PromiseState::Fulfilled(val) => {
                    if on_ok.truthy() {
                        match call_value(on_ok, &[val.clone()]) {
                            Ok(result) => {
                                // If result is itself a promise-like object, propagate
                                if let JsValue::Object(ref robj) = result {
                                    if robj.borrow().contains_key("__promise_state") {
                                        return Ok(JsValue::Object(robj.clone()));
                                    }
                                }
                                next.insert(
                                    "__promise_state".into(),
                                    JsValue::String("fulfilled".into()),
                                );
                                next.insert("value".into(), result);
                            }
                            Err(e) => {
                                next.insert(
                                    "__promise_state".into(),
                                    JsValue::String("rejected".into()),
                                );
                                next.insert("reason".into(), JsValue::String(e));
                            }
                        }
                    } else {
                        next.insert(
                            "__promise_state".into(),
                            JsValue::String("fulfilled".into()),
                        );
                        next.insert("value".into(), val.clone());
                    }
                }
                PromiseState::Rejected(reason) => {
                    if on_err.truthy() {
                        match call_value(on_err, &[reason.clone()]) {
                            Ok(result) => {
                                next.insert(
                                    "__promise_state".into(),
                                    JsValue::String("fulfilled".into()),
                                );
                                next.insert("value".into(), result);
                            }
                            Err(e) => {
                                next.insert(
                                    "__promise_state".into(),
                                    JsValue::String("rejected".into()),
                                );
                                next.insert("reason".into(), JsValue::String(e));
                            }
                        }
                    } else {
                        next.insert("__promise_state".into(), JsValue::String("rejected".into()));
                        next.insert("reason".into(), reason.clone());
                    }
                }
                PromiseState::Pending => {
                    next.insert("__promise_state".into(), JsValue::String("pending".into()));
                }
            }

            let next_state = Rc::new(RefCell::new(
                match next.get("__promise_state").and_then(|v| {
                    if let JsValue::String(s) = v {
                        Some(s.as_str())
                    } else {
                        None
                    }
                }) {
                    Some("fulfilled") => PromiseState::Fulfilled(
                        next.get("value").cloned().unwrap_or(JsValue::Undefined),
                    ),
                    Some("rejected") => PromiseState::Rejected(
                        next.get("reason").cloned().unwrap_or(JsValue::Undefined),
                    ),
                    _ => PromiseState::Pending,
                },
            ));
            install_then_catch(&mut next, next_state);
            Ok(JsValue::Object(Rc::new(RefCell::new(next))))
        }))),
    );

    let st2 = state.clone();
    obj.insert(
        "catch".into(),
        JsValue::Native(Rc::new(NativeFunction::new(
            "catch",
            Some(1),
            move |args| {
                let handler = args.first().cloned().unwrap_or(JsValue::Undefined);
                let current = st2.borrow().clone();
                let mut next = HashMap::new();

                match &current {
                    PromiseState::Rejected(reason) => {
                        if handler.truthy() {
                            match call_value(handler, &[reason.clone()]) {
                                Ok(result) => {
                                    next.insert(
                                        "__promise_state".into(),
                                        JsValue::String("fulfilled".into()),
                                    );
                                    next.insert("value".into(), result);
                                }
                                Err(e) => {
                                    next.insert(
                                        "__promise_state".into(),
                                        JsValue::String("rejected".into()),
                                    );
                                    next.insert("reason".into(), JsValue::String(e));
                                }
                            }
                        } else {
                            next.insert(
                                "__promise_state".into(),
                                JsValue::String("rejected".into()),
                            );
                        }
                    }
                    PromiseState::Fulfilled(val) => {
                        next.insert(
                            "__promise_state".into(),
                            JsValue::String("fulfilled".into()),
                        );
                        next.insert("value".into(), val.clone());
                    }
                    PromiseState::Pending => {
                        next.insert("__promise_state".into(), JsValue::String("pending".into()));
                    }
                }

                let next_state = Rc::new(RefCell::new(
                    match next.get("__promise_state").and_then(|v| {
                        if let JsValue::String(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    }) {
                        Some("fulfilled") => PromiseState::Fulfilled(
                            next.get("value").cloned().unwrap_or(JsValue::Undefined),
                        ),
                        Some("rejected") => PromiseState::Rejected(
                            next.get("reason").cloned().unwrap_or(JsValue::Undefined),
                        ),
                        _ => PromiseState::Pending,
                    },
                ));
                install_then_catch(&mut next, next_state);
                Ok(JsValue::Object(Rc::new(RefCell::new(next))))
            },
        ))),
    );
}

fn array_global_functions() -> Vec<(&'static str, JsValue)> {
    vec![
        (
            "push",
            native_array_global("push", |array, args| array_push(array, args)),
        ),
        (
            "pop",
            native_array_global("pop", |array, _| array_pop(array)),
        ),
        (
            "slice",
            native_array_global("slice", |array, args| array_slice(array, args)),
        ),
        (
            "join",
            native_array_global("join", |array, args| array_join(array, args)),
        ),
        (
            "forEach",
            native_array_global("forEach", |array, args| array_for_each(array, args)),
        ),
        (
            "map",
            native_array_global("map", |array, args| array_map(array, args)),
        ),
    ]
}

fn native_array_global(
    name: &'static str,
    func: impl Fn(Rc<RefCell<Vec<JsValue>>>, &[JsValue]) -> Result<JsValue, String> + 'static,
) -> JsValue {
    JsValue::Native(Rc::new(NativeFunction::new(name, None, move |args| {
        let Some(JsValue::Array(array)) = args.first() else {
            return Err(format!("{}: expected array as first arg", name));
        };
        func(array.clone(), &args[1..])
    })))
}

type EnvRef = Rc<RefCell<Env>>;

#[derive(Clone)]
struct Env {
    values: HashMap<String, JsValue>,
    parent: Option<EnvRef>,
}

impl Env {
    fn new(parent: Option<EnvRef>) -> EnvRef {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            parent,
        }))
    }

    fn define(&mut self, name: &str, value: JsValue) {
        self.values.insert(name.into(), value);
    }

    fn get(&self, name: &str) -> Option<JsValue> {
        self.values
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.borrow().get(name)))
    }

    fn assign(env: &EnvRef, name: &str, value: JsValue) -> Result<(), String> {
        if env.borrow().values.contains_key(name) {
            env.borrow_mut().values.insert(name.into(), value);
            return Ok(());
        }
        if let Some(parent) = env.borrow().parent.clone() {
            return Env::assign(&parent, name, value);
        }
        Err(format!("ReferenceError: {} is not defined", name))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Number(f64),
    String(String),
    Ident(String),
    Let,
    Const,
    Var,
    Function,
    Return,
    If,
    Else,
    While,
    For,
    New,
    Break,
    Continue,
    True,
    False,
    Null,
    This,
    Typeof,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Eq,
    EqEq,
    BangEq,
    StrictEq,
    StrictBangEq,
    Lt,
    Lte,
    Gt,
    Gte,
    AndAnd,
    OrOr,
    Dot,
    Comma,
    Semi,
    Colon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Eof,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    line: usize,
    col: usize,
}

struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    _src: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            _src: src,
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, String> {
        let mut out = Vec::new();
        loop {
            let tok = self.next_token()?;
            let eof = tok.kind == TokenKind::Eof;
            out.push(tok);
            if eof {
                break;
            }
        }
        Ok(out)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_ws_and_comments();
        let line = self.line;
        let col = self.col;
        let Some(c) = self.advance() else {
            return Ok(Token {
                kind: TokenKind::Eof,
                line,
                col,
            });
        };
        let kind = match c {
            '0'..='9' => self.number(c)?,
            '"' | '\'' => self.string(c)?,
            'a'..='z' | 'A'..='Z' | '_' | '$' => self.ident(c),
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '!' => {
                if self.match_char('=') {
                    if self.match_char('=') {
                        TokenKind::StrictBangEq
                    } else {
                        TokenKind::BangEq
                    }
                } else {
                    TokenKind::Bang
                }
            }
            '=' => {
                if self.match_char('=') {
                    if self.match_char('=') {
                        TokenKind::StrictEq
                    } else {
                        TokenKind::EqEq
                    }
                } else {
                    TokenKind::Eq
                }
            }
            '<' => {
                if self.match_char('=') {
                    TokenKind::Lte
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.match_char('=') {
                    TokenKind::Gte
                } else {
                    TokenKind::Gt
                }
            }
            '&' if self.match_char('&') => TokenKind::AndAnd,
            '|' if self.match_char('|') => TokenKind::OrOr,
            '.' => TokenKind::Dot,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semi,
            ':' => TokenKind::Colon,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            other => {
                return Err(format!(
                    "Unexpected character '{}' at {}:{}",
                    other, line, col
                ))
            }
        };
        Ok(Token { kind, line, col })
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while matches!(self.peek(), Some(c) if c.is_whitespace()) {
                self.advance();
            }
            if self.peek() == Some('/') && self.peek_next() == Some('/') {
                while !matches!(self.peek(), None | Some('\n')) {
                    self.advance();
                }
                continue;
            }
            if self.peek() == Some('/') && self.peek_next() == Some('*') {
                self.advance();
                self.advance();
                while !(self.peek() == Some('*') && self.peek_next() == Some('/'))
                    && self.peek().is_some()
                {
                    self.advance();
                }
                if self.peek().is_some() {
                    self.advance();
                    self.advance();
                }
                continue;
            }
            break;
        }
    }

    fn number(&mut self, first: char) -> Result<TokenKind, String> {
        let mut s = String::new();
        s.push(first);
        while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '.') {
            s.push(self.advance().unwrap());
        }
        Ok(TokenKind::Number(
            s.parse().map_err(|_| format!("Invalid number {}", s))?,
        ))
    }

    fn string(&mut self, quote: char) -> Result<TokenKind, String> {
        let mut s = String::new();
        loop {
            let Some(c) = self.advance() else {
                return Err("Unterminated string".into());
            };
            if c == quote {
                break;
            }
            if c == '\\' {
                let Some(e) = self.advance() else {
                    return Err("Unterminated string escape".into());
                };
                s.push(match e {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    other => other,
                });
            } else {
                s.push(c);
            }
        }
        Ok(TokenKind::String(s))
    }

    fn ident(&mut self, first: char) -> TokenKind {
        let mut s = String::new();
        s.push(first);
        while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_' || c == '$') {
            s.push(self.advance().unwrap());
        }
        match s.as_str() {
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "var" => TokenKind::Var,
            "function" => TokenKind::Function,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "new" => TokenKind::New,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "this" => TokenKind::This,
            "typeof" => TokenKind::Typeof,
            _ => TokenKind::Ident(s),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }
    fn match_char(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }
}

#[derive(Debug, Clone)]
enum Stmt {
    Expr(Expr),
    Var(String, Option<Expr>),
    Function(String, Vec<String>, Vec<Stmt>),
    Return(Option<Expr>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    For(Option<Box<Stmt>>, Option<Expr>, Option<Expr>, Box<Stmt>),
    Break,
    Continue,
}
#[derive(Debug, Clone)]
enum Expr {
    Literal(JsValue),
    Var(String),
    This,
    Function(Vec<String>, Vec<Stmt>),
    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    Unary(String, Box<Expr>),
    Typeof(Box<Expr>),
    Binary(Box<Expr>, String, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    New(Box<Expr>, Vec<Expr>),
    Get(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    fn parse_program(&mut self) -> Result<Vec<Stmt>, String> {
        let mut s = Vec::new();
        while !self.is_eof() {
            s.push(self.statement()?);
        }
        Ok(s)
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.matches(&[TokenKind::Let, TokenKind::Const, TokenKind::Var]) {
            return self.var_decl();
        }
        if self.matches(&[TokenKind::Function]) {
            return self.function_decl();
        }
        if self.matches(&[TokenKind::Return]) {
            let expr = if self.check(&TokenKind::Semi) || self.check(&TokenKind::RBrace) {
                None
            } else {
                Some(self.expression()?)
            };
            self.consume_optional_semi();
            return Ok(Stmt::Return(expr));
        }
        if self.matches(&[TokenKind::If]) {
            return self.if_stmt();
        }
        if self.matches(&[TokenKind::While]) {
            return self.while_stmt();
        }
        if self.matches(&[TokenKind::For]) {
            return self.for_stmt();
        }
        if self.matches(&[TokenKind::LBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }
        if self.matches(&[TokenKind::Break]) {
            self.consume_optional_semi();
            return Ok(Stmt::Break);
        }
        if self.matches(&[TokenKind::Continue]) {
            self.consume_optional_semi();
            return Ok(Stmt::Continue);
        }
        let expr = self.expression()?;
        self.consume_optional_semi();
        Ok(Stmt::Expr(expr))
    }

    fn var_decl(&mut self) -> Result<Stmt, String> {
        let name = self.consume_ident("Expected variable name")?;
        let init = if self.matches(&[TokenKind::Eq]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume_optional_semi();
        Ok(Stmt::Var(name, init))
    }

    fn function_decl(&mut self) -> Result<Stmt, String> {
        let name = self.consume_ident("Expected function name")?;
        self.consume(&TokenKind::LParen, "Expected '(' after function name")?;
        let params = self.params()?;
        self.consume(&TokenKind::LBrace, "Expected function body")?;
        Ok(Stmt::Function(name, params, self.block()?))
    }

    fn params(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                params.push(self.consume_ident("Expected parameter")?);
                if !self.matches(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        self.consume(&TokenKind::RParen, "Expected ')' after parameters")?;
        Ok(params)
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut out = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_eof() {
            out.push(self.statement()?);
        }
        self.consume(&TokenKind::RBrace, "Expected '}'")?;
        Ok(out)
    }
    fn if_stmt(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenKind::LParen, "Expected '('")?;
        let cond = self.expression()?;
        self.consume(&TokenKind::RParen, "Expected ')'")?;
        let then = Box::new(self.statement()?);
        let els = if self.matches(&[TokenKind::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If(cond, then, els))
    }
    fn while_stmt(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenKind::LParen, "Expected '('")?;
        let cond = self.expression()?;
        self.consume(&TokenKind::RParen, "Expected ')'")?;
        Ok(Stmt::While(cond, Box::new(self.statement()?)))
    }

    fn for_stmt(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenKind::LParen, "Expected '(' after for")?;
        let init = if self.matches(&[TokenKind::Semi]) {
            None
        } else if self.matches(&[TokenKind::Let, TokenKind::Const, TokenKind::Var]) {
            let name = self.consume_ident("Expected variable name")?;
            let value = if self.matches(&[TokenKind::Eq]) {
                Some(self.expression()?)
            } else {
                None
            };
            self.consume(&TokenKind::Semi, "Expected ';' after for initializer")?;
            Some(Box::new(Stmt::Var(name, value)))
        } else {
            let expr = self.expression()?;
            self.consume(&TokenKind::Semi, "Expected ';' after for initializer")?;
            Some(Box::new(Stmt::Expr(expr)))
        };
        let condition = if self.check(&TokenKind::Semi) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(&TokenKind::Semi, "Expected ';' after for condition")?;
        let increment = if self.check(&TokenKind::RParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(&TokenKind::RParen, "Expected ')' after for clauses")?;
        Ok(Stmt::For(
            init,
            condition,
            increment,
            Box::new(self.statement()?),
        ))
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }
    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;
        if self.matches(&[TokenKind::Eq]) {
            Ok(Expr::Assign(Box::new(expr), Box::new(self.assignment()?)))
        } else {
            Ok(expr)
        }
    }
    fn or(&mut self) -> Result<Expr, String> {
        let mut e = self.and()?;
        while self.matches(&[TokenKind::OrOr]) {
            e = Expr::Binary(Box::new(e), "||".into(), Box::new(self.and()?));
        }
        Ok(e)
    }
    fn and(&mut self) -> Result<Expr, String> {
        let mut e = self.equality()?;
        while self.matches(&[TokenKind::AndAnd]) {
            e = Expr::Binary(Box::new(e), "&&".into(), Box::new(self.equality()?));
        }
        Ok(e)
    }
    fn equality(&mut self) -> Result<Expr, String> {
        let mut e = self.comparison()?;
        while self.matches(&[
            TokenKind::EqEq,
            TokenKind::BangEq,
            TokenKind::StrictEq,
            TokenKind::StrictBangEq,
        ]) {
            let op = match &self.previous().kind {
                TokenKind::EqEq => "==",
                TokenKind::BangEq => "!=",
                TokenKind::StrictEq => "===",
                _ => "!==",
            };
            e = Expr::Binary(Box::new(e), op.into(), Box::new(self.comparison()?));
        }
        Ok(e)
    }
    fn comparison(&mut self) -> Result<Expr, String> {
        let mut e = self.term()?;
        while self.matches(&[TokenKind::Lt, TokenKind::Lte, TokenKind::Gt, TokenKind::Gte]) {
            let op = match &self.previous().kind {
                TokenKind::Lt => "<",
                TokenKind::Lte => "<=",
                TokenKind::Gt => ">",
                _ => ">=",
            };
            e = Expr::Binary(Box::new(e), op.into(), Box::new(self.term()?));
        }
        Ok(e)
    }
    fn term(&mut self) -> Result<Expr, String> {
        let mut e = self.factor()?;
        while self.matches(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = if self.previous().kind == TokenKind::Plus {
                "+"
            } else {
                "-"
            };
            e = Expr::Binary(Box::new(e), op.into(), Box::new(self.factor()?));
        }
        Ok(e)
    }
    fn factor(&mut self) -> Result<Expr, String> {
        let mut e = self.unary()?;
        while self.matches(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let op = match &self.previous().kind {
                TokenKind::Star => "*",
                TokenKind::Slash => "/",
                _ => "%",
            };
            e = Expr::Binary(Box::new(e), op.into(), Box::new(self.unary()?));
        }
        Ok(e)
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if self.matches(&[TokenKind::Bang, TokenKind::Minus, TokenKind::Plus]) {
            let op = match &self.previous().kind {
                TokenKind::Bang => "!",
                TokenKind::Minus => "-",
                _ => "+",
            };
            Ok(Expr::Unary(op.into(), Box::new(self.unary()?)))
        } else if self.matches(&[TokenKind::Typeof]) {
            Ok(Expr::Typeof(Box::new(self.unary()?)))
        } else if self.matches(&[TokenKind::New]) {
            self.new_expr()
        } else {
            self.call()
        }
    }

    fn new_expr(&mut self) -> Result<Expr, String> {
        let mut callee = self.primary()?;
        loop {
            if self.matches(&[TokenKind::Dot]) {
                callee = Expr::Get(Box::new(callee), self.consume_ident("Expected property")?);
            } else if self.matches(&[TokenKind::LBracket]) {
                let idx = self.expression()?;
                self.consume(&TokenKind::RBracket, "Expected ']'")?;
                callee = Expr::Index(Box::new(callee), Box::new(idx));
            } else {
                break;
            }
        }
        self.consume(&TokenKind::LParen, "Expected '(' after constructor")?;
        let args = self.args()?;
        let mut expr = Expr::New(Box::new(callee), args);
        loop {
            if self.matches(&[TokenKind::LParen]) {
                expr = Expr::Call(Box::new(expr), self.args()?);
            } else if self.matches(&[TokenKind::Dot]) {
                expr = Expr::Get(Box::new(expr), self.consume_ident("Expected property")?);
            } else if self.matches(&[TokenKind::LBracket]) {
                let idx = self.expression()?;
                self.consume(&TokenKind::RBracket, "Expected ']'")?;
                expr = Expr::Index(Box::new(expr), Box::new(idx));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut e = self.primary()?;
        loop {
            if self.matches(&[TokenKind::LParen]) {
                e = Expr::Call(Box::new(e), self.args()?);
            } else if self.matches(&[TokenKind::Dot]) {
                e = Expr::Get(Box::new(e), self.consume_ident("Expected property")?);
            } else if self.matches(&[TokenKind::LBracket]) {
                let idx = self.expression()?;
                self.consume(&TokenKind::RBracket, "Expected ']'")?;
                e = Expr::Index(Box::new(e), Box::new(idx));
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                args.push(self.expression()?);
                if !self.matches(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        self.consume(&TokenKind::RParen, "Expected ')' after arguments")?;
        Ok(args)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::Number(n) => Ok(Expr::Literal(JsValue::Number(n))),
            TokenKind::String(s) => Ok(Expr::Literal(JsValue::String(s))),
            TokenKind::True => Ok(Expr::Literal(JsValue::Bool(true))),
            TokenKind::False => Ok(Expr::Literal(JsValue::Bool(false))),
            TokenKind::Null => Ok(Expr::Literal(JsValue::Null)),
            TokenKind::Ident(s) => Ok(Expr::Var(s)),
            TokenKind::This => Ok(Expr::This),
            TokenKind::Function => {
                self.consume(&TokenKind::LParen, "Expected '(' after function")?;
                let params = self.params()?;
                self.consume(&TokenKind::LBrace, "Expected function body")?;
                Ok(Expr::Function(params, self.block()?))
            }
            TokenKind::LParen => {
                let e = self.expression()?;
                self.consume(&TokenKind::RParen, "Expected ')'")?;
                Ok(e)
            }
            TokenKind::LBracket => {
                let mut items = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        items.push(self.expression()?);
                        if !self.matches(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(&TokenKind::RBracket, "Expected ']'")?;
                Ok(Expr::Array(items))
            }
            TokenKind::LBrace => {
                let mut props = Vec::new();
                if !self.check(&TokenKind::RBrace) {
                    loop {
                        let key = match self.advance().kind.clone() {
                            TokenKind::Ident(s) | TokenKind::String(s) => s,
                            other => return Err(format!("Expected object key, got {:?}", other)),
                        };
                        self.consume(&TokenKind::Colon, "Expected ':'")?;
                        props.push((key, self.expression()?));
                        if !self.matches(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(&TokenKind::RBrace, "Expected '}'")?;
                Ok(Expr::Object(props))
            }
            other => Err(format!(
                "Expected expression at {}:{}, got {:?}",
                tok.line, tok.col, other
            )),
        }
    }

    fn matches(&mut self, kinds: &[TokenKind]) -> bool {
        for k in kinds {
            if self.check(k) {
                self.advance();
                return true;
            }
        }
        false
    }
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }
    fn consume(&mut self, kind: &TokenKind, msg: &str) -> Result<(), String> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "{} at {}:{}",
                msg,
                self.peek().line,
                self.peek().col
            ))
        }
    }
    fn consume_ident(&mut self, msg: &str) -> Result<String, String> {
        match self.advance().kind.clone() {
            TokenKind::Ident(s) => Ok(s),
            _ => Err(format!(
                "{} at {}:{}",
                msg,
                self.previous().line,
                self.previous().col
            )),
        }
    }
    fn consume_optional_semi(&mut self) {
        self.matches(&[TokenKind::Semi]);
    }
    fn is_eof(&self) -> bool {
        self.check(&TokenKind::Eof)
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }
    fn advance(&mut self) -> &Token {
        if !self.is_eof() {
            self.pos += 1;
        }
        self.previous()
    }
}

enum Flow {
    Value(JsValue),
    Return(JsValue),
    Break,
    Continue,
}

fn execute_block(stmts: &[Stmt], env: EnvRef) -> Result<Flow, String> {
    let mut last = JsValue::Undefined;
    for stmt in stmts {
        match execute(stmt, env.clone())? {
            Flow::Value(v) => last = v,
            other => return Ok(other),
        }
    }
    Ok(Flow::Value(last))
}

fn execute(stmt: &Stmt, env: EnvRef) -> Result<Flow, String> {
    match stmt {
        Stmt::Expr(e) => Ok(Flow::Value(eval_expr(e, env)?)),
        Stmt::Var(name, init) => {
            let v = init
                .as_ref()
                .map(|e| eval_expr(e, env.clone()))
                .transpose()?
                .unwrap_or(JsValue::Undefined);
            env.borrow_mut().define(name, v);
            Ok(Flow::Value(JsValue::Undefined))
        }
        Stmt::Function(name, params, body) => {
            let fun = JsValue::Function(Rc::new(JsFunction {
                name: Some(name.clone()),
                params: params.clone(),
                body: body.clone(),
                env: env.clone(),
            }));
            env.borrow_mut().define(name, fun);
            Ok(Flow::Value(JsValue::Undefined))
        }
        Stmt::Return(e) => Ok(Flow::Return(
            e.as_ref()
                .map(|e| eval_expr(e, env))
                .transpose()?
                .unwrap_or(JsValue::Undefined),
        )),
        Stmt::Block(stmts) => execute_block(stmts, Env::new(Some(env))),
        Stmt::If(c, t, e) => {
            if eval_expr(c, env.clone())?.truthy() {
                execute(t, env)
            } else if let Some(e) = e {
                execute(e, env)
            } else {
                Ok(Flow::Value(JsValue::Undefined))
            }
        }
        Stmt::While(c, body) => {
            let mut last = JsValue::Undefined;
            while eval_expr(c, env.clone())?.truthy() {
                match execute(body, env.clone())? {
                    Flow::Value(v) => last = v,
                    Flow::Break => break,
                    Flow::Continue => continue,
                    other => return Ok(other),
                }
            }
            Ok(Flow::Value(last))
        }
        Stmt::For(init, condition, increment, body) => execute_for(
            init.as_deref(),
            condition.as_ref(),
            increment.as_ref(),
            body,
            env,
        ),
        Stmt::Break => Ok(Flow::Break),
        Stmt::Continue => Ok(Flow::Continue),
    }
}

fn execute_for(
    init: Option<&Stmt>,
    condition: Option<&Expr>,
    increment: Option<&Expr>,
    body: &Stmt,
    env: EnvRef,
) -> Result<Flow, String> {
    let loop_env = Env::new(Some(env));
    if let Some(init) = init {
        execute(init, loop_env.clone())?;
    }
    let mut last = JsValue::Undefined;
    while condition
        .map(|c| eval_expr(c, loop_env.clone()))
        .transpose()?
        .unwrap_or(JsValue::Bool(true))
        .truthy()
    {
        match execute(body, loop_env.clone())? {
            Flow::Value(v) => last = v,
            Flow::Break => break,
            Flow::Continue => {}
            other => return Ok(other),
        }
        if let Some(increment) = increment {
            eval_expr(increment, loop_env.clone())?;
        }
    }
    Ok(Flow::Value(last))
}

fn eval_expr(expr: &Expr, env: EnvRef) -> Result<JsValue, String> {
    match expr {
        Expr::Literal(v) => Ok(v.clone()),
        Expr::Var(name) => env
            .borrow()
            .get(name)
            .ok_or_else(|| format!("ReferenceError: {} is not defined", name)),
        Expr::This => Ok(env.borrow().get("this").unwrap_or(JsValue::Undefined)),
        Expr::Function(params, body) => Ok(JsValue::Function(Rc::new(JsFunction {
            name: None,
            params: params.clone(),
            body: body.clone(),
            env,
        }))),
        Expr::Array(items) => Ok(JsValue::Array(Rc::new(RefCell::new(
            items
                .iter()
                .map(|e| eval_expr(e, env.clone()))
                .collect::<Result<Vec<_>, _>>()?,
        )))),
        Expr::Object(props) => {
            let mut m = HashMap::new();
            for (k, e) in props {
                m.insert(k.clone(), eval_expr(e, env.clone())?);
            }
            Ok(JsValue::Object(Rc::new(RefCell::new(m))))
        }
        Expr::Unary(op, e) => {
            let v = eval_expr(e, env)?;
            match op.as_str() {
                "!" => Ok(JsValue::Bool(!v.truthy())),
                "-" => Ok(JsValue::Number(-v.number())),
                "+" => Ok(JsValue::Number(v.number())),
                _ => unreachable!(),
            }
        }
        Expr::Typeof(e) => Ok(JsValue::String(typeof_expr(e, env))),
        Expr::Binary(a, op, b) => eval_binary(eval_expr(a, env.clone())?, op, || eval_expr(b, env)),
        Expr::Assign(target, rhs) => {
            let v = eval_expr(rhs, env.clone())?;
            assign_target(target, v.clone(), env)?;
            Ok(v)
        }
        Expr::Call(callee, args) => {
            let callee = eval_callee(callee, env.clone())?;
            let args = args
                .iter()
                .map(|a| eval_expr(a, env.clone()))
                .collect::<Result<Vec<_>, _>>()?;
            call_value(callee, &args)
        }
        Expr::New(callee, args) => {
            let callee = eval_expr(callee, env.clone())?;
            let args = args
                .iter()
                .map(|a| eval_expr(a, env.clone()))
                .collect::<Result<Vec<_>, _>>()?;
            construct_value(callee, &args)
        }
        Expr::Get(obj, prop) => get_property(&eval_expr(obj, env)?, prop),
        Expr::Index(obj, idx) => {
            let key = eval_expr(idx, env.clone())?.display();
            get_property(&eval_expr(obj, env)?, &key)
        }
    }
}

fn eval_callee(expr: &Expr, env: EnvRef) -> Result<JsValue, String> {
    match expr {
        Expr::Get(obj, prop) => {
            let this_value = eval_expr(obj, env)?;
            Ok(bind_this(get_property(&this_value, prop)?, this_value))
        }
        Expr::Index(obj, idx) => {
            let this_value = eval_expr(obj, env.clone())?;
            let key = eval_expr(idx, env)?.display();
            Ok(bind_this(get_property(&this_value, &key)?, this_value))
        }
        _ => eval_expr(expr, env),
    }
}

fn bind_this(function: JsValue, this_value: JsValue) -> JsValue {
    match function {
        JsValue::Function(_) | JsValue::Native(_) | JsValue::BoundFunction(_) => {
            JsValue::BoundFunction(Rc::new(BoundFunction {
                function,
                this_value,
            }))
        }
        other => other,
    }
}

fn typeof_expr(expr: &Expr, env: EnvRef) -> String {
    match expr {
        Expr::Var(name) if env.borrow().get(name).is_none() => "undefined".into(),
        _ => match eval_expr(expr, env).unwrap_or(JsValue::Undefined) {
            JsValue::Undefined => "undefined".into(),
            JsValue::Null => "object".into(),
            JsValue::Bool(_) => "boolean".into(),
            JsValue::Number(_) => "number".into(),
            JsValue::String(_) => "string".into(),
            JsValue::Function(_) | JsValue::BoundFunction(_) | JsValue::Native(_) => {
                "function".into()
            }
            JsValue::Array(_) | JsValue::Object(_) => "object".into(),
        },
    }
}

fn eval_binary(
    left: JsValue,
    op: &str,
    right_eval: impl FnOnce() -> Result<JsValue, String>,
) -> Result<JsValue, String> {
    if op == "&&" {
        return if left.truthy() {
            right_eval()
        } else {
            Ok(left)
        };
    }
    if op == "||" {
        return if left.truthy() {
            Ok(left)
        } else {
            right_eval()
        };
    }
    let right = right_eval()?;
    match op {
        "+" => {
            if matches!(left, JsValue::String(_)) || matches!(right, JsValue::String(_)) {
                Ok(JsValue::String(format!(
                    "{}{}",
                    left.display(),
                    right.display()
                )))
            } else {
                Ok(JsValue::Number(left.number() + right.number()))
            }
        }
        "-" => Ok(JsValue::Number(left.number() - right.number())),
        "*" => Ok(JsValue::Number(left.number() * right.number())),
        "/" => Ok(JsValue::Number(left.number() / right.number())),
        "%" => Ok(JsValue::Number(left.number() % right.number())),
        "==" | "===" => Ok(JsValue::Bool(left == right)),
        "!=" | "!==" => Ok(JsValue::Bool(left != right)),
        "<" => Ok(JsValue::Bool(left.number() < right.number())),
        "<=" => Ok(JsValue::Bool(left.number() <= right.number())),
        ">" => Ok(JsValue::Bool(left.number() > right.number())),
        ">=" => Ok(JsValue::Bool(left.number() >= right.number())),
        _ => unreachable!(),
    }
}

fn assign_target(target: &Expr, value: JsValue, env: EnvRef) -> Result<(), String> {
    match target {
        Expr::Var(name) => Env::assign(&env, name, value),
        Expr::Get(obj, prop) => set_property(&eval_expr(obj, env)?, prop, value),
        Expr::Index(obj, idx) => {
            let key = eval_expr(idx, env.clone())?.display();
            set_property(&eval_expr(obj, env)?, &key, value)
        }
        _ => Err("Invalid assignment target".into()),
    }
}

fn get_property(value: &JsValue, prop: &str) -> Result<JsValue, String> {
    match value {
        JsValue::Object(obj) => Ok(obj
            .borrow()
            .get(prop)
            .cloned()
            .unwrap_or(JsValue::Undefined)),
        JsValue::Array(items) if prop == "length" => {
            Ok(JsValue::Number(items.borrow().len() as f64))
        }
        JsValue::Array(items) => {
            if let Ok(index) = prop.parse::<usize>() {
                return Ok(items
                    .borrow()
                    .get(index)
                    .cloned()
                    .unwrap_or(JsValue::Undefined));
            }
            Ok(array_method(prop, items.clone()).unwrap_or(JsValue::Undefined))
        }
        JsValue::String(s) if prop == "length" => Ok(JsValue::Number(s.chars().count() as f64)),
        JsValue::Native(native) if native.name == "Promise" => match prop {
            "resolve" => Ok(JsValue::Native(Rc::new(NativeFunction::new(
                "Promise.resolve",
                Some(1),
                promise_resolve,
            )))),
            "reject" => Ok(JsValue::Native(Rc::new(NativeFunction::new(
                "Promise.reject",
                Some(1),
                promise_reject,
            )))),
            _ => Ok(JsValue::Undefined),
        },
        JsValue::Native(native) => Ok(native.property(prop).unwrap_or(JsValue::Undefined)),
        _ => Ok(JsValue::Undefined),
    }
}

fn array_method(prop: &str, array: Rc<RefCell<Vec<JsValue>>>) -> Option<JsValue> {
    let name = prop.to_string();
    let method = match prop {
        "push" => NativeFunction::new(name, None, move |args| array_push(array.clone(), args)),
        "pop" => NativeFunction::new(name, Some(0), move |_| array_pop(array.clone())),
        "slice" => NativeFunction::new(name, None, move |args| array_slice(array.clone(), args)),
        "join" => NativeFunction::new(name, None, move |args| array_join(array.clone(), args)),
        "forEach" => NativeFunction::new(name, Some(1), move |args| {
            array_for_each(array.clone(), args)
        }),
        "map" => NativeFunction::new(name, Some(1), move |args| array_map(array.clone(), args)),
        _ => return None,
    };
    Some(JsValue::Native(Rc::new(method)))
}

fn array_push(array: Rc<RefCell<Vec<JsValue>>>, args: &[JsValue]) -> Result<JsValue, String> {
    let mut array = array.borrow_mut();
    array.extend(args.iter().cloned());
    Ok(JsValue::Number(array.len() as f64))
}

fn array_pop(array: Rc<RefCell<Vec<JsValue>>>) -> Result<JsValue, String> {
    Ok(array.borrow_mut().pop().unwrap_or(JsValue::Undefined))
}

fn array_slice(array: Rc<RefCell<Vec<JsValue>>>, args: &[JsValue]) -> Result<JsValue, String> {
    let array = array.borrow();
    let len = array.len();
    let start = args
        .first()
        .map(|v| slice_index(v.number(), len, false))
        .unwrap_or(0);
    let end = args
        .get(1)
        .map(|v| slice_index(v.number(), len, true))
        .unwrap_or(len);
    let end = end.max(start);
    Ok(JsValue::Array(Rc::new(RefCell::new(
        array[start..end].to_vec(),
    ))))
}

fn slice_index(raw: f64, len: usize, default_to_len_on_nan: bool) -> usize {
    if raw.is_nan() {
        return if default_to_len_on_nan { len } else { 0 };
    }
    let len_i = len as i64;
    let index = raw.trunc() as i64;
    let normalized = if index < 0 {
        (len_i + index).max(0)
    } else {
        index.min(len_i)
    };
    normalized as usize
}

fn array_join(array: Rc<RefCell<Vec<JsValue>>>, args: &[JsValue]) -> Result<JsValue, String> {
    let sep = args
        .first()
        .map(JsValue::display)
        .unwrap_or_else(|| ",".into());
    let joined = array
        .borrow()
        .iter()
        .map(|value| match value {
            JsValue::Undefined | JsValue::Null => String::new(),
            other => other.display(),
        })
        .collect::<Vec<_>>()
        .join(&sep);
    Ok(JsValue::String(joined))
}

fn array_for_each(array: Rc<RefCell<Vec<JsValue>>>, args: &[JsValue]) -> Result<JsValue, String> {
    let callback = args
        .first()
        .cloned()
        .ok_or_else(|| "forEach: expected callback".to_string())?;
    let snapshot = array.borrow().clone();
    let array_value = JsValue::Array(array);
    for (index, item) in snapshot.into_iter().enumerate() {
        call_value(
            callback.clone(),
            &[item, JsValue::Number(index as f64), array_value.clone()],
        )?;
    }
    Ok(JsValue::Undefined)
}

fn array_map(array: Rc<RefCell<Vec<JsValue>>>, args: &[JsValue]) -> Result<JsValue, String> {
    let callback = args
        .first()
        .cloned()
        .ok_or_else(|| "map: expected callback".to_string())?;
    let snapshot = array.borrow().clone();
    let array_value = JsValue::Array(array);
    let mut mapped = Vec::with_capacity(snapshot.len());
    for (index, item) in snapshot.into_iter().enumerate() {
        mapped.push(call_value(
            callback.clone(),
            &[item, JsValue::Number(index as f64), array_value.clone()],
        )?);
    }
    Ok(JsValue::Array(Rc::new(RefCell::new(mapped))))
}

fn set_property(value: &JsValue, prop: &str, new_value: JsValue) -> Result<(), String> {
    match value {
        JsValue::Object(obj) => {
            let setter_key = format!("__set:{}", prop);
            let setter = obj.borrow().get(&setter_key).cloned();
            let mut stored_value = new_value.clone();
            if let Some(setter) = setter {
                let setter_result = call_value(setter, std::slice::from_ref(&new_value))?;
                if !matches!(setter_result, JsValue::Undefined) {
                    stored_value = setter_result;
                }
            }
            obj.borrow_mut().insert(prop.into(), stored_value);
            Ok(())
        }
        JsValue::Array(items) => {
            let idx: usize = prop
                .parse()
                .map_err(|_| format!("Invalid array index {}", prop))?;
            let mut items = items.borrow_mut();
            if idx >= items.len() {
                items.resize(idx + 1, JsValue::Undefined);
            }
            items[idx] = new_value;
            Ok(())
        }
        _ => Err("Cannot set property on primitive".into()),
    }
}

fn call_value(callee: JsValue, args: &[JsValue]) -> Result<JsValue, String> {
    match callee {
        JsValue::Native(native) => {
            if let Some(arity) = native.arity {
                if args.len() != arity {
                    return Err(format!(
                        "{}: expected {} args, got {}",
                        native.name,
                        arity,
                        args.len()
                    ));
                }
            }
            (native.func)(args)
        }
        JsValue::BoundFunction(bound) => {
            call_with_this(bound.function.clone(), bound.this_value.clone(), args)
        }
        JsValue::Function(fun) => {
            let call_env = Env::new(Some(fun.env.clone()));
            for (i, p) in fun.params.iter().enumerate() {
                call_env
                    .borrow_mut()
                    .define(p, args.get(i).cloned().unwrap_or(JsValue::Undefined));
            }
            match execute_block(&fun.body, call_env)? {
                Flow::Return(v) | Flow::Value(v) => Ok(v),
                Flow::Break | Flow::Continue => Err("break/continue outside loop".into()),
            }
        }
        _ => Err(format!("TypeError: {} is not callable", callee.display())),
    }
}

fn construct_value(callee: JsValue, args: &[JsValue]) -> Result<JsValue, String> {
    match callee {
        JsValue::Native(_) => call_value(callee, args),
        JsValue::BoundFunction(bound) => construct_value(bound.function.clone(), args),
        JsValue::Function(fun) => {
            let this_value = JsValue::Object(Rc::new(RefCell::new(HashMap::new())));
            let result = call_with_this(JsValue::Function(fun), this_value.clone(), args)?;
            if matches!(
                result,
                JsValue::Array(_)
                    | JsValue::Object(_)
                    | JsValue::Function(_)
                    | JsValue::BoundFunction(_)
                    | JsValue::Native(_)
            ) {
                Ok(result)
            } else {
                Ok(this_value)
            }
        }
        other => Err(format!(
            "TypeError: {} is not a constructor",
            other.display()
        )),
    }
}

pub fn call_function_with_this(
    callee: JsValue,
    this_value: JsValue,
    args: &[JsValue],
) -> Result<JsValue, String> {
    call_with_this(callee, this_value, args)
}

fn call_with_this(
    callee: JsValue,
    this_value: JsValue,
    args: &[JsValue],
) -> Result<JsValue, String> {
    match callee {
        JsValue::Function(fun) => {
            let call_env = Env::new(Some(fun.env.clone()));
            call_env.borrow_mut().define("this", this_value);
            for (i, p) in fun.params.iter().enumerate() {
                call_env
                    .borrow_mut()
                    .define(p, args.get(i).cloned().unwrap_or(JsValue::Undefined));
            }
            match execute_block(&fun.body, call_env)? {
                Flow::Return(v) | Flow::Value(v) => Ok(v),
                Flow::Break | Flow::Continue => Err("break/continue outside loop".into()),
            }
        }
        JsValue::BoundFunction(bound) => {
            call_with_this(bound.function.clone(), bound.this_value.clone(), args)
        }
        other => call_value(other, args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_variables_and_functions_work() {
        assert_eq!(
            eval("let x = 2 + 3 * 4; x;").unwrap(),
            JsValue::Number(14.0)
        );
        assert_eq!(
            eval("function add(a,b){ return a + b; } add(4, 5);").unwrap(),
            JsValue::Number(9.0)
        );
    }

    #[test]
    fn control_flow_arrays_and_objects_work() {
        let src = "let xs=[1,2]; xs[2]=3; let o={name:'Ada'}; let i=0; let sum=0; while(i<xs.length){ sum=sum+xs[i]; i=i+1; } o.name + ':' + sum;";
        assert_eq!(eval(src).unwrap(), JsValue::String("Ada:6".into()));
    }

    #[test]
    fn console_log_is_captured() {
        let mut engine = JsEngine::new();
        engine.eval("console.log('hello', 42);").unwrap();
        assert_eq!(engine.console_output(), vec!["hello 42".to_string()]);
    }

    #[test]
    fn array_methods_work_as_properties_and_globals() {
        let src = "let xs=[1,2]; let pushed=xs.push(3,4); let popped=xs.pop(); let sliced=xs.slice(1).join('|'); let seen=''; xs.forEach(function(v,i){ seen=seen+i+v; }); let mapped=xs.map(function(v,i){ return v+i; }); pushed + ':' + popped + ':' + xs.join('-') + ':' + sliced + ':' + seen + ':' + mapped.join(',') + ':' + push(xs, 9) + ':' + pop(xs);";
        assert_eq!(
            eval(src).unwrap(),
            JsValue::String("4:4:1-2-3:2|3:011223:1,3,5:4:9".into())
        );
    }

    #[test]
    fn new_invokes_native_constructors_like_calls() {
        let mut engine = JsEngine::new();
        engine.set_global(
            "Thing",
            JsValue::Native(Rc::new(NativeFunction::new("Thing", None, |args| {
                let mut obj = HashMap::new();
                obj.insert(
                    "kind".into(),
                    args.first().cloned().unwrap_or(JsValue::Undefined),
                );
                obj.insert(
                    "label".into(),
                    JsValue::Native(Rc::new(NativeFunction::new("Thing.label", Some(0), |_| {
                        Ok(JsValue::String("method".into()))
                    }))),
                );
                Ok(JsValue::Object(Rc::new(RefCell::new(obj))))
            }))),
        );
        assert_eq!(
            engine
                .eval("let a=Thing('call'); let b=new Thing('ctor'); a.kind + ':' + b.kind + ':' + new Thing('x').label();")
                .unwrap(),
            JsValue::String("call:ctor:method".into())
        );
    }

    #[test]
    fn new_constructs_user_functions_with_this_object() {
        assert_eq!(
            eval("function Person(name){ this.name=name; return 7; } let p=new Person('Ada'); p.name + ':' + typeof p;").unwrap(),
            JsValue::String("Ada:object".into())
        );
        assert_eq!(
            eval("function Box(v){ return {value:v}; } new Box(3).value;").unwrap(),
            JsValue::Number(3.0)
        );
    }

    #[test]
    fn promises_support_constructor_then_catch_and_static_helpers() {
        assert_eq!(
            eval("let seen=''; Promise(function(resolve,reject){ resolve('ok'); }).then(function(v){ seen=v+'!'; }); seen;").unwrap(),
            JsValue::String("ok!".into())
        );
        assert_eq!(
            eval("let a=''; Promise.resolve(2).then(function(v){ a='r'+v; }); let b=''; Promise.reject('bad').catch(function(e){ b=e; }); a + ':' + b;").unwrap(),
            JsValue::String("r2:bad".into())
        );
    }
}
