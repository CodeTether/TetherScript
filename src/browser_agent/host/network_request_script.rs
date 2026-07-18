//! JavaScript bridge source for normalized agent network responses.

use crate::value::Value;

pub(super) fn source(
    action: &str,
    url: &str,
    method: &str,
    body: Option<&str>,
) -> Result<String, String> {
    let action = quote(action)?;
    let url = quote(url)?;
    let method = quote(&method.to_ascii_uppercase())?;
    let body = body
        .map(quote)
        .transpose()?
        .unwrap_or_else(|| "null".into());
    Ok(format!(
        "window.__tetherscriptHostResponse=null;fetch({url},{{method:{method},body:{body}}}).then(function(r){{r.text().then(function(t){{window.__tetherscriptHostResponse={{status:r.status,url:r.url,method:{method},body:t,ok:r.ok,transport:{action}}};}});}}).catch(function(e){{window.__tetherscriptHostResponse={{error:String(e),url:{url},method:{method},transport:{action}}};}});"
    ))
}

fn quote(value: &str) -> Result<String, String> {
    crate::json::encode_to_string(&Value::Str(std::rc::Rc::new(value.to_string())))
}
