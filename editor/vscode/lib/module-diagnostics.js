const cycleDiagnostics = require('./module-cycle-diagnostics');
const importDiagnostics = require('./module-import-diagnostics');
const memberDiagnostics = require('./module-member-diagnostics');
const syntax = require('./module-syntax');

function range(vscode, document, start, length) {
  return new vscode.Range(document.positionAt(start), document.positionAt(start + length));
}

function problem(vscode, location, message) {
  const item = new vscode.Diagnostic(location, message, vscode.DiagnosticSeverity.Error);
  item.source = 'tetherscript modules';
  return item;
}

async function analyze(vscode, document) {
  const text = document.getText();
  const diagnostics = await importDiagnostics.analyze(vscode, document);
  for (const value of syntax.exported(text).filter((item) => item.kind === 'unknown')) {
    const location = range(vscode, document, value.start, value.name.length);
    diagnostics.push(problem(vscode, location, `Export \`${value.name}\` is not declared.`));
  }
  diagnostics.push(...await memberDiagnostics.analyze(vscode, document));
  diagnostics.push(...await cycleDiagnostics.analyze(vscode, document));
  return diagnostics;
}

function refresh(vscode, collection, document) {
  if (document.languageId !== 'tetherscript' || document.uri.scheme !== 'file') return;
  const version = document.version;
  void analyze(vscode, document).then((diagnostics) => {
    if (document.version === version) collection.set(document.uri, diagnostics);
  });
}

module.exports = { analyze, refresh };