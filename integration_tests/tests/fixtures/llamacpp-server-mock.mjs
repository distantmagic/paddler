#!/usr/bin/env node

import { createServer } from 'node:http';
import { parseArgs } from 'node:util';

const {
  values: {
    port,
    slots,
  },
} = parseArgs({
  args: process.argv.slice(2),
  options: {
    port: {
      type: 'string',
    },
    slots: {
      type: 'string',
    },
  },
});

const slotsInt = parseInt(slots, 10);
const slotsStatuses = [];

for (let i = 0; i < slotsInt; i += 1) {
  slotsStatuses.push({
    id: i,
    is_processing: false,
  });
}

const server = createServer(function (req, res) {
  console.log('Request:', req.method, req.url);

  if (req.url === "/chat/completions") {
    res.statusCode = 200;
    res.setHeader('Content-Type', 'application/json');
    res.end('{}');
  } else if (req.url === '/health') {
    res.statusCode = 200;
    res.setHeader('Content-Type', 'text/plain');
    res.end('OK');
  } else if (req.url === '/slots') {
    res.statusCode = 200;
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify(slotsStatuses));
  } else {
    res.statusCode = 404;
    res.setHeader('Content-Type', 'application/json');
    res.end('{"status":"not found"}');
  }
});

server.listen(parseInt(port, 10), function () {
  console.log(`Server running at ${port}`);
});
