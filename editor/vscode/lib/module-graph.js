const path = require('path');
const index = require('./module-index');
const modulePath = require('./module-path');
const syntax = require('./module-syntax');

function key(file) {
  const normalized = path.normalize(file);
  return process.platform === 'win32' ? normalized.toLowerCase() : normalized;
}

async function cycle(vscode, document) {
  const root = modulePath.packageRoot(document.uri.fsPath);
  const active = new Map();
  const complete = new Set();
  const stack = [];

  async function visit(module) {
    const id = key(module.uri.fsPath);
    if (active.has(id)) return stack.slice(active.get(id)).concat(module.uri.fsPath);
    if (complete.has(id)) return undefined;
    active.set(id, stack.length);
    stack.push(module.uri.fsPath);
    for (const declaration of syntax.imports(module.text)) {
      const target = modulePath.resolveImport(module.uri.fsPath, declaration.path);
      if (!target || !modulePath.contained(root, target)) continue;
      try {
        const next = await index.record(vscode, vscode.Uri.file(target));
        const found = await visit(next);
        if (found) return found;
      } catch (_) {
        // The direct import diagnostic reports unreadable modules.
      }
    }
    stack.pop();
    active.delete(id);
    complete.add(id);
    return undefined;
  }

  return visit({
    uri: document.uri,
    text: document.getText(),
  });
}

module.exports = { cycle };