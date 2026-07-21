const path = require('path');
const graph = require('./module-graph');

async function analyze(vscode, document) {
  const cycle = await graph.cycle(vscode, document);
  if (!cycle) return [];
  const names = cycle.map((file) => path.basename(file)).join(' -> ');
  const diagnostic = new vscode.Diagnostic(
    new vscode.Range(new vscode.Position(0, 0), new vscode.Position(0, 1)),
    `Module import cycle: ${names}`,
    vscode.DiagnosticSeverity.Error,
  );
  diagnostic.source = 'tetherscript modules';
  return [diagnostic];
}

module.exports = { analyze };