#!/usr/bin/env node

import express from 'express';
import axios from 'axios';
import { parseArgs } from 'node:util';

const {
    values: {
      target = "9102",
      managementPort = "9090",
      scrapeInterval = "1",
    },
  } = parseArgs({
    args: process.argv.slice(2),
    options: {
      managementPort: { type: 'string' },
      target: { type: 'string' },
      scrapeInterval: { type: 'string' }
    },
  });

const app = express();
let cachedMetrics = '';

async function scrapeMetrics() {
  try {
    const response = await axios.get(`http://localhost:${target}/metrics`);
    cachedMetrics = response.data;
    console.log('Successfully scraped metrics from StatsD exporter');
  } catch (error) {
    console.error('Error scraping metrics:', error.message);
    cachedMetrics = '# ERROR: Failed to scrape metrics from StatsD exporter';
  }
}

scrapeMetrics();
setInterval(scrapeMetrics, parseInt(scrapeInterval) * 1000);

app.get('/metrics', (req, res) => {
    const { query } = req.query;
    
    if (!query) {
      return res.type('text/plain').send(cachedMetrics);
    }
  
    const lines = cachedMetrics.split('\n');
    let filteredMetrics = '';
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      
      if (line.startsWith('# TYPE ' + query) || 
          line.startsWith(query + ' ')) {
        if (line.startsWith('# TYPE')) {
          filteredMetrics += line + '\n';
          if (i+1 < lines.length) {
            filteredMetrics += lines[i+1] + '\n';
            i++;
          }
        } else {
          filteredMetrics += line + '\n';
        }
      }
    }
  
    if (!filteredMetrics) {
      return res.status(404).type('text/plain').send('# HELP Metric not found\n');
    }
  
    res.type('text/plain').send(filteredMetrics);
  });

app.listen(parseInt(managementPort), "localhost");

