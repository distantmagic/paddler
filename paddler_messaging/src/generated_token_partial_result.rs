use llama_cpp_bindings_types::ParsedToolCall;

use crate::raw_tool_call_tokens::RawToolCallTokens;

#[derive(Debug, Eq, PartialEq)]
pub enum GeneratedTokenPartialResult {
    ContentToken(String),
    ReasoningToken(String),
    ToolCallParseFailed(String),
    ToolCallParsed(Vec<ParsedToolCall>),
    ToolCallToken(String),
    ToolCallValidationFailed(Vec<String>),
    UndeterminableToken(String),
    UnrecognizedToolCallFormat(RawToolCallTokens),
}
