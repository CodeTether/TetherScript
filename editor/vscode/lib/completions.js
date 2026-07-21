const { completionContext } = require('./completion-context');
const { memberItems, regularItems } = require('./completion-items');
const language = require('./language-data');

function registerCompletions(vscode, context) {
  const selector = { scheme: 'file', language: 'tetherscript' };
  const provider = {
    provideCompletionItems(document, position) {
      const target = completionContext(document, position);
      if (target === 'resource') {
        return memberItems(vscode, language.factories);
      }
      if (target === 'member') {
        return memberItems(vscode, language.methods);
      }
      return regularItems(vscode, language);
    },
  };
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(selector, provider, '.'),
  );
}

module.exports = { registerCompletions };
