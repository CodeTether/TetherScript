const assert = require('assert');
const { hoverData } = require('../lib/hovers');

function word(text, start) {
  return { range: { start: { character: start } }, text };
}

function document(text) {
  return { lineAt: () => ({ text }) };
}

function run() {
  const factory = hoverData(document('resource.file'), word('file', 9));
  assert.strictEqual(factory[0], 'resource.file(path, mode)');
  const builtin = hoverData(document('process_list'), word('process_list', 0));
  assert.strictEqual(builtin[0], 'process_list()');
  const method = hoverData(document('handle.cancel'), word('cancel', 7));
  assert.strictEqual(method[0], 'resource.cancel()');
}

if (require.main === module) run();

module.exports = run;
