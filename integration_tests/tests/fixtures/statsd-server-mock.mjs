#!/usr/bin/env node

import dgram from "node:dgram";
import { createServer } from "node:http";
import { parseArgs } from "node:util";

const {
  values: { managementPort = "9125", exposePort = "9102" },
} = parseArgs({
  args: process.argv.slice(2),
  options: {
    managementPort: { type: "string" },
    exposePort: { type: "string" },
  },
});

const metrics = {
  last_update: null,
  values: {},
};

const udpServer = dgram.createSocket("udp4");

udpServer.on("message", (msg, rinfo) => {
  const text = msg.toString().trim();
  console.log(`Received: ${text} from ${rinfo.address}:${rinfo.port}`);
  const totalMetrics = text.split("\n");

  for (const oneMetric of totalMetrics) {
    const [name, typeData] = oneMetric.split(":");
    if (!typeData) continue;

    const [rawValue, type] = typeData.split("|");
    const value = parseInt(rawValue);
    const metric = name.replace(/[^a-zA-Z0-9_]/g, "_");

    if (!metrics.values[metric]) {
      metrics.values[metric] = {
        type,
        value: 0,
      };
      if (type === "g") {
        metrics.values[metric].value = value;
      }
      metrics.last_update = Date.now();
      continue;
    }

    const stored = metrics.values[metric];

    if (stored.value !== value) {
      stored.value = value;
      metrics.last_update = Date.now();
    }
  }
});

udpServer.on("listening", () => {
  const { port } = udpServer.address();
  console.log(`StatsD server listening on port ${port}`);
});

udpServer.bind(parseInt(managementPort));

const server = createServer(function (req, res) {
  const url = new URL(req.url, `http://${req.headers.host}`);

  if (url.pathname === "/health") {
    res.statusCode = 200;
    res.setHeader("Content-Type", "text/plain");
    res.end("OK");
  } else if (url.pathname === "/metrics") {
    const output = new Map();

    for (const [name, { value }] of Object.entries(metrics.values)) {
      output.set(name, value);
    }

    const mapObject = Object.fromEntries(output);
    mapObject.last_update = metrics.last_update;

    res.statusCode = 200;
    res.setHeader("Content-Type", "application/json");
    res.end(JSON.stringify(mapObject));
  } else {
    res.statusCode = 404;
    res.setHeader("Content-Type", "text/plain");
    res.end("Not Found");
  }
});

server.listen(parseInt(exposePort, 10), function () {
  console.log(`Statsd server is listening on port ${parseInt(exposePort)}`);
});
