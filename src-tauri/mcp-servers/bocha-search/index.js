#!/usr/bin/env node
// Bocha Search MCP Server
// A minimal MCP server that calls the Bocha Web Search API.

const BOCHA_API_KEY = process.env.BOCHA_API_KEY;
console.error(`[BOCHA] init | key_len=${BOCHA_API_KEY?.length || 0} key_empty=${!BOCHA_API_KEY}`);
if (!BOCHA_API_KEY) {
  process.stderr.write('ERROR: BOCHA_API_KEY environment variable is required\n');
  process.exit(1);
}

const BOCHA_API_URL = 'https://api.bochaai.com/v1/web-search';

let pendingRequests = 0;
let stdinEnded = false;
let keepAliveTimer = null;

// Read all stdin, process line by line
let buffer = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => {
  buffer += chunk;
  // Process complete lines
  let newlineIdx;
  while ((newlineIdx = buffer.indexOf('\n')) !== -1) {
    const line = buffer.slice(0, newlineIdx).trim();
    buffer = buffer.slice(newlineIdx + 1);
    if (line) handleLine(line);
  }
});

process.stdin.on('end', () => {
  stdinEnded = true;
  // Process any remaining buffer
  if (buffer.trim()) handleLine(buffer.trim());
  buffer = '';
  maybeExit();
});

function handleLine(line) {
  let req;
  try {
    req = JSON.parse(line);
  } catch {
    return;
  }

  const id = req.id;

  if (req.method === 'initialize') {
    respond(id, {
      protocolVersion: '2024-11-05',
      serverInfo: { name: 'bocha-search', version: '1.0.0' },
      capabilities: { tools: {} },
    });
  } else if (req.method === 'notifications/initialized') {
    // No response for notifications
  } else if (req.method === 'tools/list') {
    respond(id, {
      tools: [{
        name: 'bocha_search',
        description: 'Search the web using Bocha AI Search API. Returns web page titles, URLs, and snippets.',
        inputSchema: {
          type: 'object',
          properties: {
            query: { type: 'string', description: 'The search query' },
            count: { type: 'number', description: 'Number of results (1-10)', default: 5 },
            freshness: {
              type: 'string',
              description: 'Time range filter',
              enum: ['noLimit', 'oneDay', 'oneWeek', 'oneMonth', 'oneYear'],
              default: 'noLimit',
            },
          },
          required: ['query'],
        },
      }],
    });
  } else if (req.method === 'tools/call') {
    const args = req.params?.arguments || {};
    const query = args.query || '';
    const count = args.count || 5;
    const freshness = args.freshness || 'noLimit';

    pendingRequests++;
    startKeepAlive();
    bochaSearch(query, count, freshness)
      .then((result) => {
        respond(id, { content: [{ type: 'text', text: result }] });
      })
      .catch((err) => {
        respondError(id, -32000, `Search failed: ${err.message}`);
      })
      .finally(() => {
        pendingRequests--;
        maybeExit();
      });
  } else {
    respondError(id, -32601, `Method not found: ${req.method}`);
  }
}

// Keep the process alive while there are pending requests
function startKeepAlive() {
  if (keepAliveTimer) return;
  keepAliveTimer = setInterval(() => {
    if (pendingRequests <= 0 && stdinEnded) {
      clearInterval(keepAliveTimer);
      keepAliveTimer = null;
    }
  }, 100);
}

function maybeExit() {
  if (stdinEnded && pendingRequests <= 0) {
    if (keepAliveTimer) {
      clearInterval(keepAliveTimer);
      keepAliveTimer = null;
    }
    // Let stdout flush, then exit cleanly
    if (process.stdout.writableLength > 0) {
      process.stdout.once('drain', () => process.exit(0));
    } else {
      process.exit(0);
    }
  }
}

function respond(id, result) {
  const msg = JSON.stringify({ jsonrpc: '2.0', id, result }) + '\n';
  process.stdout.write(msg);
}

function respondError(id, code, message) {
  const msg = JSON.stringify({ jsonrpc: '2.0', id, error: { code, message } }) + '\n';
  process.stdout.write(msg);
}

async function bochaSearch(query, count, freshness) {
  const body = JSON.stringify({ query, count, freshness, summary: true });

  const resp = await fetch(BOCHA_API_URL, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${BOCHA_API_KEY}`,
      'Content-Type': 'application/json',
    },
    body,
  });

  if (!resp.ok) {
    const text = await resp.text();
    console.error(`[BOCHA] API error | status=${resp.status} key_len=${BOCHA_API_KEY?.length || 0} key_empty=${!BOCHA_API_KEY} body=${text}`);
    throw new Error(`Bocha API error ${resp.status}: ${text}`);
  }

  const data = await resp.json();
  const pages = data?.data?.webPages?.value || [];

  if (pages.length === 0) {
    return `No results found for: ${query}`;
  }

  const lines = pages.map((p, i) => {
    let line = `${i + 1}. [${p.name}](${p.url})`;
    if (p.snippet) line += `\n   ${p.snippet}`;
    if (p.datePublished) line += `\n   Published: ${p.datePublished}`;
    return line;
  });

  return `Search results for "${query}":\n\n${lines.join('\n\n')}`;
}
