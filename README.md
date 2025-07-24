# Paddler

Paddler is an open-source, production-ready, stateful load balancer based on [llama.cpp](https://github.com/ggerganov/llama.cpp), designed for LLM deployment at scale.

## Why Paddler

<img align="right" src="https://github.com/user-attachments/assets/19e74262-1918-4b1d-9b4c-bcb4f0ab79f5">

Typical load balancing strategies like round robin and least connections are ineffective for LLM inference workloads, which utilize continuous batching algorithms and unique features like slots.

Paddler embeds llama.cpp and implements its own implementation of the server and slots management system optimized for distributed workloads. It manages multiple slots across distributed agents, maintaining stateful awareness of each slot's availability to ensure efficient request distribution. Paddler supports directing traffic to agents with the relevant KV cache, ensuring proper context management.

> [!NOTE]
> `Slots` are predefined memory slices within the server that handle individual requests. When a request comes in, it is assigned to an available slot for processing. Paddler implements its own slot system where the model is loaded into memory once, and each slot maintains its own KV cache (the KV cache is divided between slots) and can handle its own conversation in parallel.
>

## Key features
* Comes with its own implementation of llama.cpp server and slots management.
* Uses agents to monitor the slots availability and balance the incoming requests.
* Allows for parallel usage of slots and context maintenance, with each slot having its own KV kache. 
* Supports the dynamic addition or removal of instances, enabling integration with autoscaling tools.
* Buffers requests, allowing to scale from zero hosts.
* Integrates with StatsD protocol but also comes with a built-in web admin panel.

{new graphic to be inserted}

## How it Works

{TODO}

## Usage

### Quick Start with Docker Compose

For a quick demonstration of Paddler, see the Docker Compose example in the `example/` directory.

### Installation

Download the latest release for Linux or Mac from the 
[releases page](https://github.com/distantmagic/paddler/releases).

On Linux, if you want Paddler to be accessible system-wide, rename the downloaded executable to `/usr/bin/paddler` (or `/usr/local/bin/paddler`).

{TODO new installation steps}

## Feature Highlights

### KV cache

{TODO}

### Buffered Requests (Scaling from Zero Hosts)

> [!NOTE]
> Available since v0.3.0

Load balancer's buffered requests allow your infrastructure to scale from zero hosts by providing an additional metric (unhandled requests). 

It also gives your infrastructure some additional time to add additional hosts. For example, if your autoscaler is setting up an additional server, putting an incoming request on hold for 60 seconds might give it a chance to be handled even though there might be no available Paddler instances at the moment of issuing it.

Scaling from zero hosts is especially suitable for low-traffic projects because it allows you to cut costs on your infrastructure—you won't be paying your cloud provider anything if you are not using your service at the moment.


#### Web Admin Panel

Paddler needs to be compiled with the `web_dashboard` feature flag enabled (enabled by default in GitHub releases).

To start the dashboard, run `paddler balancer` with the `--management-dashboard-enable` flag.

{TODO: add new screenshot, new flags}

### StatsD Metrics

{TODO: verify metrics}

> [!NOTE]
> Available since v0.3.0

> [!TIP]
> If you keep your stack self-hosted you can use [Prometheus](https://prometheus.io/) with StatsD exporter to handle the incoming metrics.

> [!TIP]
> This feature works with [AWS CloudWatch Agent](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-custom-metrics-statsd.html) as well.

Paddler supports the following StatsD metrics:
- `requests_buffered` number of buffered requests since the last report (resets after each report)
- `slots_idle` total idle slots
- `slots_processing` total slots processing requests

All of them use `gauge` internally.

StatsD metrics need to be enabled with the following flags:

```shell
./paddler balancer \
    # .. put all the other flags here ...
    --statsd-addr=127.0.0.1:8125
```

If you do not provide the `--statsd-addr` flag, the StatsD metrics will not be collected.


## Changelog

### v2.0.0 (in progress)

> [!IMPORTANT]
> This release no longer uses `llama-server`. Instead, we bundle the `llama.cpp` codebase directly into Paddler.
> We only use `llama.cpp` as a library for inference and have reimplemented `llama-server` functionality within Paddler itself.
> Instead of `llama-server`, you can use `paddler agent`, and you no longer need to run `llama-server` separately, which significantly simplifies the setup.

#### Breaking Changes

- Change the agent object structure (which in turn changes the response from the API)
- Move web dashboard to its own host (to separate it from the management API)
- Rename the `agents` endpoint from `/agents` to `/api/v1/agents`
- Use milliseconds instead of seconds in CLI arguments for the following flags:
    - `--monitoring-interval` in `agents`
    - `--buffered-request_timeout` and `--statsd-reporting-interval` in `balancer`

#### Features

- Add `--management-cors-allowed-host` repeatable flag to be able to specify the allowed CORS hosts for the management API
- Add `/api/v1/agents/stream` endpoint that streams the updates from the agents in real-time

### v1.2.0

#### Features

- Add TUI dashboard (`paddler dashboard --management-addr [HOST]:[PORT]`) to be able to easily observe balancer instances from the terminal level

### v1.1.0

- More meaningful error messages when the agent can't connect to the llama.cpp slot endpoint, or when slot endpoint is not enabled in llama.cpp
- Set default logging level to `info` for agents and balancer to increase the amount of information in the logs (it wasn't clean if the agent was running or not)
- Enable LTO optimization for the release builds (see [#28](https://github.com/distantmagic/paddler/issues/28))

### v1.0.0

The first stable release! Paddler is now rewritten in Rust and uses the [Pingora](https://github.com/cloudflare/pingora) framework for the networking stack. A few minor API changes and reporting improvements are introduced (documented in the README). API and configuration are now stable, and won't be changed until version `2.0.0`.

This is a stability/quality release. The next plan is to introduce a supervisor who does not just monitor llama.cpp instances, but to also manage them.

Requires llama.cpp version [b4027](https://github.com/ggerganov/llama.cpp/releases/tag/b4027) or above.

### v0.10.0

This update is a minor release to make Paddler compatible with `/slots` endpoint changes introduced in llama.cpp b4027.

Requires llama.cpp version [b4027](https://github.com/ggerganov/llama.cpp/releases/tag/b4027) or above.

### v0.9.0

Latest supported llama.cpp release: [b4026](https://github.com/ggerganov/llama.cpp/releases/tag/b4026)

#### Features

- Add `--local-llamacpp-api-key` flag to balancer to support llama.cpp API keys (see: [#23](https://github.com/distantmagic/paddler/issues/23))

### v0.8.0

#### Features

- Add `--rewrite-host-header` flag to balancer to rewrite the `Host` header in forwarded requests (see: [#20](https://github.com/distantmagic/paddler/issues/20))

### v0.7.1

#### Fixes

- Incorrect preemptive counting of remaining slots in some scenarios

### v0.7.0

Requires at least [b3606](https://github.com/ggerganov/llama.cpp/releases/tag/b3606) llama.cpp release.

#### Breaking Changes

- Adjusted to handle breaking changes in llama.cpp `/health` endpoint: https://github.com/ggerganov/llama.cpp/pull/9056
  
    Instead of using the `/health` endpoint to monitor slot statuses, starting from this version, Paddler uses the `/slots` endpoint to monitor llama.cpp instances.
    Paddler's `/health` endpoint remains unchanged.

### v0.6.0

Latest supported llama.cpp release: [b3604](https://github.com/ggerganov/llama.cpp/releases/tag/b3604)

#### Features

- [Name agents with `--name` flag](https://github.com/distantmagic/paddler/issues/15)

### v0.5.0

#### Fixes

- Management server crashed in some scenarios due to concurrency issues

### v0.4.0

Thank you, [@ScottMcNaught](https://github.com/ScottMcNaught), for the help with debugging the issues! :)

#### Fixes

- OpenAI compatible endpoint is now properly balanced (`/v1/chat/completions`)
- Balancer's reverse proxy `panic`ked in some scenarios when the underlying `llama.cpp` instance was abruptly closed during the generation of completion tokens
- Added mutex in the targets collection for better internal slots data integrity

### v0.3.0

#### Features

* Requests can queue when all llama.cpp instances are busy
* AWS Metadata support for agent local IP address
* StatsD metrics support

### v0.1.0

#### Features

* [Aggregated Health Status Responses](https://github.com/distantmagic/paddler/releases/tag/v0.1.0)

## Contribution guidelines and community

We welcome every contribution, whether it is a submitted issue, PR, or a new discussion topic. All pull requests go through a code review process to maintain code quality and ensure consistency with the project's standards.
Please use GitHub discussions for community conversations. 

## Why the Name

I initially wanted to use [Raft](https://raft.github.io/) consensus algorithm (thus Paddler, because it paddles on a Raft), but eventually, I dropped that idea. The name stayed, though.

Later, people started sending me a "that's a paddlin'" clip from The Simpsons, and I just embraced it.

