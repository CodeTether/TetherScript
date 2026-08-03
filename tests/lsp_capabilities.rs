//! Integration tests for the server-side LSP capabilities.
//!
//! Every test drives a handler with an LSP-shaped JSON request — the request is
//! written as real JSON text and parsed with the in-tree parser, exactly as the
//! stdio server does after reading a `Content-Length` frame — and asserts on the
//! reply. Where the shape matters, the reply is re-encoded to JSON and asserted
//! as text.
//!
//! Coverage, in the order the task requires it:
//!
//! - completion at a position offering an expected builtin and an expected local;
//! - hover over a builtin and over a user `fn`;
//! - definition of a local, of a same-file `fn`, and of an imported symbol;
//! - UTF-16 positions with a 2-byte and a 4-byte character before the cursor;
//! - invalid positions replying safely instead of panicking.

use std::collections::HashMap;
use std::rc::Rc;

use tetherscript::json;
use tetherscript::lsp_capabilities::capabilities;
use tetherscript::lsp_capabilities::jsonval::{field, pointer, ValueText};
use tetherscript::lsp_capabilities::{completion, definition, dispatch, hover};
use tetherscript::value::Value;

const URI: &str = "file:///workspace/main.tether";

/// Parse LSP request params from JSON text, the way the server does.
fn params(uri: &str, line: usize, character: usize) -> Value {
    let text = format!(
        r#"{{"textDocument":{{"uri":"{uri}"}},"position":{{"line":{line},"character":{character}}}}}"#
    );
    json::parse(&Value::Str(Rc::new(text))).expect("params must parse as JSON")
}

/// Re-encode a reply so assertions can be made against real JSON text.
fn encoded(reply: &Value) -> String {
    match json::encode(reply).expect("reply must encode as JSON") {
        Value::Str(text) => text.to_string(),
        other => panic!("encode must return a string, got {}", other.type_name()),
    }
}

/// A one-document store.
fn store(uri: &str, text: &str) -> HashMap<String, String> {
    let mut docs = HashMap::new();
    docs.insert(uri.to_string(), text.to_string());
    docs
}

/// Every `label` in a completion reply.
fn labels(reply: &Value) -> Vec<String> {
    match field(reply, "items") {
        Value::List(items) => items
            .borrow()
            .iter()
            .filter_map(|item| field(item, "label").as_deref_str().map(str::to_string))
            .collect(),
        other => panic!("items must be a list, got {}", other.type_name()),
    }
}

/// The `sortText` of the completion item with a given label.
fn sort_text(reply: &Value, label: &str) -> String {
    match field(reply, "items") {
        Value::List(items) => items
            .borrow()
            .iter()
            .find(|item| field(item, "label").as_deref_str() == Some(label))
            .map(|item| {
                field(item, "sortText")
                    .as_deref_str()
                    .unwrap_or_default()
                    .to_string()
            })
            .unwrap_or_else(|| panic!("no completion item labelled `{label}`")),
        other => panic!("items must be a list, got {}", other.type_name()),
    }
}

/// Markdown body of a hover reply.
fn hover_text(reply: &Value) -> String {
    assert_eq!(
        pointer(reply, &["contents", "kind"]).as_deref_str(),
        Some("markdown"),
        "hover contents must be MarkupContent with kind=markdown"
    );
    pointer(reply, &["contents", "value"])
        .as_deref_str()
        .expect("hover contents must carry a markdown string")
        .to_string()
}

/// `(line, character)` of a definition or hover range start.
fn range_start(reply: &Value) -> (i64, i64) {
    let line = pointer(reply, &["range", "start", "line"]);
    let character = pointer(reply, &["range", "start", "character"]);
    match (line, character) {
        (Value::Int(line), Value::Int(character)) => (line, character),
        _ => panic!("range start must hold integers"),
    }
}

/// A unique scratch directory for the cross-module tests.
fn scratch(name: &str) -> std::path::PathBuf {
    let unique = format!(
        "tetherscript-lsp-{name}-{}-{:?}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or_default()
    );
    let path = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&path).expect("scratch directory must be creatable");
    std::fs::canonicalize(&path).expect("scratch directory must canonicalize")
}

#[test]
fn initialize_advertises_exactly_the_implemented_providers() {
    let names: Vec<&str> = capabilities::entries()
        .iter()
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        names,
        vec!["completionProvider", "hoverProvider", "definitionProvider"],
        "advertising a provider without a handler makes editors show empty popups"
    );
    assert_eq!(capabilities::METHODS.len(), names.len());
    for method in capabilities::METHODS {
        assert!(
            dispatch(method, &Value::Nil, &HashMap::new()).is_some(),
            "advertised method `{method}` must have a handler"
        );
    }
}

#[test]
fn dispatch_declines_methods_it_does_not_implement() {
    let docs = HashMap::new();
    for method in [
        "textDocument/formatting",
        "textDocument/rename",
        "textDocument/references",
        "completionItem/resolve",
    ] {
        assert!(
            dispatch(method, &Value::Nil, &docs).is_none(),
            "`{method}` must fall through to the server's method-not-found reply"
        );
    }
}

#[test]
fn completion_offers_a_builtin_and_an_in_scope_local() {
    let source = "fn main() {\n    let total = 1\n    \n}\n";
    let docs = store(URI, source);
    let reply = completion::handle(&params(URI, 2, 4), &docs);
    let found = labels(&reply);
    assert!(found.contains(&"println".to_string()), "builtin missing");
    assert!(found.contains(&"total".to_string()), "local missing");
    assert!(found.contains(&"main".to_string()), "same-file fn missing");
    assert!(found.contains(&"let".to_string()), "keyword missing");
    assert!(found.contains(&"nil".to_string()), "constant missing");
    assert!(matches!(field(&reply, "isIncomplete"), Value::Bool(false)));
    assert!(encoded(&reply).contains(r#""label":"total""#));
}

#[test]
fn completion_ranks_locals_above_builtins_and_keywords() {
    let source = "fn main() {\n    let total = 1\n    to\n}\n";
    let docs = store(URI, source);
    let reply = completion::handle(&params(URI, 2, 6), &docs);
    let local = sort_text(&reply, "total");
    let same_file_fn = sort_text(&reply, "main");
    let builtin = sort_text(&reply, "println");
    let keyword = sort_text(&reply, "let");
    let constant = sort_text(&reply, "nil");
    assert!(local < same_file_fn, "prefix-matching local outranks fn");
    assert!(same_file_fn < builtin, "file-local fn outranks builtins");
    assert!(builtin < keyword, "builtins outrank keywords");
    assert!(keyword < constant, "keywords outrank constants");
}

#[test]
fn completion_excludes_bindings_whose_scope_has_closed() {
    let source = "fn one() {\n    let hidden = 1\n}\nfn two() {\n    \n}\n";
    let docs = store(URI, source);
    let reply = completion::handle(&params(URI, 4, 4), &docs);
    let found = labels(&reply);
    assert!(
        !found.contains(&"hidden".to_string()),
        "a binding from another function body must not be offered"
    );
    assert!(found.contains(&"two".to_string()));
}

#[test]
fn completion_offers_parameters_of_the_enclosing_function() {
    let source = "fn scaled(factor, value) {\n    \n}\n";
    let docs = store(URI, source);
    let found = labels(&completion::handle(&params(URI, 1, 4), &docs));
    assert!(found.contains(&"factor".to_string()));
    assert!(found.contains(&"value".to_string()));
}

#[test]
fn completion_in_member_position_offers_methods_not_builtins() {
    let source = "fn main() {\n    let text = \"a\"\n    text.\n}\n";
    let docs = store(URI, source);
    let found = labels(&completion::handle(&params(URI, 2, 9), &docs));
    assert!(found.contains(&"trim".to_string()), "method missing");
    assert!(
        !found.contains(&"println".to_string()),
        "offering a global builtin after a `.` teaches the wrong syntax"
    );
}

#[test]
fn completion_after_resource_dot_offers_only_constructors() {
    let source = "fn main() {\n    resource.\n}\n";
    let docs = store(URI, source);
    let found = labels(&completion::handle(&params(URI, 1, 13), &docs));
    assert!(found.contains(&"timer".to_string()));
    assert!(found.contains(&"channel".to_string()));
    assert!(!found.contains(&"trim".to_string()));
}

#[test]
fn completion_still_works_when_the_document_does_not_parse() {
    // Lexes cleanly but fails the parser (`fn main( {`). Symbol extraction works
    // from tokens, so completion must not degrade.
    let docs = store(URI, "fn main( {\n    prin\n");
    let found = labels(&completion::handle(&params(URI, 1, 8), &docs));
    assert!(
        found.contains(&"println".to_string()),
        "a half-typed buffer must still get builtins"
    );
}

#[test]
fn hover_over_a_builtin_reports_its_signature_and_description() {
    let docs = store(URI, "println(1)\n");
    let reply = hover::handle(&params(URI, 0, 2), &docs);
    let text = hover_text(&reply);
    assert!(text.starts_with("```tetherscript\nprintln(...values)\n```"));
    assert!(text.contains("Write values with a newline."));
    assert_eq!(range_start(&reply), (0, 0));
}

#[test]
fn hover_over_a_user_fn_reports_its_signature() {
    let docs = store(URI, "fn add(a, b) {\n    a + b\n}\n");
    let text = hover_text(&hover::handle(&params(URI, 0, 4), &docs));
    assert!(text.contains("add(a, b)"), "got: {text}");
    assert!(text.contains("Function declared in this file."));
}

#[test]
fn hover_over_a_let_binding_reports_what_is_known() {
    let docs = store(URI, "let mut total = 1\nprintln(total)\n");
    let text = hover_text(&hover::handle(&params(URI, 1, 9), &docs));
    assert!(text.contains("let mut total"), "got: {text}");
    assert!(
        text.contains("dynamically typed"),
        "a dynamically typed binding has no declared type to report"
    );
}

#[test]
fn hover_over_a_parameter_names_it_as_a_parameter() {
    let docs = store(URI, "fn scaled(factor) {\n    factor\n}\n");
    let text = hover_text(&hover::handle(&params(URI, 1, 6), &docs));
    assert!(text.contains("Parameter of the enclosing function."));
}

#[test]
fn hover_over_a_user_fn_shadowing_a_builtin_describes_the_user_fn() {
    let docs = store(URI, "fn map(items) {\n    items\n}\nlet m = map([])\n");
    let text = hover_text(&hover::handle(&params(URI, 3, 9), &docs));
    assert!(text.contains("map(items)"), "got: {text}");
    assert!(
        !text.contains("Create an empty map."),
        "the user's own fn shadows the builtin, so hover must describe the fn"
    );
}

#[test]
fn hover_over_a_method_after_a_dot_uses_the_method_catalog() {
    let docs = store(URI, "let items = []\nitems.push(1)\n");
    let text = hover_text(&hover::handle(&params(URI, 1, 7), &docs));
    assert!(text.contains("list.push(value)"), "got: {text}");
}

#[test]
fn hover_on_punctuation_and_unknown_names_returns_null() {
    let docs = store(URI, "let x = 1\nzzz_unknown\n");
    // Column 6 is the `=`: no word touches it.
    assert!(matches!(
        hover::handle(&params(URI, 0, 6), &docs),
        Value::Nil
    ));
    assert!(matches!(
        hover::handle(&params(URI, 1, 4), &docs),
        Value::Nil
    ));
}

#[test]
fn definition_of_a_local_points_at_its_declaration() {
    let docs = store(URI, "let total = 1\nprintln(total)\n");
    let reply = definition::handle(&params(URI, 1, 9), &docs);
    assert_eq!(field(&reply, "uri").as_deref_str(), Some(URI));
    assert_eq!(range_start(&reply), (0, 4));
}

#[test]
fn definition_of_a_same_file_fn_points_at_its_name() {
    let source = "fn twice(n) {\n    n * 2\n}\nlet v = twice(2)\n";
    let docs = store(URI, source);
    let reply = definition::handle(&params(URI, 3, 10), &docs);
    assert_eq!(field(&reply, "uri").as_deref_str(), Some(URI));
    assert_eq!(range_start(&reply), (0, 3));
}

#[test]
fn definition_of_a_fn_declared_below_the_call_site_still_resolves() {
    let source = "fn main() {\n    helper()\n}\nfn helper() {\n    1\n}\n";
    let docs = store(URI, source);
    let reply = definition::handle(&params(URI, 1, 6), &docs);
    assert_eq!(
        range_start(&reply),
        (3, 3),
        "top-level fns hoist, so a call above the declaration must resolve"
    );
}

#[test]
fn definition_picks_the_nearest_visible_binding_when_shadowed() {
    let source = "let value = 1\nfn inner() {\n    let value = 2\n    value\n}\n";
    let docs = store(URI, source);
    let reply = definition::handle(&params(URI, 3, 5), &docs);
    assert_eq!(range_start(&reply), (2, 8), "the inner binding shadows");
}

#[test]
fn definition_of_a_builtin_returns_null() {
    let docs = store(URI, "println(1)\n");
    assert!(matches!(
        definition::handle(&params(URI, 0, 2), &docs),
        Value::Nil
    ));
}

#[test]
fn definition_of_an_imported_symbol_crosses_the_module_boundary() {
    let root = scratch("imported-symbol");
    let module = root.join("math.tether");
    let entry = root.join("main.tether");
    std::fs::write(&module, "export add\nfn add(a, b) {\n    a + b\n}\n")
        .expect("module must be writable");
    let source = "import \"./math.tether\" as math\nlet sum = math.add(1, 2)\n";
    std::fs::write(&entry, source).expect("entry must be writable");

    let uri = tetherscript::lsp_capabilities::uri::from_path(&entry);
    let docs = store(&uri, source);
    let reply = definition::handle(&params(&uri, 1, 15), &docs);

    let target = field(&reply, "uri")
        .as_deref_str()
        .unwrap_or_default()
        .to_string();
    assert!(
        target.ends_with("math.tether"),
        "definition must jump into the imported file, got `{target}`"
    );
    assert_eq!(
        range_start(&reply),
        (1, 3),
        "the jump must land on the declaration, not on the `export` line"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn definition_of_an_import_alias_opens_the_module() {
    let root = scratch("import-alias");
    let module = root.join("math.tether");
    let entry = root.join("main.tether");
    std::fs::write(&module, "export add\nfn add(a) {\n    a\n}\n").expect("module writable");
    let source = "import \"./math.tether\" as math\nlet sum = math.add(1)\n";
    std::fs::write(&entry, source).expect("entry writable");

    let uri = tetherscript::lsp_capabilities::uri::from_path(&entry);
    let docs = store(&uri, source);
    let reply = definition::handle(&params(&uri, 1, 11), &docs);

    assert!(field(&reply, "uri")
        .as_deref_str()
        .unwrap_or_default()
        .ends_with("math.tether"));
    assert_eq!(range_start(&reply), (0, 0));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn definition_of_a_non_exported_module_symbol_returns_null() {
    let root = scratch("not-exported");
    let module = root.join("math.tether");
    let entry = root.join("main.tether");
    std::fs::write(&module, "fn hidden(a) {\n    a\n}\n").expect("module writable");
    let source = "import \"./math.tether\" as math\nlet v = math.hidden(1)\n";
    std::fs::write(&entry, source).expect("entry writable");

    let uri = tetherscript::lsp_capabilities::uri::from_path(&entry);
    let docs = store(&uri, source);
    assert!(
        matches!(definition::handle(&params(&uri, 1, 15), &docs), Value::Nil),
        "a symbol the module does not export is not reachable, so not a jump target"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn completion_after_a_module_alias_offers_its_exports() {
    let root = scratch("module-exports");
    let module = root.join("math.tether");
    let entry = root.join("main.tether");
    std::fs::write(
        &module,
        "export add\nfn add(a, b) {\n    a + b\n}\nfn hidden() {\n    1\n}\n",
    )
    .expect("module writable");
    let source = "import \"./math.tether\" as math\nlet sum = math.\n";
    std::fs::write(&entry, source).expect("entry writable");

    let uri = tetherscript::lsp_capabilities::uri::from_path(&entry);
    let docs = store(&uri, source);
    let found = labels(&completion::handle(&params(&uri, 1, 15), &docs));
    assert_eq!(found, vec!["add".to_string()], "only exports are reachable");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn hover_over_a_module_alias_reports_its_export_count() {
    let root = scratch("alias-hover");
    let module = root.join("math.tether");
    let entry = root.join("main.tether");
    std::fs::write(&module, "export add\nfn add(a) {\n    a\n}\n").expect("module writable");
    let source = "import \"./math.tether\" as math\nlet sum = math.add(1)\n";
    std::fs::write(&entry, source).expect("entry writable");

    let uri = tetherscript::lsp_capabilities::uri::from_path(&entry);
    let docs = store(&uri, source);
    let text = hover_text(&hover::handle(&params(&uri, 1, 11), &docs));
    assert!(
        text.contains("import \"./math.tether\" as math"),
        "got: {text}"
    );
    assert!(text.contains("1 explicit exports."), "got: {text}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn hover_over_an_imported_member_reports_its_qualified_signature() {
    let root = scratch("member-hover");
    let module = root.join("math.tether");
    let entry = root.join("main.tether");
    std::fs::write(&module, "export add\nfn add(a, b) {\n    a + b\n}\n").expect("module writable");
    let source = "import \"./math.tether\" as math\nlet sum = math.add(1, 2)\n";
    std::fs::write(&entry, source).expect("entry writable");

    let uri = tetherscript::lsp_capabilities::uri::from_path(&entry);
    let docs = store(&uri, source);
    let text = hover_text(&hover::handle(&params(&uri, 1, 15), &docs));
    assert!(text.contains("math.add(a, b)"), "got: {text}");
    assert!(text.contains("Exported by `./math.tether`."), "got: {text}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn utf16_positions_survive_a_two_byte_character_before_the_cursor() {
    // `é` is two UTF-8 bytes but one UTF-16 code unit. Column 17 is `len`;
    // treating the column as a byte offset would land on the space before it.
    let source = "println(\"héllo\", len(\"x\"))\n";
    assert_eq!(source.find("len"), Some(18), "byte offset of `len`");
    let docs = store(URI, source);
    let reply = hover::handle(&params(URI, 0, 17), &docs);
    let text = hover_text(&reply);
    assert!(text.contains("len(value)"), "got: {text}");
    assert_eq!(
        range_start(&reply),
        (0, 17),
        "the reply range must be UTF-16 columns, not byte offsets"
    );
}

#[test]
fn utf16_positions_survive_a_four_byte_character_before_the_cursor() {
    // An emoji is four UTF-8 bytes and *two* UTF-16 code units, so the byte
    // offset and the LSP column differ by two here.
    let source = "println(\"😀\", len(\"x\"))\n";
    assert_eq!(source.find("len"), Some(16), "byte offset of `len`");
    let reply = hover::handle(&params(URI, 0, 14), &store(URI, source));
    assert!(hover_text(&reply).contains("len(value)"));
    assert_eq!(range_start(&reply), (0, 14));
}

#[test]
fn definition_reports_utf16_columns_for_a_declaration_after_a_multibyte_char() {
    let source = "let bag = \"café\"\nlet copy = bag\n";
    let docs = store(URI, source);
    let reply = definition::handle(&params(URI, 1, 11), &docs);
    assert_eq!(range_start(&reply), (0, 4));
    let target = hover_text(&hover::handle(&params(URI, 0, 5), &docs));
    assert!(target.contains("let bag"), "got: {target}");
}

#[test]
fn a_line_past_the_end_of_the_document_replies_safely() {
    let docs = store(URI, "let x = 1\n");
    assert!(matches!(
        hover::handle(&params(URI, 900, 0), &docs),
        Value::Nil
    ));
    assert!(matches!(
        definition::handle(&params(URI, 900, 0), &docs),
        Value::Nil
    ));
    assert!(labels(&completion::handle(&params(URI, 900, 0), &docs)).is_empty());
}

#[test]
fn a_character_past_the_end_of_a_line_clamps_instead_of_failing() {
    let docs = store(URI, "let total = 1\n");
    // Clamped to the end of the line, which lands on the `1` literal — a word
    // with no declaration, hover, or documentation, so the reply is null.
    let reply = hover::handle(&params(URI, 0, 9_000), &docs);
    assert!(matches!(reply, Value::Nil));
    assert!(!labels(&completion::handle(&params(URI, 0, 9_000), &docs)).is_empty());
}

#[test]
fn a_request_for_a_document_that_is_not_open_replies_safely() {
    let docs = store(URI, "let x = 1\n");
    let other = params("file:///workspace/never-opened.tether", 0, 0);
    assert!(matches!(hover::handle(&other, &docs), Value::Nil));
    assert!(matches!(definition::handle(&other, &docs), Value::Nil));
    assert!(labels(&completion::handle(&other, &docs)).is_empty());
}

#[test]
fn malformed_params_reply_safely_rather_than_panicking() {
    let docs = store(URI, "let x = 1\n");
    let broken = [
        Value::Nil,
        Value::Int(7),
        json::parse(&Value::Str(Rc::new("{}".to_string()))).expect("parses"),
        json::parse(&Value::Str(Rc::new(
            r#"{"textDocument":{"uri":42},"position":{"line":"x","character":null}}"#.to_string(),
        )))
        .expect("parses"),
    ];
    for value in broken {
        assert!(matches!(hover::handle(&value, &docs), Value::Nil));
        assert!(matches!(definition::handle(&value, &docs), Value::Nil));
        assert!(labels(&completion::handle(&value, &docs)).is_empty());
    }
}

#[test]
fn replies_round_trip_through_the_in_tree_json_encoder() {
    let docs = store(URI, "fn add(a, b) {\n    a + b\n}\n");
    let hover_reply = encoded(&hover::handle(&params(URI, 0, 4), &docs));
    assert!(hover_reply.contains(r#""kind":"markdown""#));
    assert!(hover_reply.contains("add(a, b)"));
    let definition_reply = encoded(&definition::handle(&params(URI, 0, 4), &docs));
    assert!(definition_reply.contains(URI));
    let completion_reply = encoded(&completion::handle(&params(URI, 1, 4), &docs));
    assert!(completion_reply.contains(r#""insertTextFormat":2"#));
}

#[test]
fn dispatch_routes_each_advertised_method_to_the_matching_handler() {
    // Compared field by field rather than by encoded text: JSON object key order
    // comes from a HashMap and is not stable between two separately built maps.
    let docs = store(URI, "fn add(a, b) {\n    a + b\n}\n");
    let request = params(URI, 0, 4);

    let hovered = dispatch("textDocument/hover", &request, &docs).expect("handled");
    assert_eq!(
        hover_text(&hovered),
        hover_text(&hover::handle(&request, &docs))
    );

    let jumped = dispatch("textDocument/definition", &request, &docs).expect("handled");
    let direct = definition::handle(&request, &docs);
    assert_eq!(
        field(&jumped, "uri").as_deref_str(),
        field(&direct, "uri").as_deref_str()
    );
    assert_eq!(range_start(&jumped), range_start(&direct));

    let listed = dispatch("textDocument/completion", &request, &docs).expect("handled");
    assert_eq!(
        labels(&listed),
        labels(&completion::handle(&request, &docs))
    );
}
