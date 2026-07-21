function codeOnly(text) {
  const out = [...text];
  let string = false;
  let escaped = false;
  for (let index = 0; index < out.length; index += 1) {
    const char = text[index];
    if (!string && char === '/' && text[index + 1] === '/') {
      while (index < out.length && text[index] !== '\n') {
        out[index] = ' ';
        index += 1;
      }
      continue;
    }
    if (string) {
      if (char === '\n') {
        string = false;
        escaped = false;
        continue;
      }
      out[index] = ' ';
      if (!escaped && char === '"') string = false;
      escaped = !escaped && char === '\\';
      if (char !== '\\') escaped = false;
      continue;
    }
    if (char === '"') {
      string = true;
      escaped = false;
      out[index] = ' ';
    }
  }
  return out.join('');
}

module.exports = { codeOnly };