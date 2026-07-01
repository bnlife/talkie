// Bocha-format MCP server that returns no results.

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
        serverInfo: { name: 'bocha-search-empty', version: '1.0.0' },
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
          inputSchema: { type: 'object', properties: { query: { type: 'string' } }, required: ['query'] },
        }],
      },
    }) + '\n');
  } else if (req.method === 'tools/call') {
    const query = req.params?.arguments?.query || 'unknown';
    process.stdout.write(JSON.stringify({
      jsonrpc: '2.0',
      id: req.id,
      result: {
        content: [{ type: 'text', text: `No results found for: ${query}` }],
      },
    }) + '\n');
  }
});
