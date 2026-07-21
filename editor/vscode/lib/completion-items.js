function item(vscode, label, kind, data, callable) {
  const completion = new vscode.CompletionItem(label, kind);
  if (data) {
    completion.detail = data[0];
    completion.documentation = data[1];
  }
  if (callable) {
    const name = data[0].split('(')[0].split('.').pop();
    completion.insertText = new vscode.SnippetString(`${name}($0)`);
  }
  return completion;
}

function memberItems(vscode, entries) {
  return Object.entries(entries).map(([name, data]) => {
    return item(vscode, name, vscode.CompletionItemKind.Method, data, true);
  });
}

function regularItems(vscode, language) {
  const items = language.keywords.map((name) => {
    return item(vscode, name, vscode.CompletionItemKind.Keyword);
  });
  for (const name of language.constants) {
    items.push(item(vscode, name, vscode.CompletionItemKind.Constant));
  }
  for (const [name, data] of Object.entries(language.namespaces)) {
    items.push(item(vscode, name, vscode.CompletionItemKind.Module, data));
  }
  for (const [name, data] of Object.entries(language.builtins)) {
    items.push(item(vscode, name, vscode.CompletionItemKind.Function, data, true));
  }
  return items;
}

module.exports = { memberItems, regularItems };
