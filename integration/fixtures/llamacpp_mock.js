#!/usr/bin/env node

import { basename } from 'node:path';
import { createServer} from 'node:http';
import { fileURLToPath } from 'node:url';
import { parse } from 'node:url';
import { parseArgs } from 'node:util';
import { unlink } from 'node:fs/promises';

const filename = fileURLToPath(import.meta.url);
const whoami = basename(filename, ".mjs");

function getPort(addr) {
  const [ip, port] = addr.split(':');

  return parseInt(port, 10);
}

function serve() {
  const {
    values: {
      addr,
      version,
    },
  } = parseArgs({
    args: process.argv.slice(3),
    options: {
      addr: {
        type: 'string',
        default: '127.0.0.1:8081',
      },
      ["filebase-base-directory"]: {
        type: 'string',
        default: '/tmp',
      },
    },
  });

  const port = getPort(addr);

  const server = createServer(function (req, res) {
    console.error('Request:', req.method, req.url);

    res.statusCode = 200;
    res.setHeader('Content-Type', 'text/plain');

    if (req.url === "/whoami") {
      res.end(whoami);
    } else {
      res.end('OK');
    }
  });

  server.listen(port, function () {
    console.log(`Server running at ${port}`);
  });
}

if (process.argv.includes('--version')) {
  console.log(`intentt-mock (${whoami}) v0.1.0`);
} else {
  serve();
}