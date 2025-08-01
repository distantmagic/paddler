import React, { useCallback, useState } from "react";

import { ChatTemplatesPageTemplatesStream } from "./ChatTemplatesPageTemplatesStream";
import { ChatTemplatesPageToolbar } from "./ChatTemplatesPageToolbar";
import { CodeEditor } from "./CodeEditor";

import {
  chatTemplatesPage,
  chatTemplatesPage__editor,
  chatTemplatesPage__templates,
  chatTemplatesPage__toolbar,
} from "./ChatTemplatesPage.module.css";

export function ChatTemplatesPage({
  managementAddr,
}: {
  managementAddr: string;
}) {
  const [editorValue, setEditorValue] = useState(`{% set name = "World" %}`);

  const onChange = useCallback(
    function (value: string) {
      setEditorValue(value);
    },
    [setEditorValue],
  );

  return (
    <div className={chatTemplatesPage}>
      <div className={chatTemplatesPage__templates}>
        <ChatTemplatesPageTemplatesStream managementAddr={managementAddr} />
      </div>
      <div className={chatTemplatesPage__toolbar}>
        <ChatTemplatesPageToolbar />
      </div>
      <div className={chatTemplatesPage__editor}>
        <CodeEditor editable={true} onChange={onChange} value={editorValue} />
      </div>
    </div>
  );
}
