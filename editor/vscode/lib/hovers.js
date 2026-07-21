const {
  builtins, constants, factories, keywords, methods, namespaces,
} = require('./language-data');

const wordPattern = /[A-Za-z_][A-Za-z0-9_]*/;

function markdown(vscode, signature, description) {
  const text = new vscode.MarkdownString();
  text.appendCodeblock(signature, 'tetherscript');
  text.appendMarkdown(description);
  return text;
}

function wordAt(document, position) {
  const range = document.getWordRangeAtPosition(position, wordPattern);
  if (!range) return undefined;
  return { range, text: document.getText(range) };
}

function qualifierBefore(document, range) {
  const prefix = document.lineAt(range.start).text.slice(0, range.start.character);
  const match = prefix.match(/([A-Za-z_][A-Za-z0-9_]*)\.$/);
  return match ? match[1] : undefined;
}

function hoverData(document, word) {
  const qualifier = qualifierBefore(document, word.range);
  if (qualifier === 'resource' && factories[word.text]) return factories[word.text];
  if (qualifier && methods[word.text]) return methods[word.text];
  if (builtins[word.text]) return builtins[word.text];
  if (namespaces[word.text]) return namespaces[word.text];
  if (keywords.includes(word.text)) return [word.text, 'tetherscript language keyword.'];
  if (constants.includes(word.text)) return [word.text, 'Built-in tetherscript constant.'];
  if (methods[word.text]) return methods[word.text];
  return undefined;
}

function registerHovers(vscode, context) {
  const selector = { scheme: 'file', language: 'tetherscript' };
  context.subscriptions.push(vscode.languages.registerHoverProvider(selector, {
    provideHover(document, position) {
      const word = wordAt(document, position);
      if (!word) return undefined;
      const data = hoverData(document, word);
      if (!data) return undefined;
      return new vscode.Hover(markdown(vscode, data[0], data[1]), word.range);
    },
  }));
}

module.exports = { hoverData, registerHovers };
