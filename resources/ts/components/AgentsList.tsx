import React, { CSSProperties } from "react";

import { type Agent } from "../schemas/Agent";
import { AgentIssuesPreviewButton } from "./AgentIssuesPreviewButton";
import { DownloadProgress } from "./DownloadProgress";
import { ModelMetadataPreviewButton } from "./ModelMetadataPreviewButton";

import {
  agentList,
  agentList__model,
  agentList__progress,
  agentsTable,
} from "./AgentList.module.css";

function displayLastPathPart(path: string | null | undefined): string {
  if (!path) {
    return "";
  }

  const parts = path.split("/");
  const last = parts.pop();

  if (!last) {
    return "";
  }

  return last;
}

export function AgentsList({
  agents,
  managementAddr,
}: {
  agents: Array<Agent>;
  managementAddr: string;
}) {
  return (
    <table className={agentsTable}>
      <thead>
        <tr>
          <th>Name</th>
          <th>Model</th>
          <th>Issues</th>
          <th>Status</th>
          <th>Slots usage</th>
          <th>Used/Actual/Desired</th>
        </tr>
      </thead>
      <tbody>
        {agents.map(function ({
          id,
          desired_slots_total,
          is_state_applied,
          issues,
          model_path,
          name,
          download_current,
          download_filename,
          download_total,
          slots_processing,
          slots_total,
        }: Agent) {
          return (
            <tr key={id}>
              <td>{name}</td>
              <td>
                {"string" === typeof model_path ? (
                  <div className={agentList__model}>
                    🪺
                    <abbr title={model_path}>
                      {displayLastPathPart(model_path)}
                    </abbr>
                    <ModelMetadataPreviewButton
                      agentId={id}
                      agentName={name}
                      managementAddr={managementAddr}
                    />
                  </div>
                ) : (
                  <div className={agentList__model}>
                    🪹 <i>No model loaded</i>
                  </div>
                )}
              </td>
              <td>
                {issues.length > 0 ? (
                  <AgentIssuesPreviewButton agentName={name} issues={issues} />
                ) : (
                  <i>None</i>
                )}
              </td>
              <td>
                {!is_state_applied &&
                  (issues.length > 0 ? (
                    <>
                      🙁 <i>Issues; pending changes blocked</i>
                    </>
                  ) : (
                    <>
                      ⏳ <i>Changes pending</i>
                    </>
                  ))}
                {download_total > 0 && (
                  <DownloadProgress
                    current={download_current}
                    filename={download_filename}
                    total={download_total}
                  />
                )}
              </td>
              <td
                className={agentList}
                style={
                  {
                    "--slots-usage": `${((slots_total - slots_processing) / slots_total) * 100}%`,
                  } as CSSProperties
                }
              >
                {slots_total > 0 && <div className={agentList__progress}></div>}
              </td>
              <td>
                {slots_processing}/{slots_total}/{desired_slots_total}
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
