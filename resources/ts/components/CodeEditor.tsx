import { jinja } from "@codemirror/lang-jinja";
import CodeMirror from "@uiw/react-codemirror";
import React from "react";

import { codeEditor } from "./CodeEditor.module.css";

export function CodeEditor({
  editable,
  value,
}:
  | {
      editable: true;
      value: string;
      onChange(this: void, value: string): void;
    }
  | {
      editable: false;
      value: string;
    }) {
  return (
    <div className={codeEditor}>
      <CodeMirror
        editable={editable}
        extensions={[jinja()]}
        height="100%"
        readOnly={true}
        value={value}
      />
    </div>
  );
}
