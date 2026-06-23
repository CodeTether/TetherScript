const { builtins, constants, keywords, methods } = require('./language-data');

function item(vscode, label, kind, detail, insertText) {
  const completion = new vscode.CompletionItem(label, kind);
  completion.detail = detail;
  if (insertText) completion.insertText = new vscode.SnippetString(insertText);
  return completion;
}

function functionInsert(signature) {
  const name = signature.split('(')[0];
  if (!signature.includes('(')) return name;
  return `${name}($0)`;
}

function regularItems(vscode) {
  const out = [];
  for (const keyword of keywords) {
    out.push(item(vscode, keyword, vscode.CompletionItemKind.Keyword));
  }
  for (const constant of constants) {
    out.push(item(vscode, constant, vscode.CompletionItemKind.Constant));
  }
  for (const [name, data] of Object.entries(builtins)) {
    out.push(item(vscode, name, vscode.CompletionItemKind.Function, data[0], functionInsert(data[0])));
  }
  return out;
}

function methodItems(vscode) {
  return Object.entries(methods).map(([name, data]) => {
    return item(vscode, name, vscode.CompletionItemKind.Method, data[0], functionInsert(data[0].split('.').pop()));
  });
}

function isMemberCompletion(document, position) {
  const prefix = document.lineAt(position).text.slice(0, position.character);
  return /\.[A-Za-z_][A-Za-z0-9_]*$|\.$/.test(prefix);
}

function registerCompletions(vscode, context) {
  const selector = { scheme: 'file', language: 'tetherscript' };
  context.subscriptions.push(vscode.languages.registerCompletionItemProvider(selector, {
    provideCompletionItems(document, position) {
      return isMemberCompletion(document, position) ? methodItems(vscode) : regularItems(vscode);
    },
  }, '.'));
}

module.exports = { registerCompletions };
