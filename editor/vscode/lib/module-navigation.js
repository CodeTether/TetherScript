const index = require('./module-index');
const syntax = require('./module-syntax');

function position(vscode, text, offset) {
  const before = text.slice(0, offset).split(/\r?\n/);
  return new vscode.Position(before.length - 1, before[before.length - 1].length);
}

async function target(vscode, document, declaration) {
  const result = await index.imported(vscode, document, declaration);
  return result.module;
}

async function definition(vscode, document, cursor) {
  const range = document.getWordRangeAtPosition(cursor, /[A-Za-z_][A-Za-z0-9_]*/);
  if (!range) return undefined;
  const word = document.getText(range);
  const declarations = syntax.imports(document.getText());
  const direct = declarations.find((item) => item.alias === word);
  if (direct) {
    const module = await target(vscode, document, direct);
    return module && new vscode.Location(module.uri, new vscode.Position(0, 0));
  }
  const line = document.lineAt(cursor).text.slice(0, range.start.character);
  const owner = line.match(/([A-Za-z_][A-Za-z0-9_]*)\.$/);
  if (!owner) return undefined;
  const declaration = declarations.find((item) => item.alias === owner[1]);
  if (!declaration) return undefined;
  const module = await target(vscode, document, declaration);
  const value = module && module.exports.find((item) => item.name === word);
  return value && new vscode.Location(module.uri, position(vscode, module.text, value.start));
}

async function links(vscode, document) {
  const out = [];
  for (const declaration of syntax.imports(document.getText())) {
    const module = await target(vscode, document, declaration);
    if (!module) continue;
    const start = document.positionAt(declaration.pathStart);
    const end = document.positionAt(declaration.pathStart + declaration.path.length);
    const link = new vscode.DocumentLink(new vscode.Range(start, end), module.uri);
    link.tooltip = `Open ${declaration.path}`;
    out.push(link);
  }
  return out;
}

module.exports = { definition, links };