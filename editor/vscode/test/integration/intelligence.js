const vscode = require('vscode');
const { assert } = require('./helpers');

async function document(root, name, text) {
  const uri = vscode.Uri.joinPath(root, name);
  await vscode.workspace.fs.writeFile(uri, Buffer.from(text));
  return vscode.workspace.openTextDocument(uri);
}

async function run(root) {
  const source = [
    'import "./math.tether" as math',
    'fn main() {',
    '    math.add(1, 2)',
    '}',
  ].join('\n');
  const valid = await document(root, 'intelligence.tether', source);
  const member = new vscode.Position(2, 10);
  const definitions = await vscode.commands.executeCommand(
    'vscode.executeDefinitionProvider', valid.uri, member,
  );
  assert(definitions.length > 0, 'expected imported definition');
  assert(definitions[0].uri.fsPath.endsWith('math.tether'));
  const hovers = await vscode.commands.executeCommand(
    'vscode.executeHoverProvider', valid.uri, member,
  );
  assert(hovers.length > 0, 'expected imported export hover');
  const links = await vscode.commands.executeCommand(
    'vscode.executeLinkProvider', valid.uri,
  );
  assert(links.some((link) => link.target.fsPath.endsWith('math.tether')));
}

module.exports = { run };