//! Deterministic starter-package templates.

pub(super) fn manifest(name: &str) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"schema\": 1,\n",
            "  \"package\": {{\n",
            "    \"name\": \"{}\",\n",
            "    \"version\": \"0.1.0\",\n",
            "    \"entry\": \"src/main.tether\"\n",
            "  }}\n",
            "}}\n"
        ),
        name
    )
}

pub(super) const MAIN: &str = "fn main() {\n    println(\"Hello from tetherscript!\")\n}\n";
