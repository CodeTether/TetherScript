const vscode = require('vscode');
const { assert, label } = require('./helpers');

async function completions(document, position) {
  return vscode.commands.executeCommand('vscode.executeCompletionItemProvider', document.uri, position);
}

async function write(root, name, text) {
  const uri = vscode.Uri.joinPath(root, name);
  await vscode.workspace.fs.writeFile(uri, Buffer.from(text));
  return vscode.workspace.openTextDocument(uri);
}

async function run(root) {
  const main = await write(root, 'main.tether', 'fn main() {\n    ad\n}\n');
  const auto = await completions(main, new vscode.Position(1, 6));
  const add = auto.items.find((item) => label(item) === 'add' && item.additionalTextEdits);
  assert(add, 'expected real auto-import completion for add');
  assert.strictEqual(add.insertText.value, 'math.add($0)');
  const edit = new vscode.WorkspaceEdit();
  edit.set(main.uri, [
    ...add.additionalTextEdits,
    vscode.TextEdit.replace(new vscode.Range(1, 4, 1, 6), 'math.add()'),
  ]);
  assert(await vscode.workspace.applyEdit(edit));
  await main.save();
  assert(main.getText().includes('import "./math.tether" as math'));
  assert(main.getText().includes('math.add()'));

  const namespace = await write(root, 'namespace.tether', [
    'import "./math.tether" as math',
    'fn main() {',
    '    math.',
    '}',
  ].join('\n'));
  const members = await completions(namespace, new vscode.Position(2, 9));
  assert(members.items.some((item) => label(item) === 'add'));
  assert(members.items.some((item) => label(item) === 'answer'));

  const pathDoc = await write(root, 'path.tether', 'import "./ma');
  const paths = await completions(pathDoc, new vscode.Position(0, 12));
  assert(paths.items.some((item) => label(item) === './math.tether'));
}

module.exports = { run };