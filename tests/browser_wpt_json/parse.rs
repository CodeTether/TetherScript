use std::rc::Rc;

use tetherscript::json;
use tetherscript::value::Value;

use super::{access, model::Fixture};

pub fn fixture(source: &str) -> Result<Fixture, String> {
    let root = json::parse(&Value::Str(Rc::new(source.into())))?;
    let Value::Map(root) = root else {
        return Err("browser fixture root must be an object".into());
    };
    let root = root.borrow();
    let expected = access::object(&root, "expect")?;
    let expected = expected.borrow();
    let fixture = Fixture {
        area: access::string(&root, "area")?,
        wpt_shape: access::string(&root, "wpt_shape")?,
        html: access::string(&root, "html")?,
        script: access::string(&root, "script")?,
        expected_value: access::string(&expected, "value")?,
        unsupported: access::strings(&root, "unsupported")?,
    };
    if fixture.unsupported.is_empty() {
        return Err("fixture must document unsupported behavior".into());
    }
    Ok(fixture)
}
