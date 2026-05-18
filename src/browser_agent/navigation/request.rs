//! Top-level document navigation request model.

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentRequest {
    pub(crate) method: String,
    pub(crate) url: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Option<String>,
}

impl DocumentRequest {
    pub(crate) fn get(url: impl Into<String>) -> Self {
        Self::new("GET", url)
    }

    pub(crate) fn form_post(url: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new("POST", url)
            .with_header("content-type", "application/x-www-form-urlencoded")
            .with_body(body)
    }

    pub(crate) fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            method: method.into().to_ascii_uppercase(),
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    pub(crate) fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub(crate) fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
}
