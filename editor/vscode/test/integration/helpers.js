const assert = require('assert');

function label(item) {
  return typeof item.label === 'string' ? item.label : item.label.label;
}

async function waitFor(check, timeout = 5000) {
  const end = Date.now() + timeout;
  while (Date.now() < end) {
    const value = await check();
    if (value) return value;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.fail('Timed out waiting for real VS Code provider result.');
}

module.exports = { assert, label, waitFor };