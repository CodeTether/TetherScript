const vscode = require('vscode');
const { assert, waitFor } = require('./helpers');

async function document(root, name, text) {
  const uri = vscode.Uri.joinPath(root, name);
  await vscode.workspace.fs.writeFile(uri, Buffer.from(text));
  return vscode.workspace.openTextDocument(uri);
}

async function run(root) {
  const broken = await document(root, 'broken.tether', [
    'import "./math.tether" as math',
    'fn main() { math.missing() }',
  ].join('\n'));
  const found = await waitFor(() => {
    const diagnostics = vscode.languages.getDiagnostics(broken.uri);
    return diagnostics.find((item) => item.message.includes('does not export `missing`'));
  });
  assert.strictEqual(found.source, 'tetherscript modules');

  await document(root, 'cycle-b.tether', 'import "./cycle-a.tether" as a');
  const cycle = await document(root, 'cycle-a.tether', 'import "./cycle-b.tether" as b');
  const cycleDiagnostic = await waitFor(() => {
    const diagnostics = vscode.languages.getDiagnostics(cycle.uri);
    return diagnostics.find((item) => item.message.includes('Module import cycle'));
  });
  assert(cycleDiagnostic.message.includes('cycle-a.tether -> cycle-b.tether'));
}

module.exports = { run };