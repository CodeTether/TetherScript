const assert = require('assert');
const { codeOnly } = require('../lib/source-mask');

module.exports = function sourceMaskTest() {
  const source = [
    'math.add()',
    'println("math.hidden")',
    '// math.comment()',
    'math.sub()',
  ].join('\n');
  const code = codeOnly(source);
  assert(code.includes('math.add()'));
  assert(code.includes('math.sub()'));
  assert(!code.includes('math.hidden'));
  assert(!code.includes('math.comment'));
};