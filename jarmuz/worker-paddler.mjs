import notifier from "node-notifier";

import { spawner } from "jarmuz/job-types";

spawner(async function ({ buildId, command }) {
  notifier.notify({
    title: "paddler",
    message: `Build ${buildId} finished`,
    timeout: 1,
  });

  const results = await Promise.all([
    command(`
      target/debug/paddler balancer
        --management-addr 127.0.0.1:8060
        --web-dashboard-enable
        --web-dashboard-addr 127.0.1:8061
        --reverseproxy-addr 127.0.0.1:8062
    `),
    command(`
      target/debug/paddler agent
        --management-addr 127.0.0.1:8060
        --name agent-1
        --local-llamacpp-addr 127.0.0.1:8050
    `),
    command(`
      target/debug/paddler agent
        --management-addr 127.0.0.1:8060
        --name agent-2
        --local-llamacpp-addr 127.0.0.1:8051
    `),
  ]);

  for (const result of results) {
    if (!result) {
      return false;
    }
  }
});
