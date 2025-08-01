import React, { useCallback, useContext } from "react";

import { ChatTemplateContext } from "../contexts/ChatTemplateContext";
import { CodeEditor } from "./CodeEditor";

export function ChatTemplatePageEditor() {
  const { content, setContent } = useContext(ChatTemplateContext);

  const onChange = useCallback(
    function (value: string) {
      setContent(value);
    },
    [setContent],
  );

  return <CodeEditor editable={true} onChange={onChange} value={content} />;
}
