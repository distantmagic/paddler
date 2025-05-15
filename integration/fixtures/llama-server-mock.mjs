#!/usr/bin/env node

import { basename } from 'node:path';
import { createServer} from 'node:http';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const filename = fileURLToPath(import.meta.url);
const whoami = basename(filename, ".mjs");

function toggle_value(array, toggle_value) {
  const i = Math.floor(Math.random() * array.lenght);

  const indexes = array
    .map((val, i) => val === toggle_value ? i : -1)
    .filter(i => i !== -1);

  if (indexes.length === 0) return;

  const randomIndex = indexes[Math.floor(Math.random() * indexes.length)];

  array[randomIndex] = !array[randomIndex];
}

function serve() {
  const {
    values: {
      port,
      np,
      version,
    },
  } = parseArgs({
    args: process.argv.slice(3),
    options: {
      port: {
        type: 'string',
        default: '8080',
      },
      np: {
        type: 'string',
        default: '1',
      },
      ["filebase-base-directory"]: {
        type: 'string',
        default: '/tmp',
      },
    },
  });

  const addr = parseInt(port, 10);

  let slots = [];
  for (let i = 0; i < parseInt(np, 10); i++) {
    slots.push(false);
  }

  const server = createServer(async function (req, res) {
    console.error('Request:', req.method, req.url);

    res.statusCode = 200;
    res.setHeader('Content-Type', 'text/plain');

    if (req.url === "/health" || req.url === "/") {
      res.setHeader("Content-Type", "application/json");
      res.end(JSON.stringify('{"status":"ok"}'));
    } 
    else if (req.url === "/chat/completions" || req.url === "/v1/chat/completions" || req.url === "/completion") {
      if (slots.length <= 0) {
        res.setHeader("Content-Type", "application/json");
        res.end(JSON.stringify("Unsuccessful response"));
      } else {
        toggle_value(slots, false)
        await new Promise(resolve => setTimeout(resolve, 5000));
        toggle_value(slots, true)
        res.setHeader("Content-Type", "application/json");
        res.end(JSON.stringify("Successful response"));
      }
    } 
    else if (req.url === "/slots") {
      let slots_message = []
      let slots_lenght = Object.keys(slots).length
      for (let i = 0; i < slots_lenght; i++) {
        slots_message.push({
          id: i,
          id_task: -1,
          n_ctx: 4096,
          speculative: false,
          is_processing: slots[i],
          non_causal: false,
          params: {
            n_predict: -1,
            seed: 4294967295,
            temperature: 0.800000011920929,
            dynatemp_range: 0.0,
            dynatemp_exponent: 1.0,
            top_k: 40,
            top_p: 0.949999988079071,
            min_p: 0.05000000074505806,
            xtc_probability: 0.0,
            xtc_threshold: 0.10000000149011612,
            typical_p: 1.0,
            repeat_last_n: 64,
            repeat_penalty: 1.0,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
            dry_multiplier: 0.0,
            dry_base: 1.75,
            dry_allowed_length: 2,
            dry_penalty_last_n: 4096,
            dry_sequence_breakers: ["\n", ":", "\"", "*"],
            mirostat: 0,
            mirostat_tau: 5.0,
            mirostat_eta: 0.10000000149011612,
            stop: [],
            max_tokens: -1,
            n_keep: 0,
            n_discard: 0,
            ignore_eos: false,
            stream: true,
            logit_bias: [],
            n_probs: 0,
            min_keep: 0,
            grammar: "",
            grammar_lazy: false,
            grammar_triggers: [],
            preserved_tokens: [],
            chat_format: "Content-only",
            samplers: ["penalties", "dry", "top_k", "typ_p", "top_p", "min_p", "xtc", "temperature"],
            "speculative.n_max": 16,
            "speculative.n_min": 0,
            "speculative.p_min": 0.75,
            timings_per_token: false,
            post_sampling_probs: false,
            lora: []
          },
          prompt: "",
          next_token: {
            has_next_token: true,
            has_new_line: false,
            n_remain: -1,
            n_decoded: 0,
            stopping_word: ""
          }
        });
      }
      res.setHeader("Content-Type", "application/json");
      res.end(JSON.stringify(slots_message));
    } 
    else {
      res.setHeader("Content-Type", "application/json");
      res.end(JSON.stringify('{"error":{"code":404,"message":"File Not Found","type":"not_found_error"}}'));
    }
  });

  server.listen(addr, function () {
    console.log(`Server running at ${addr}`);
  });
}

if (process.argv.includes('--version')) {
  console.log(`llama-server-mock (${whoami}) v0.1.0`);
} else {
  serve();
}