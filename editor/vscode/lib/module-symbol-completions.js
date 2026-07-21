const index = require('./module-index');
const items = require('./module-completion-items');
const syntax = require('./module-syntax');

function linePrefix(document, position) {
  return document.lineAt(position).text.slice(0, position.character);
}

async function namespace(vscode, document, position) {
  const member = syntax.memberBefore(linePrefix(document, position));
  if (!member) return undefined;
  const declaration = syntax.imports(document.getText()).find((item) => item.alias === member.alias);
  if (!declaration) return undefined;
  const result = await index.imported(vscode, document, declaration);
  if (!result.module) return [];
  result.module.importPath = declaration.path;
  return result.module.exports
    .filter((value) => value.name.startsWith(member.prefix))
    .map((value) => items.namespace(vscode, value, result.module));
}

function wordPrefix(document, position) {
  const before = linePrefix(document, position);
  if (/^\s*(?:import|export)\b/.test(before) || syntax.memberBefore(before)) return undefined;
  const match = before.match(/[A-Za-z_][A-Za-z0-9_]*$/);
  return match && match[0];
}

async function symbols(vscode, document, position) {
  const prefix = wordPrefix(document, position);
  if (!prefix) return undefined;
  const declarations = syntax.imports(document.getText());
  const modules = await index.packageModules(vscode, document);
  const out = [];
  for (const module of modules) {
    for (const value of module.exports.filter((item) => item.name.startsWith(prefix))) {
      out.push(items.autoImport(vscode, document, value, module, declarations));
    }
  }
  return out;
}

async function provide(vscode, document, position) {
  return (await namespace(vscode, document, position)) || symbols(vscode, document, position);
}

module.exports = { namespace, provide, symbols };