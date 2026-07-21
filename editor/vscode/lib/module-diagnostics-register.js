const index = require('./module-index');
const { refresh } = require('./module-diagnostics');

function registerModuleDiagnostics(vscode, context) {
  const collection = vscode.languages.createDiagnosticCollection('tetherscript-modules');
  const update = (document) => refresh(vscode, collection, document);
  const refreshOpen = () => vscode.workspace.textDocuments.forEach(update);
  const changed = (uri) => {
    index.clear(uri);
    refreshOpen();
  };
  const watcher = vscode.workspace.createFileSystemWatcher('**/{*.tether,tetherscript.json}');
  context.subscriptions.push(
    collection,
    watcher,
    vscode.workspace.onDidOpenTextDocument(update),
    vscode.workspace.onDidChangeTextDocument((event) => update(event.document)),
    vscode.workspace.onDidSaveTextDocument(update),
    vscode.workspace.onDidCloseTextDocument((document) => collection.delete(document.uri)),
    watcher.onDidCreate(changed),
    watcher.onDidChange(changed),
    watcher.onDidDelete(changed),
  );
  refreshOpen();
}

module.exports = { registerModuleDiagnostics };