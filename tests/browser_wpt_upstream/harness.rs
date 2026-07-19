pub const SOURCE: &str = r#"
var __wpt_failures=[];
function setup(_) {}
function done() {}
function test(callback) {
  try { callback(); }
  catch(error) { __wpt_failures.push(error.message); }
}
function assert_equals(actual,expected,label) {
  if(actual!==expected) {
    __wpt_failures.push((label||'assert_equals')+': '+actual+' != '+expected);
  }
}
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
