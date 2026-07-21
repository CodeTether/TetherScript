const modulePath = require('./module-path');
const records = require('./module-record');

const roots = new Map();

async function imported(vscode, document, declaration) {
  const target = modulePath.resolveImport(document.uri.fsPath, declaration.path);
  if (!target) return { error: 'Imports must use a relative .tether path.' };
  const root = modulePath.packageRoot(document.uri.fsPath);
  if (!modulePath.contained(root, target)) return { error: 'Import escapes the package root.' };
  try {
    return { module: await records.record(vscode, vscode.Uri.file(target)) };
  } catch (error) {
    return { error: `Cannot read module: ${error.message || error}` };
  }
}

async function packageModules(vscode, document) {
  const root = modulePath.packageRoot(document.uri.fsPath);
  let uris = roots.get(root);
  if (!uris) {
    const pattern = new vscode.RelativePattern(root, '**/*.tether');
    const exclude = '**/{target,node_modules,.git,.codetether-worktrees}/**';
    uris = await vscode.workspace.findFiles(pattern, exclude, 500);
    roots.set(root, uris);
  }
  const modules = [];
  for (const uri of uris) {
    if (uri.fsPath === document.uri.fsPath) continue;
    try {
      const module = await records.record(vscode, uri);
      module.importPath = modulePath.importPath(document.uri.fsPath, uri.fsPath);
      modules.push(module);
    } catch (_) {
      // Unreadable files are reported when explicitly imported.
    }
  }
  return modules;
}

function clear(uri) {
  records.clear(uri);
  roots.clear();
}

module.exports = { clear, imported, packageModules, record: records.record };