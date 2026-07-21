const { exported } = require('./module-syntax');

const cache = new Map();

function openText(vscode, uri) {
  const document = vscode.workspace.textDocuments.find((item) => item.uri.fsPath === uri.fsPath);
  return document && document.getText();
}

async function record(vscode, uri) {
  const open = openText(vscode, uri);
  if (open !== undefined) return { uri, text: open, exports: exported(open) };
  const stat = await vscode.workspace.fs.stat(uri);
  const existing = cache.get(uri.fsPath);
  if (existing && existing.mtime === stat.mtime && existing.size === stat.size) {
    return existing.module;
  }
  const text = Buffer.from(await vscode.workspace.fs.readFile(uri)).toString('utf8');
  const module = { uri, text, exports: exported(text) };
  cache.set(uri.fsPath, { mtime: stat.mtime, size: stat.size, module });
  return module;
}

function clear(uri) {
  if (uri) cache.delete(uri.fsPath);
  else cache.clear();
}

module.exports = { clear, record };