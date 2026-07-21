const aliases = require('./module-alias');
const modulePath = require('./module-path');

function kind(vscode, value) {
  return value.kind === 'function'
    ? vscode.CompletionItemKind.Function
    : vscode.CompletionItemKind.Variable;
}

function insertion(vscode, value, qualifier) {
  const call = value.kind === 'function' ? '($0)' : '';
  return new vscode.SnippetString(`${qualifier}${value.name}${call}`);
}

function namespace(vscode, value, module) {
  const item = new vscode.CompletionItem(value.name, kind(vscode, value));
  item.detail = `${value.signature} · exported by ${module.importPath || module.uri.fsPath}`;
  item.insertText = insertion(vscode, value, '');
  item.documentation = `Explicit tetherscript module export from \`${module.uri.fsPath}\`.`;
  return item;
}

function autoImport(vscode, document, value, module, declarations) {
  const existing = declarations.find((item) => item.path === module.importPath);
  const text = document.getText();
  const alias = existing ? existing.alias : aliases.unique(module, declarations, text);
  const item = new vscode.CompletionItem(value.name, kind(vscode, value));
  item.label = { label: value.name, description: `auto import from ${module.importPath}` };
  item.detail = `${value.signature} · add namespace import as ${alias}`;
  item.insertText = insertion(vscode, value, `${alias}.`);
  item.sortText = `9-${value.name}-${module.importPath}`;
  if (!existing) {
    const offset = modulePath.importInsertOffset(document.getText());
    const position = document.positionAt(offset);
    const newline = document.eol === vscode.EndOfLine.CRLF ? '\r\n' : '\n';
    const edit = `import "${module.importPath}" as ${alias}${newline}`;
    item.additionalTextEdits = [vscode.TextEdit.insert(position, edit)];
  }
  return item;
}

module.exports = { autoImport, namespace };