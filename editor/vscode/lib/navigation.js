const selector = { scheme: 'file', language: 'tetherscript' };
const fnPattern = /^\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/;

function functionRanges(vscode, document) {
  const items = [];
  for (let line = 0; line < document.lineCount; line += 1) {
    const text = document.lineAt(line).text;
    const match = text.match(fnPattern);
    if (!match) continue;

    const start = text.indexOf(match[1]);
    items.push({
      name: match[1],
      range: new vscode.Range(line, 0, line, text.length),
      selectionRange: new vscode.Range(line, start, line, start + match[1].length),
    });
  }
  return items;
}

function symbol(vscode, item) {
  return new vscode.DocumentSymbol(item.name, 'fn', vscode.SymbolKind.Function, item.range, item.selectionRange);
}

function codeLens(vscode, item) {
  const command = { title: 'Run tetherscript file', command: 'tetherscript.runFile' };
  return new vscode.CodeLens(item.selectionRange, command);
}

function registerNavigation(vscode, context) {
  context.subscriptions.push(
    vscode.languages.registerDocumentSymbolProvider(selector, {
      provideDocumentSymbols: (document) => functionRanges(vscode, document).map((item) => symbol(vscode, item)),
    }),
    vscode.languages.registerDefinitionProvider(selector, {
      provideDefinition(document, position) {
        const wordRange = document.getWordRangeAtPosition(position, /[A-Za-z_][A-Za-z0-9_]*/);
        if (!wordRange) return undefined;
        if (wordRange.start.character > 0) {
          const before = wordRange.start.translate(0, -1);
          if (document.getText(new vscode.Range(before, wordRange.start)) === '.') return undefined;
        }

        const word = document.getText(wordRange);
        return functionRanges(vscode, document)
          .filter((item) => item.name === word && !item.selectionRange.contains(position))
          .map((item) => new vscode.Location(document.uri, item.selectionRange))[0];
      },
    }),
    vscode.languages.registerCodeLensProvider(selector, {
      provideCodeLenses: (document) => functionRanges(vscode, document).filter((item) => item.name === 'main').map((item) => codeLens(vscode, item)),
    }),
  );
}

module.exports = { registerNavigation };