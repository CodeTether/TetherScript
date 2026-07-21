const assert = require('assert');
const { completionContext } = require('../lib/completion-context');
const language = require('../lib/language-data');

function position(text) {
  return { character: text.length };
}

function document(text) {
  return { lineAt: () => ({ text }) };
}

function includesEvery(record, names) {
  for (const name of names) assert.ok(record[name], `missing ${name}`);
}

function run() {
  includesEvery(language.builtins, [
    'browser_render', 'chdir', 'http_serve', 'jsonrpc_request', 'os_platform',
    'path_join', 'process_list', 'smtp_send', 'stdio_read', 'tui_read_key',
  ]);
  includesEvery(language.factories, [
    'channel', 'child_process', 'file', 'request_body', 'response_writer',
    'task', 'tcp_connect', 'tcp_listen', 'timer',
  ]);
  includesEvery(language.methods, [
    'cancel', 'close', 'deadline_remaining_ms', 'read', 'recv', 'send', 'wait',
  ]);
  assert.strictEqual(Object.keys(language.builtins).length, 109);
  assert.strictEqual(completionContext(document('resource.'), position('resource.')), 'resource');
  assert.strictEqual(completionContext(document('handle.'), position('handle.')), 'member');
  assert.strictEqual(completionContext(document('let x'), position('let x')), 'regular');
}

if (require.main === module) run();

module.exports = run;
