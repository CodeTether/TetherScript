function completionContext(document, position) {
  const prefix = document.lineAt(position).text.slice(0, position.character);
  if (/\bresource\.[A-Za-z_][A-Za-z0-9_]*$|\bresource\.$/.test(prefix)) {
    return 'resource';
  }
  if (/\.[A-Za-z_][A-Za-z0-9_]*$|\.$/.test(prefix)) {
    return 'member';
  }
  return 'regular';
}

module.exports = { completionContext };
