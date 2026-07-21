const navigation = require('./module-navigation');

function registerModuleNavigation(vscode, context) {
  const selector = { scheme: 'file', language: 'tetherscript' };
  context.subscriptions.push(
    vscode.languages.registerDefinitionProvider(selector, {
      provideDefinition: (document, position) => navigation.definition(vscode, document, position),
    }),
    vscode.languages.registerDocumentLinkProvider(selector, {
      provideDocumentLinks: (document) => navigation.links(vscode, document),
    }),
  );
}

module.exports = { registerModuleNavigation };