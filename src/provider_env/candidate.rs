//! Known OpenAI-compatible provider environment mappings.

use super::vars;

pub(super) struct Candidate {
    pub id: &'static str,
    pub key_vars: &'static [&'static str],
    pub base_vars: &'static [&'static str],
    pub org_vars: &'static [&'static str],
    pub default_base: &'static str,
}

pub(super) fn named(id: &str) -> Option<&'static Candidate> {
    all().iter().find(|candidate| candidate.id == id)
}

pub(super) fn first_configured() -> Option<&'static Candidate> {
    all()
        .iter()
        .find(|candidate| vars::first(candidate.key_vars).is_some())
}

pub(super) fn all() -> &'static [Candidate] {
    &[
        Candidate {
            id: "openai",
            key_vars: &["OPENAI_API_KEY"],
            base_vars: &["OPENAI_BASE_URL", "OPENAI_API_BASE"],
            org_vars: &["OPENAI_ORGANIZATION", "OPENAI_ORG_ID"],
            default_base: "https://api.openai.com/v1",
        },
        Candidate {
            id: "openrouter",
            key_vars: &["OPENROUTER_API_KEY"],
            base_vars: &["OPENROUTER_BASE_URL"],
            org_vars: &[],
            default_base: "https://openrouter.ai/api/v1",
        },
        Candidate {
            id: "cerebras",
            key_vars: &["CEREBRAS_API_KEY"],
            base_vars: &["CEREBRAS_BASE_URL"],
            org_vars: &[],
            default_base: "https://api.cerebras.ai/v1",
        },
        Candidate {
            id: "zai",
            key_vars: &["ZAI_API_KEY", "ZHIPUAI_API_KEY"],
            base_vars: &["ZAI_BASE_URL", "ZHIPUAI_BASE_URL"],
            org_vars: &[],
            default_base: "https://open.bigmodel.cn/api/paas/v4",
        },
    ]
}
