const vscode = require('vscode');
const completion = require('./completion');
const diagnostics = require('./diagnostics');
const intelligence = require('./intelligence');

async function run() {
  const extension = vscode.extensions.getExtension('tetherscript-rs.tetherscript');
  if (!extension) throw new Error('Real tetherscript extension was not loaded.');
  await extension.activate();
  const folder = vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
  if (!folder) throw new Error('Real package workspace was not opened.');
  await completion.run(folder.uri);
  await intelligence.run(folder.uri);
  await diagnostics.run(folder.uri);
  console.log('Real VS Code module intelligence integration passed.');
}

module.exports = { run };