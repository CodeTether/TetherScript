const index = require('./module-index');

function context(document, position) {
  const before = document.lineAt(position).text.slice(0, position.character);
  const match = before.match(/^\s*import\s+"([^"]*)$/);
  if (!match) return undefined;
  return { prefix: match[1], start: position.translate(0, -match[1].length) };
}

async function provide(vscode, document, position) {
  const current = context(document, position);
  if (!current) return undefined;
  const modules = await index.packageModules(vscode, document);
  return modules
    .filter((module) => module.importPath.startsWith(current.prefix))
    .map((module) => {
      const item = new vscode.CompletionItem(module.importPath, vscode.CompletionItemKind.File);
      item.detail = `tetherscript module · ${module.exports.length} exports`;
      item.insertText = module.importPath;
      item.range = new vscode.Range(current.start, position);
      return item;
    });
}

module.exports = { context, provide };