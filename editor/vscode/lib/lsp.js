const { createClient } = require('./lsp-client');

function alive(child) {
  return child && child.pid && !child.killed;
}

function createLspController(vscode) {
  let client;

  async function start(notify = true) {
    if (client && !client.needsStart()) {
      if (notify) vscode.window.showInformationMessage('tetherscript language server is already running.');
      return;
    }
    client = createClient(vscode);
    await client.start();
    if (notify) vscode.window.showInformationMessage('tetherscript language server started.');
  }

  async function stop(notify = true) {
    if (!client || !client.needsStop()) {
      client = undefined;
      if (notify) vscode.window.showInformationMessage('tetherscript language server is not running.');
      return;
    }
    const current = client;
    const child = current._serverProcess;
    client = undefined;
    try {
      await current.stop(750);
      if (notify) vscode.window.showInformationMessage('tetherscript language server stopped.');
    } catch (error) {
      if (alive(child)) child.kill();
      if (notify) vscode.window.showWarningMessage('tetherscript language server was force stopped.');
    }
  }

  async function restart() {
    await stop(false);
    await start(false);
    vscode.window.showInformationMessage('tetherscript language server restarted.');
  }

  function register(context) {
    context.subscriptions.push(
      vscode.commands.registerCommand('tetherscript.startLsp', () => start()),
      vscode.commands.registerCommand('tetherscript.stopLsp', () => stop()),
      vscode.commands.registerCommand('tetherscript.restartLsp', () => restart()),
    );
  }

  return { register, restart, start, stop };
}

module.exports = { createLspController };
