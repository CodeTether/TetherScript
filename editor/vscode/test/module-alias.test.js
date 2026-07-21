const assert = require('assert');
const aliases = require('../lib/module-alias');

module.exports = function moduleAliasTest() {
  const module = { uri: { fsPath: '/workspace/math.tether' } };
  assert.strictEqual(aliases.unique(module, [], ''), 'math');
  assert.strictEqual(aliases.unique(module, [], 'let math = map()'), 'math_2');
  assert.strictEqual(aliases.unique(module, [
    { alias: 'math' },
    { alias: 'math_2' },
  ], ''), 'math_3');
};