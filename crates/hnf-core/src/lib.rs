//! HNF core — document model and ToolAdapter traits (HummingBird v8).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sidecar_protocol::{SceneGraphEdge, SceneGraphNode};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HnfDocument {
    pub document_uri: String,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default)]
    pub objects: Vec<HnfObject>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HnfObject {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub properties: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HnfMutation {
    pub kind: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneGraphDeltas {
    pub commit_id: String,
    #[serde(default)]
    pub nodes: Vec<SceneGraphNode>,
    #[serde(default)]
    pub edges: Vec<SceneGraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolArtifact {
    pub path: String,
    pub content_type: String,
}

pub trait ToolAdapter {
    type Error;

    fn apply_mutation(
        &self,
        document: &mut HnfDocument,
        mutation: &HnfMutation,
    ) -> Result<SceneGraphDeltas, Self::Error>;

    fn export(
        &self,
        document: &HnfDocument,
        format: &str,
        output_dir: &str,
    ) -> Result<Vec<ToolArtifact>, Self::Error>;
}

/// Shared host-OSS environment flags for sidecars and sim runners.
pub mod host_env {
    pub fn env_flag(name: &str) -> bool {
        std::env::var(name)
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(false)
    }

    /// When set, bridge plugins may spawn host binaries from OSS builds.
    pub fn use_host_oss() -> bool {
        env_flag("HBP_USE_HOST_OSS") || env_flag("HCP_USE_HOST_OSS")
    }

    /// When set, simulation runners resolve default commands via `which` on PATH.
    pub fn sim_use_host() -> bool {
        env_flag("HBP_SIM_USE_HOST") || env_flag("HCP_SIM_USE_HOST")
    }

    pub fn env_or_default(name: &str, default: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| default.to_string())
    }

    pub fn timeout_from_env(name: &str, default_secs: u64) -> std::time::Duration {
        let secs = env_or_default(name, &default_secs.to_string())
            .parse()
            .unwrap_or(default_secs);
        std::time::Duration::from_secs(secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn minimum_document_model_serializes() {
        let doc = HnfDocument {
            document_uri: "hcp://docs/board.kicad".to_string(),
            metadata: json!({"tool": "kicad"}),
            objects: vec![HnfObject {
                id: "obj-1".to_string(),
                kind: "schematic.symbol".to_string(),
                properties: json!({"refdes": "R1"}),
            }],
        };

        let encoded = serde_json::to_string(&doc).expect("serialize hnf");
        let decoded: HnfDocument = serde_json::from_str(&encoded).expect("deserialize hnf");
        assert_eq!(decoded.objects.len(), 1);
    }

    #[test]
    fn host_env_flags_default_off() {
        std::env::remove_var("HBP_USE_HOST_OSS");
        std::env::remove_var("HBP_SIM_USE_HOST");
        assert!(!host_env::use_host_oss());
        assert!(!host_env::sim_use_host());
    }

    #[test]
    fn timeout_from_env_parses_override() {
        std::env::set_var("HBP_KICAD_TIMEOUT_SECS", "90");
        assert_eq!(
            host_env::timeout_from_env("HBP_KICAD_TIMEOUT_SECS", 120).as_secs(),
            90
        );
        std::env::remove_var("HBP_KICAD_TIMEOUT_SECS");
    }
}
