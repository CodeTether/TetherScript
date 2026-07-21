const assert = require('assert');
const { memberItems, regularItems } = require('../lib/completion-items');
const language = require('../lib/language-data');

class CompletionItem {
  constructor(label, kind) { this.label = label; this.kind = kind; }
}

class SnippetString {
  constructor(value) { this.value = value; }
}

const vscode = {
  CompletionItem,
  SnippetString,
  CompletionItemKind: { Constant: 1, Function: 2, Keyword: 3, Method: 4, Module: 5 },
};

function run() {
  const regular = regularItems(vscode, language);
  assert.ok(regular.some((item) => item.label === 'process_list'));
  assert.ok(regular.some((item) => item.label === 'resource'));
  const factories = memberItems(vscode, language.factories);
  const file = factories.find((item) => item.label === 'file');
  assert.strictEqual(file.insertText.value, 'file($0)');
  assert.strictEqual(file.detail, 'resource.file(path, mode)');
}

if (require.main === module) run();

module.exports = run;
