import React from "react";
import { createRoot } from "react-dom/client";

import { Dashboard } from "./components/Dashboard";

const rootNode = document.getElementById("paddler-dashboard");

if (!rootNode) {
  throw new Error("Root node not found");
}

const managementAddr = rootNode.dataset["managementAddr"];

console.log(rootNode.dataset);

if ("string" !== typeof managementAddr) {
  throw new Error("Management address not found in root node data attributes");
}

const root = createRoot(rootNode);

root.render(<Dashboard managementAddr={managementAddr} />);
