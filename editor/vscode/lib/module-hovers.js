const index = require('./module-index');
const syntax = require('./module-syntax');

async function provide(vscode, document, cursor) {
  const range = document.getWordRangeAtPosition(cursor, /[A-Za-z_][A-Za-z0-9_]*/);
  if (!range) return undefined;
  const word = document.getText(range);
  const declarations = syntax.imports(document.getText());
  let declaration = declarations.find((item) => item.alias === word);
  let member;
  if (!declaration) {
    const before = document.lineAt(cursor).text.slice(0, range.start.character);
    const owner = before.match(/([A-Za-z_][A-Za-z0-9_]*)\.$/);
    declaration = owner && declarations.find((item) => item.alias === owner[1]);
    member = word;
  }
  if (!declaration) return undefined;
  const result = await index.imported(vscode, document, declaration);
  if (!result.module) return undefined;
  const markdown = new vscode.MarkdownString();
  if (!member) {
    markdown.appendCodeblock(`import "${declaration.path}" as ${declaration.alias}`, 'tetherscript');
    markdown.appendMarkdown(`${result.module.exports.length} explicit exports.`);
  } else {
    const value = result.module.exports.find((item) => item.name === member);
    if (!value) return undefined;
    markdown.appendCodeblock(`${declaration.alias}.${value.signature}`, 'tetherscript');
    markdown.appendMarkdown(`Exported by \`${declaration.path}\`.`);
  }
  return new vscode.Hover(markdown, range);
}

function registerModuleHovers(vscode, context) {
  const selector = { scheme: 'file', language: 'tetherscript' };
  context.subscriptions.push(vscode.languages.registerHoverProvider(selector, {
    provideHover: (document, position) => provide(vscode, document, position),
  }));
}

module.exports = { provide, registerModuleHovers };