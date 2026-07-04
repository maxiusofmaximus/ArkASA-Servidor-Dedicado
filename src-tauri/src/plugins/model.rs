//! AI model plugins — 8 OpenAI-API-compatible adapters the operator
//! can pick from inside the AI config panel. The catalog is purely
//! declarative — each entry pins the adapter's default base URL,
//! default model name, and whether the adapter needs an API key.
//! Selecting an adapter fills the relevant env vars the existing
//! `integrations::ai::AiConfig` reads at runtime.
//!
//! Why a catalog instead of plugins-with-process-spawning?
//! All 8 adapters speak the OpenAI Chat Completions API. The only
//! real difference between them is which HTTP endpoint and what
//! default model they expect. There's no value in spawning 8
//! different CLIs when a single `reqwest::Client` handles all of
//! them. OpenAI-compatible is a real, broadly-supported contract.
//!
//! Backwards rule: ZERO breaking changes to `integrations::ai`. The
//! catalog is a *front-end* discovery aid; actual request handling
//! stays in `ai::AiClient::ask`.

use serde::Serialize;

/// Stable id for every AI model plugin in the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelPluginId {
    Openai,
    Cerebras,
    NvidiaNim,
    LlamaCpp,
    Ollama,
    Vllm,
    LmStudio,
    Custom,
}

impl ModelPluginId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openai     => "openai",
            Self::Cerebras   => "cerebras",
            Self::NvidiaNim  => "nvidia_nim",
            Self::LlamaCpp   => "llama.cpp",
            Self::Ollama     => "ollama",
            Self::Vllm       => "vllm",
            Self::LmStudio   => "lm_studio",
            Self::Custom     => "custom",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "openai"      => Self::Openai,
            "cerebras"    => Self::Cerebras,
            "nvidia_nim"  => Self::NvidiaNim,
            "llama.cpp"   => Self::LlamaCpp,
            "ollama"      => Self::Ollama,
            "vllm"        => Self::Vllm,
            "lm_studio"   => Self::LmStudio,
            "custom"      => Self::Custom,
            _             => return None,
        })
    }
}

/// Discovery record the React side renders.
#[derive(Debug, Clone)]
pub struct ModelPluginView {
    pub id:                        &'static str,
    pub label:                     &'static str,
    pub description:               &'static str,
    pub default_base_url:          &'static str,
    pub default_model:             &'static str,
    pub requires_api_key:          bool,
    pub is_local:                  bool,
    pub install_hint:              &'static str,
    pub docs_url:                  &'static str,
}

/// Static catalog of every model adapter we recognise. These are the
/// 8 OpenAI-API-compatible providers that have shipped with stable
/// integration tutorials during 2024-2025.
pub const CATALOG: &[ModelPluginView] = &[
    ModelPluginView {
        id: "openai", label: "OpenAI",
        description: "Official OpenAI Chat Completions. Strong models, paid per token.",
        default_base_url: "https://api.openai.com/v1",
        default_model: "gpt-4o-mini",
        requires_api_key: true,
        is_local: false,
        install_hint: "Get a key at https://platform.openai.com/api-keys",
        docs_url: "https://platform.openai.com/docs/api-reference/chat",
    },
    ModelPluginView {
        id: "cerebras", label: "Cerebras Inference",
        description: "Free tier, very fast Llama 3.1 70B. OpenAI-compatible.",
        default_base_url: "https://api.cerebras.ai/v1",
        default_model: "llama3.1-70b",
        requires_api_key: true,
        is_local: false,
        install_hint: "Sign up at https://inference.cerebras.ai/ — get a free API key.",
        docs_url: "https://inference.cerebras.ai/docs",
    },
    ModelPluginView {
        id: "nvidia_nim", label: "NVIDIA NIM",
        description: "NVIDIA-hosted Llama/Mistral models via NIM endpoint. Some free credits.",
        default_base_url: "https://integrate.api.nvidia.com/v1",
        default_model: "meta/llama-3.1-70b-instruct",
        requires_api_key: true,
        is_local: false,
        install_hint: "Sign in at https://build.nvidia.com/ and grab an API key.",
        docs_url: "https://docs.api.nvidia.com/nim/reference",
    },
    ModelPluginView {
        id: "llama.cpp", label: "llama.cpp (local server)",
        description: "Run llama.cpp's built-in server in OpenAI-compat mode. CPU-only baseline.",
        default_base_url: "http://localhost:8080/v1",
        default_model: "Qwen2.5-7B-Instruct-Q4_K_M.gguf",
        requires_api_key: false,
        is_local: true,
        install_hint: "llama-server -m ./my-model.gguf --port 8080 -c 4096 — no API key needed.",
        docs_url: "https://github.com/ggerganov/llama.cpp/blob/master/examples/server/README.md",
    },
    ModelPluginView {
        id: "ollama", label: "Ollama (local)",
        description: "Ollama's built-in OpenAI-compat endpoint on :11434.",
        default_base_url: "http://localhost:11434/v1",
        default_model: "qwen2.5:7b",
        requires_api_key: false,
        is_local: true,
        install_hint: "ollama serve  (then  ollama pull qwen2.5:7b  once).",
        docs_url: "https://github.com/ollama/ollama/blob/main/docs/openai.md",
    },
    ModelPluginView {
        id: "vllm", label: "vLLM (local)",
        description: "vLLM's OpenAI-compat server on :8000. GPU-backed, fast batching.",
        default_base_url: "http://localhost:8000/v1",
        default_model: "Qwen/Qwen2.5-7B-Instruct",
        requires_api_key: false,
        is_local: true,
        install_hint: "vllm serve Qwen/Qwen2.5-7B-Instruct --port 8000",
        docs_url: "https://docs.vllm.ai/en/latest/serving/openai_compatible_server.html",
    },
    ModelPluginView {
        id: "lm_studio", label: "LM Studio (local)",
        description: "LM Studio's local OpenAI-compat server on :1234.",
        default_base_url: "http://localhost:1234/v1",
        default_model: "qwen2.5-7b-instruct",
        requires_api_key: false,
        is_local: true,
        install_hint: "Open LM Studio → Developer tab → Start Server.",
        docs_url: "https://lmstudio.ai/docs/local-server",
    },
    ModelPluginView {
        id: "custom", label: "Custom OpenAI-compat endpoint",
        description: "User-pasted base URL + API key + model string.",
        default_base_url: "https://example.com/v1",
        default_model: "your-model-name",
        requires_api_key: false,
        is_local: false,
        install_hint: "Make sure the endpoint exposes /v1/chat/completions.",
        docs_url: "https://platform.openai.com/docs/api-reference/chat",
    },
];

pub fn all() -> Vec<ModelPluginView> {
    CATALOG.to_vec()
}

pub fn view(id: ModelPluginId) -> &'static ModelPluginView {
    CATALOG.iter().find(|v| v.id == id.as_str())
        .expect("missing ModelPluginView for known id")
}

/// JSON-shared with the React side.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPluginJson {
    pub id:                String,
    pub label:             String,
    pub description:       String,
    pub default_base_url:  String,
    pub default_model:     String,
    pub requires_api_key:  bool,
    pub is_local:          bool,
    pub install_hint:      String,
    pub docs_url:          String,
}

#[tauri::command]
pub fn list_model_plugins() -> Vec<ModelPluginJson> {
    CATALOG.iter().map(|v| ModelPluginJson {
        id:                v.id.to_string(),
        label:             v.label.to_string(),
        description:       v.description.to_string(),
        default_base_url:  v.default_base_url.to_string(),
        default_model:     v.default_model.to_string(),
        requires_api_key:  v.requires_api_key,
        is_local:          v.is_local,
        install_hint:      v.install_hint.to_string(),
        docs_url:          v.docs_url.to_string(),
    }).collect()
}

#[tauri::command]
pub fn get_model_plugin(id: String) -> Result<ModelPluginJson, String> {
    let parsed = ModelPluginId::from_str(&id)
        .ok_or_else(|| format!("unknown model plugin id: {id}"))?;
    let v = view(parsed);
    Ok(ModelPluginJson {
        id:                v.id.to_string(),
        label:             v.label.to_string(),
        description:       v.description.to_string(),
        default_base_url:  v.default_base_url.to_string(),
        default_model:     v.default_model.to_string(),
        requires_api_key:  v.requires_api_key,
        is_local:          v.is_local,
        install_hint:      v.install_hint.to_string(),
        docs_url:          v.docs_url.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_eight_entries() {
        assert_eq!(CATALOG.len(), 8, "expected 8 AI model plugins");
    }

    #[test]
    fn all_ids_unique() {
        let mut seen = std::collections::HashSet::new();
        for v in CATALOG {
            assert!(seen.insert(v.id), "duplicate id {}", v.id);
        }
    }

    #[test]
    fn round_trip_id_to_str() {
        for v in CATALOG {
            let parsed = ModelPluginId::from_str(v.id)
                .unwrap_or_else(|| panic!("unknown id {}", v.id));
            assert_eq!(parsed.as_str(), v.id);
        }
    }

    #[test]
    fn local_models_dont_need_api_key() {
        for v in CATALOG {
            if v.is_local {
                assert!(!v.requires_api_key,
                    "local model `{}` should not require an API key", v.id);
            }
        }
    }

    #[test]
    fn hosted_models_have_https_base_url() {
        for v in CATALOG {
            if !v.is_local {
                assert!(v.default_base_url.starts_with("http"),
                    "hosted model `{}` must use http(s) base URL", v.id);
            }
        }
    }

    #[test]
    fn local_models_default_to_localhost() {
        for v in CATALOG {
            if v.is_local {
                assert!(v.default_base_url.starts_with("http://localhost")
                    || v.default_base_url.starts_with("http://127.0.0.1"),
                    "local model `{}` should default to localhost, got {}",
                    v.id, v.default_base_url);
            }
        }
    }
}
