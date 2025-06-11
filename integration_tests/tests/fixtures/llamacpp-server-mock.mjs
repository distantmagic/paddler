#!/usr/bin/env node

import { appendFile } from 'node:fs';
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
    completionResponseDelay,
    logFile,
    name,
    port,
    slots,
  },
} = parseArgs({
  args: process.argv.slice(2),
  options: {
    completionResponseDelay: {
      type: 'string',
    },
    logFile: {
      type: 'string',
    },
    name: {
      type: 'string',
    },
    port: {
      type: 'string',
    },
    slots: {
      type: 'string',
    },
  },
});

<<<<<<< HEAD
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
=======
const completionResponseDelayInt = parseInt(completionResponseDelay, 10);
const portInt = parseInt(port, 10);
const slotsInt = parseInt(slots, 10);
const slotsStatuses = [];

for (let i = 0; i < slotsInt; i += 1) {
  slotsStatuses.push({
    id: i,
    is_processing: false,
  });
}

const server = createServer(function (req, res) {
  if (req.url === "/chat/completions") {
    const requestName = req.headers['x-request-name'];

    if (!requestName) {
      res.statusCode = 400;
      res.setHeader('Content-Type', 'application/json');
      res.end('{"error":"Missing x-request-name header"}');

      return;
    }

    setTimeout(function () {
      appendFile(logFile, `${name};${requestName}`, function (err) {
        if (err) {
          res.statusCode = 500;
          res.setHeader('Content-Type', 'text/plain');
          res.end(String(err));
        } else {
          res.statusCode = 200;
          res.setHeader('Content-Type', 'application/json');
          res.end('{}');
        }
      });
    }, completionResponseDelayInt * 1000);
>>>>>>> 0b5ebec5ebf8b3486597977e0b6161f885293521
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

server.listen(portInt, function () {
  console.log(`Server ${name} is listening on port ${portInt} (with ${slotsInt} slots)`);
});
