//! Native agent dashboard stylesheet.

pub(super) const CSS: &str = r#"
* { box-sizing: border-box; }
body { margin: 0; background: #08111f; color: #dbeafe; font-size: 15px; }
main { width: 1280px; padding: 20px; }
header { height: 72px; padding: 16px; background: #10243e; }
h1 { margin: 0; color: #67e8f9; font-size: 24px; }
header span { color: #94a3b8; }
aside { float: left; width: 280px; padding: 18px; background: #0d1b2e; }
aside b, footer b { color: #38bdf8; }
aside p { margin: 4px 0 20px; }
section { margin-left: 300px; padding: 12px; }
article { margin-bottom: 10px; padding: 12px; background: #13263f; }
article b { color: #7dd3fc; }
article p { margin: 6px 0; }
footer { padding: 14px; background: #0b1b30; border: 1px solid #28527a; }
footer p { min-height: 28px; }
small { color: #94a3b8; }
"#;
