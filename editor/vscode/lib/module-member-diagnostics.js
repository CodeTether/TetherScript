const index = require('./module-index');
const { codeOnly } = require('./source-mask');
const syntax = require('./module-syntax');

async function analyze(vscode, document) {
  const text = document.getText();
  const modules = new Map();
  for (const declaration of syntax.imports(text)) {
    const result = await index.imported(vscode, document, declaration);
    if (result.module) modules.set(declaration.alias, result.module);
  }
  const diagnostics = [];
  const pattern = /\b([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)\b/g;
  const code = codeOnly(text);
  let match;
  while ((match = pattern.exec(code)) !== null) {
    const module = modules.get(match[1]);
    if (!module || module.exports.some((item) => item.name === match[2])) continue;
    const start = match.index + match[0].lastIndexOf(match[2]);
    const range = new vscode.Range(
      document.positionAt(start),
      document.positionAt(start + match[2].length),
    );
    const diagnostic = new vscode.Diagnostic(
      range,
      `Module \`${match[1]}\` does not export \`${match[2]}\`.`,
      vscode.DiagnosticSeverity.Error,
    );
    diagnostic.source = 'tetherscript modules';
    diagnostics.push(diagnostic);
  }
  return diagnostics;
}

module.exports = { analyze };