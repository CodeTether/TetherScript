pub const SOURCE: &str = r#"
var __wpt_failures=[];
function setup(_) {}
function done() {}
function assert_array_equals(actual,expected,label) {
  if(actual.length!==expected.length) {
    __wpt_failures.push(label+': length '+actual.length+' != '+expected.length);
    return;
  }
  for(let i=0;i<actual.length;i=i+1) {
    if(actual[i]!==expected[i]) {
      __wpt_failures.push(label+': item '+i+' differs');
    }
  }
}
"#;
