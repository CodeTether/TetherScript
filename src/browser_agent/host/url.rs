//! Minimal relative redirect URL resolution.

pub(super) fn resolve(base: &str, target: &str) -> String {
    if target.contains("://") {
        return target.into();
    }
    let Some((scheme, rest)) = base.split_once("://") else {
        return target.into();
    };
    let host_end = rest.find('/').unwrap_or(rest.len());
    let origin = format!("{}://{}", scheme, &rest[..host_end]);
    if target.starts_with('/') {
        return format!("{}{}", origin, target);
    }
    let path = rest.get(host_end..).unwrap_or("");
    let directory = path
        .split(['?', '#'])
        .next()
        .unwrap_or("")
        .rsplit_once('/')
        .map(|(dir, _)| dir)
        .unwrap_or("");
    format!("{}{}/{}", origin, directory, target)
}
