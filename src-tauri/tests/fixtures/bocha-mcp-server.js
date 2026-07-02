// Bocha-format MCP server for E2E testing.
// Returns search results in the same markdown format as the real Bocha script.

const readline = require('readline');

const rl = readline.createInterface({ input: process.stdin });

rl.on('line', (line) => {
  let req;
  try {
    req = JSON.parse(line);
  } catch {
    return;
  }

  if (req.method === 'initialize') {
    process.stdout.write(JSON.stringify({
      jsonrpc: '2.0',
      id: req.id,
      result: {
        protocolVersion: '2024-11-05',
        serverInfo: { name: 'bocha-search', version: '1.0.0' },
        capabilities: { tools: {} },
      },
    }) + '\n');
  } else if (req.method === 'notifications/initialized') {
    // No response
  } else if (req.method === 'tools/list') {
    process.stdout.write(JSON.stringify({
      jsonrpc: '2.0',
      id: req.id,
      result: {
        tools: [{
          name: 'bocha_search',
          description: 'Search the web',
          inputSchema: {
            type: 'object',
            properties: {
              query: { type: 'string' },
              count: { type: 'number', default: 5 },
            },
            required: ['query'],
          },
        }],
      },
    }) + '\n');
  } else if (req.method === 'tools/call') {
    const query = req.params?.arguments?.query || 'unknown';
    const text = `Search results for "${query}":\n\n` +
      `1. [Example Domain](https://example.com)\n` +
      `   This is an example domain for testing\n` +
      `   Published: 2024-01-01\n\n` +
      `2. [Rust 官网](https://www.rust-lang.org)\n` +
      `   Rust 是一门系统编程语言\n\n` +
      `3. [GitHub](https://github.com)\n` +
      `   Where developers build software\n` +
      `   Published: 2024-06-15`;
    process.stdout.write(JSON.stringify({
      jsonrpc: '2.0',
      id: req.id,
      result: {
        content: [{ type: 'text', text }],
      },
    }) + '\n');
  }
});
