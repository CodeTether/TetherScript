//! Request option keys forwarded to OpenAI-compatible providers.

pub(crate) const PASSTHROUGH: &[&str] = &[
    "temperature",
    "stream",
    "top_p",
    "reasoning_effort",
    "service_tier",
    "clear_thinking",
    "tools",
    "tool_choice",
    "parallel_tool_calls",
];
