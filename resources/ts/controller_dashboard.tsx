import React from "react";
import { createRoot } from "react-dom/client";

import { Dashboard } from "./components/Dashboard";

class RootNode {
  constructor(private rootNodeElement: HTMLElement) {}

  getFromDataset(key: string): string {
    const value = this.rootNodeElement.dataset[key];

    if (value === undefined) {
      throw new Error(`Missing dataset key: ${key}`);
    }

    return value;
  }
}

const rootNodeElement = document.getElementById("paddler-dashboard");

if (!rootNodeElement) {
  throw new Error("Root node not found");
}

const rootNode = new RootNode(rootNodeElement);

const root = createRoot(rootNodeElement);

root.render(
  <Dashboard
    inferenceAddr={rootNode.getFromDataset("inferenceAddr")}
    managementAddr={rootNode.getFromDataset("managementAddr")}
  />,
);
