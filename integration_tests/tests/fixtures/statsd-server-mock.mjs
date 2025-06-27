#!/usr/bin/env node

import dgram from "node:dgram";
import { appendFile } from "node:fs";
import { createServer } from "node:http";
import { parseArgs } from "node:util";

const {
  values: { managementPort = "9125", exposePort = "9102", logFile },
} = parseArgs({
  args: process.argv.slice(2),
  options: {
    managementPort: { type: "string" },
    exposePort: { type: "string" },
    logFile: { type: "string" },
  },
});

const metrics = (1, 2);

const udpServer = dgram.createSocket("udp4");

udpServer.on("message", (msg, rinfo) => {
  const text = msg.toString().trim();
  console.log(`Received: ${text} from ${rinfo.address}:${rinfo.port}`);
  const totalMetrics = text.split("\n");

  for (const oneMetric of totalMetrics) {
    const [name, typeData] = oneMetric.split(":");
    if (!typeData) return;

    const [rawValue, type] = typeData.split("|");
    const value = parseInt(rawValue);
    const metric = name.replace(/[^a-zA-Z0-9_]/g, "_");

    if (!metrics[metric]) {
      metrics[metric] = { type, value: 0 };
    }

    switch (type) {
      case "c":
        metrics[metric].value += value;
        break;
      case "g":
        metrics[metric].value = value;
        break;
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
    const query = url.searchParams.get("query");
    const output = new Map();

    for (const [name, { value }] of Object.entries(metrics)) {
      if (!query || name === query) {
        output.set(name, value);
      }
    }

    const mapObject = Object.fromEntries(output);

    appendFile(logFile, JSON.stringify(mapObject) + "\n", function (err) {
      if (err) {
        res.statusCode = 500;
        res.setHeader("Content-Type", "text/plain");
        res.end(String(err));
      } else {
        res.statusCode = 200;
        res.setHeader("Content-Type", "application/json");
        res.end("OK");
      }
    });
  } else {
    res.statusCode = 404;
    res.setHeader("Content-Type", "text/plain");
    res.end("Not Found");
  }
});

server.listen(parseInt(exposePort, 10), function () {
  console.log(`Statsd server is listening on port ${parseInt(exposePort)}`);
});
