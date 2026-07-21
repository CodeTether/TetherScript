const assert = require('assert');
const path = require('path');
const modules = require('../lib/module-path');

module.exports = function modulePathTest() {
  const root = path.resolve('workspace');
  const entry = path.join(root, 'src', 'main.tether');
  const math = path.join(root, 'src', 'lib', 'math.tether');
  assert.strictEqual(modules.resolveImport(entry, './lib/math.tether'), math);
  assert.strictEqual(modules.resolveImport(entry, './lib/math.js'), undefined);
  assert.strictEqual(modules.resolveImport(entry, 'lib/math.tether'), undefined);
  assert.strictEqual(modules.resolveImport(entry, math), undefined);
  assert.strictEqual(modules.contained(root, math), true);
  assert.strictEqual(modules.contained(root, path.resolve(root, '..', 'other.tether')), false);
  assert.strictEqual(modules.importPath(entry, math), './lib/math.tether');
  assert.strictEqual(modules.aliasFor('/some/http-client.tether'), 'http_client');
  assert.strictEqual(modules.aliasFor('/some/import.tether'), 'import_module');
  assert.strictEqual(modules.importInsertOffset('fn main() {}\n'), 0);
  const imports = [
    'import "./a.tether" as a',
    'import "./b.tether" as b',
    '',
    'fn main() {}',
  ].join('\n');
  assert.strictEqual(
    modules.importInsertOffset(imports),
    imports.indexOf('\n\n') + 1,
  );
};