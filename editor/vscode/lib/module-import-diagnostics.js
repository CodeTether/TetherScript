const index = require('./module-index');
const syntax = require('./module-syntax');

function problem(vscode, document, start, length, message) {
  const range = new vscode.Range(document.positionAt(start), document.positionAt(start + length));
  const item = new vscode.Diagnostic(range, message, vscode.DiagnosticSeverity.Error);
  item.source = 'tetherscript modules';
  return item;
}

async function analyze(vscode, document) {
  const diagnostics = [];
  const aliases = new Set();
  for (const declaration of syntax.imports(document.getText())) {
    if (aliases.has(declaration.alias)) {
      diagnostics.push(problem(
        vscode, document, declaration.aliasStart, declaration.alias.length,
        `Duplicate module alias \`${declaration.alias}\`.`,
      ));
      continue;
    }
    aliases.add(declaration.alias);
    const result = await index.imported(vscode, document, declaration);
    if (result.error) diagnostics.push(problem(
      vscode, document, declaration.pathStart, declaration.path.length, result.error,
    ));
  }
  return diagnostics;
}

module.exports = { analyze };