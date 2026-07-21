const { sqlQuery } = require('./sql-data');

function queryAt(vscode, document, position) {
  const range = document.getWordRangeAtPosition(position, /[A-Za-z_][A-Za-z0-9_]*/);
  if (!range || document.getText(range) !== 'query' || range.start.character < 3) return undefined;
  const start = range.start.translate(0, -3);
  return document.getText(new vscode.Range(start, range.start)) === 'db.' ? range : undefined;
}

function registerSqlHovers(vscode, context) {
  const selector = { scheme: 'file', language: 'tetherscript' };
  const provider = {
    provideHover(document, position) {
      const range = queryAt(vscode, document, position);
      if (!range) return undefined;
      const text = new vscode.MarkdownString();
      text.appendCodeblock(sqlQuery.signature, 'tetherscript');
      text.appendMarkdown(sqlQuery.description);
      return new vscode.Hover(text, range);
    },
  };
  context.subscriptions.push(vscode.languages.registerHoverProvider(selector, provider));
}

module.exports = { registerSqlHovers };
