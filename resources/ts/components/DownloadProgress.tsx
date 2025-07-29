import React from "react";

import { downloadProgress } from "./DownloadProgress.module.css";

export function DownloadProgress({
  current,
  filename,
  total,
}: {
  current: number;
  filename: null | string;
  total: number;
}) {
  return (
    <div className={downloadProgress}>
      <progress max={total} value={current} />
      <div>{filename}</div>
    </div>
  );
}
