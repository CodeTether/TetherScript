const { sqlQuery } = require('./sql-data');

function applies(document, position) {
  const prefix = document.lineAt(position).text.slice(0, position.character);
  return /\bdb\.[A-Za-z_][A-Za-z0-9_]*$|\bdb\.$/.test(prefix);
}

function item(vscode) {
  const result = new vscode.CompletionItem('query', vscode.CompletionItemKind.Method);
  result.detail = sqlQuery.signature;
  result.documentation = sqlQuery.description;
  result.insertText = new vscode.SnippetString('query(${1:sql}, [${2:parameters}])');
  return result;
}

function registerSqlCompletions(vscode, context) {
  const selector = { scheme: 'file', language: 'tetherscript' };
  const provider = {
    provideCompletionItems(document, position) {
      return applies(document, position) ? [item(vscode)] : [];
    },
  };
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(selector, provider, '.'),
  );
}

module.exports = { registerSqlCompletions };
