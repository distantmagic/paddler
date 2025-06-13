#!/usr/bin/env node

import { parseArgs } from 'node:util';
import dgram from 'node:dgram';
import express from 'express';

const {
  values: {
    managementPort = "9125",
    exposePort = "9102",
  },
} = parseArgs({
  args: process.argv.slice(2),
  options: {
    managementPort: { type: 'string' },
    exposePort: { type: 'string' },
  },
});

const metrics = {};

const udpServer = dgram.createSocket('udp4');

udpServer.on('message', (msg, rinfo) => {
  const text = msg.toString().trim();
  console.log(`Received: ${text} from ${rinfo.address}:${rinfo.port}`);
  const totalMetrics = text.split('\n');

  for (const oneMetric of totalMetrics) {
    const [name, typeData] = oneMetric.split(':');
    if (!typeData) return;
  
    const [rawValue, type] = typeData.split('|');
    const value = parseInt(rawValue);
    const metric = name.replace(/[^a-zA-Z0-9_]/g, '_');
  
    if (!metrics[metric]) {
      metrics[metric] = { type, value: 0 };
    }
  
    switch (type) {
      case 'c':
        metrics[metric].value += value;
        break;
      case 'g':
        metrics[metric].value = value;
        break;
    }
  }
});

udpServer.on('listening', () => {
  const { _address, port } = udpServer.address();
  console.log(`StatsD server listening on port ${port}`);
});

udpServer.bind(parseInt(managementPort));
udpServer.liste

const app = express();

app.get('/metrics', (_req, res) => {
  let output = '';

  for (const [name, { type, value }] of Object.entries(metrics)) {
    output += `${name} ${value}\n`;
  }

  res.type('text/plain').send(output);
});

app.get("/health", (_req, res) => {
  res.statusCode = 200;
  res.setHeader('Content-Type', 'text/plain');
  res.end('OK');
})

app.listen(parseInt(exposePort), "127.0.0.1");
