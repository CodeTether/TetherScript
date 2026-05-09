use super::*;

pub(super) fn object(files: Vec<AgentFile>) -> JsValue {
    let values = files.iter().map(file_object::object).collect::<Vec<_>>();
    let mut obj = HashMap::new();
    obj.insert("length".into(), JsValue::Number(values.len() as f64));
    for (index, value) in values.iter().enumerate() {
        obj.insert(index.to_string(), value.clone());
    }
    let item_values = values.clone();
    obj.insert(
        "item".into(),
        native("FileList.item", Some(1), move |args| {
            Ok(item_values
                .get(index(args.first()))
                .cloned()
                .unwrap_or(JsValue::Null))
        }),
    );
    JsValue::Object(Rc::new(RefCell::new(obj)))
}

fn index(value: Option<&JsValue>) -> usize {
    match value {
        Some(JsValue::Number(n)) if n.is_finite() && *n >= 0.0 => n.trunc() as usize,
        Some(value) => value.display().parse().unwrap_or(usize::MAX),
        None => usize::MAX,
    }
}
