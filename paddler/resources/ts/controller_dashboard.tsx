import React from "react";
import { createRoot } from "react-dom/client";

import { Dashboard } from "./components/Dashboard";

const rootNode = document.getElementById("paddler-dashboard");

if (!rootNode) {
  throw new Error("Root node not found");
}

const root = createRoot(rootNode);

root.render(<Dashboard />);
