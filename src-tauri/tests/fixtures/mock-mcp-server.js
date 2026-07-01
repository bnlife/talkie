// Mock MCP server for integration testing.
// Responds to JSON-RPC requests over stdio.

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
        serverInfo: { name: 'mock-mcp', version: '1.0.0' },
        capabilities: { tools: {} },
      },
    }) + '\n');
  } else if (req.method === 'notifications/initialized') {
    // No response for notifications
  } else if (req.method === 'tools/list') {
    process.stdout.write(JSON.stringify({
      jsonrpc: '2.0',
      id: req.id,
      result: {
        tools: [{
          name: 'mock_search',
          description: 'Mock search tool',
          inputSchema: { type: 'object', properties: { query: { type: 'string' } } },
        }],
      },
    }) + '\n');
  } else if (req.method === 'tools/call') {
    const query = req.params?.arguments?.query || 'unknown';
    process.stdout.write(JSON.stringify({
      jsonrpc: '2.0',
      id: req.id,
      result: {
        content: [{
          type: 'text',
          text: JSON.stringify({
            results: [
              { name: 'Test Result 1', url: 'https://example.com/1', snippet: `Search result for: ${query}` },
              { name: 'Test Result 2', url: 'https://example.com/2', snippet: 'Another result' },
            ],
          }),
        }],
      },
    }) + '\n');
  }
});
