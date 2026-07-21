const assert = require('assert');
const syntax = require('../lib/module-syntax');

module.exports = function moduleSyntaxTest() {
  const source = [
    'import "./math.tether" as math',
    'async fn add(left, right) { left + right }',
    'let answer = 42',
    'export add',
    'export answer',
    'export missing',
    'fn holder() {',
    '    let nested = 1',
    '}',
    'export nested',
  ].join('\n');
  assert.deepStrictEqual(
    syntax.imports(source).map(({ path, alias }) => ({ path, alias })),
    [{ path: './math.tether', alias: 'math' }],
  );
  const exports = syntax.exported(source);
  assert.deepStrictEqual(exports.map((item) => item.name), ['add', 'answer', 'missing', 'nested']);
  assert.strictEqual(exports[0].signature, 'add(left, right)');
  assert.strictEqual(exports[1].kind, 'value');
  assert.strictEqual(exports[2].kind, 'unknown');
  assert.strictEqual(exports[3].kind, 'unknown');
  assert.strictEqual(source.slice(exports[2].start, exports[2].start + 7), 'missing');
  assert.deepStrictEqual(syntax.memberBefore('println(math.ad'), {
    alias: 'math',
    prefix: 'ad',
  });
};