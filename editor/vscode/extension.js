const { workspace } = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let client;

function activate(context) {
  const config = workspace.getConfiguration('kiln');
  const command = config.get('serverPath') || 'kiln';

  const serverOptions = {
    run:   { command, args: ['--lsp'], transport: TransportKind.stdio },
    debug: { command, args: ['--lsp'], transport: TransportKind.stdio },
  };

  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'kiln' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.kl'),
    },
  };

  client = new LanguageClient('kiln', 'Kiln', serverOptions, clientOptions);
  client.start();
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
