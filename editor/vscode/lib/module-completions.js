const paths = require('./module-path-completions');
const symbols = require('./module-symbol-completions');

function registerModuleCompletions(vscode, context) {
  const selector = { scheme: 'file', language: 'tetherscript' };
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(selector, {
      provideCompletionItems: (document, position) => paths.provide(vscode, document, position),
    }, '"', '/', '.'),
    vscode.languages.registerCompletionItemProvider(selector, {
      provideCompletionItems: (document, position) => symbols.provide(vscode, document, position),
    }, '.'),
  );
}

module.exports = { registerModuleCompletions };