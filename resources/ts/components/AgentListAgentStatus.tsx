import React, { CSSProperties } from "react";

import { type Agent } from "../schemas/Agent";

import {
  agentListAgentStatus__pendingChanges,
  agentListAgentStatus__progress,
} from "./AgentListAgentStatus.module.css";

export function AgentListAgentStatus({
  agent: {
    desired_slots_total,
    is_state_applied,
    download_current,
    download_filename,
    download_total,
    slots_processing,
    slots_total,
  },
}: {
  agent: Agent;
}) {
  if (is_state_applied) {
    return (
      <div
        className={agentListAgentStatus__progress}
        style={
          {
            "--slots-usage": `${slots_total > 0 ? (slots_processing / slots_total) * 100 : 0}%`,
          } as CSSProperties
        }
      >
        <progress
          value={slots_processing}
          max={slots_total}
          title={`${slots_processing} of ${slots_total} slots used`}
        />
        <abbr title="Slots processing / total / desired total">
          {slots_processing}/{slots_total}/{desired_slots_total}
        </abbr>
      </div>
    );
  }

  if (download_total > 0) {
    return (
      <div className={agentListAgentStatus__progress}>
        <progress max={download_total} value={download_current} />
        <div>Downloading: {download_filename}</div>
      </div>
    );
  }

  return (
    <div className={agentListAgentStatus__progress}>
      <div className={agentListAgentStatus__pendingChanges}>
        ⏳ <i>Changes pending</i>
      </div>
    </div>
  );
}
