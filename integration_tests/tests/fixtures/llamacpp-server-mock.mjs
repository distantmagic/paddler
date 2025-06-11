#!/usr/bin/env node

import { createServer } from 'node:http';
import { parseArgs } from 'node:util';

function toggle_value(array, toggle_value) {
  const indexes = array
    .map((val, i) => val === toggle_value ? i : -1)
    .filter(i => i !== -1);

  if (indexes.lenght === 0) return;

  const randomIndex = indexes[Math.floor(Math.random() * indexes.length)];
  array[randomIndex] = !array[randomIndex];
}

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

let slotsNum = [];
for (let i = 0; i < parseInt(slots); i++) {
  slotsNum.push(false);
}

const server = createServer(async function (req, res) {
  console.log('Request:', req.method, req.url);

  if (req.url === "/chat/completions") {
    if (slots.length <= 0) {
      res.setHeader("Content-Type", "application/json");
      res.end(JSON.stringify({ status: "no slots available" }));
    } else {
      toggle_value(slotsNum, false);
      await new Promise(resolve => setTimeout(resolve, 5000));
      toggle_value(slotsNum, true);  
      res.statusCode = 200;
      res.setHeader("Content-Type", "application/json");
      res.end('{}');
    }    
  } else if (req.url === '/health') {
    res.statusCode = 200;
    res.setHeader('Content-Type', 'text/plain');
    res.end('OK');
  } else if (req.url === '/slots') {
    const slotsStatuses = [];

    for (let i = 0; i < slotsNum.length; i += 1) {
      slotsStatuses.push({
        id: i,
        is_processing: slotsNum[i],
      });
    }

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
