use super::*;

const METHODS: &[(&str, usize)] = &[
    ("appendData", 1),
    ("deleteData", 2),
    ("insertData", 2),
    ("replaceData", 3),
    ("substringData", 2),
];

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: &DomHandle) {
    for (method, arity) in METHODS {
        let h = handle.clone();
        obj.insert(
            (*method).into(),
            native(method, Some(*arity), move |args| {
                ops::call(&h, method, args)
            }),
        );
    }
}
