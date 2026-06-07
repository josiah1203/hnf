//! HNF core — document model and ToolAdapter traits (HummingBird v8).

pub mod diff;
pub mod domain;

pub use diff::{
    diff_bom, diff_domain, diff_path_fallback, diff_schematic, diff_values, uses_object_graph_diff,
    DiffChange, DiffChangeType,
};
pub use domain::{
    parse_bim, parse_bom, parse_domain, parse_energy_building, parse_firmware, parse_geospatial,
    parse_ic_layout, parse_layout, parse_mechanical, parse_schematic, parse_simulation,
    parse_structural, serialize_bim, serialize_bom, serialize_energy_building, serialize_firmware,
    serialize_geospatial, serialize_ic_layout, serialize_layout, serialize_mechanical,
    serialize_schematic, serialize_simulation, serialize_structural, validate_bim, validate_bom,
    validate_energy_building, validate_firmware, validate_geospatial, validate_ic_layout,
    validate_layout, validate_mechanical, validate_schematic, validate_simulation,
    validate_structural, ALL_RUST_DOMAINS, BimDomain, BimElement, BimProperties, BimStorey,
    BomDomain, BomLine, BomProperties, DomainDocument, DomainParseError, DomainValidationError,
    EnergyBuildingDomain, EnergyBuildingProperties, EnergySystem, EnergyZone, FirmwareArtifact,
    FirmwareDomain, FirmwareProperties, FirmwareSource, FirmwareTarget, GeospatialDomain,
    GeospatialLayer, GeospatialProperties, IcLayer, IcLayoutDomain, IcLayoutProperties, IcShape,
    LayoutDomain, LayoutFootprint, LayoutProperties, LayoutTrack, MechanicalBoundaryCondition,
    MechanicalConstraint, MechanicalDomain, MechanicalGeometryBlob, MechanicalMaterialSpec,
    MechanicalProperties, MechanicalSolid, MechanicalTolerance, SchematicDomain, SchematicNet,
    SchematicPin, SchematicPowerDomain, SchematicProperties, SchematicSymbol, SimulationDomain,
    SimulationModel, SimulationProbe, SimulationProperties, StructuralDomain, StructuralLoad,
    StructuralMaterial, StructuralMember, StructuralProperties, BIM_DOMAIN, BIM_VERSION,
    BOM_DOMAIN, BOM_VERSION, ENERGY_BUILDING_DOMAIN, ENERGY_BUILDING_VERSION, FIRMWARE_DOMAIN,
    FIRMWARE_VERSION, GEOSPATIAL_DOMAIN, GEOSPATIAL_VERSION, IC_LAYOUT_DOMAIN, IC_LAYOUT_VERSION,
    LAYOUT_DOMAIN, LAYOUT_VERSION, MECHANICAL_DOMAIN, MECHANICAL_VERSION, PHASE0_RUST_DOMAINS,
    PHASE1_RUST_DOMAINS, SCHEMATIC_DOMAIN, SCHEMATIC_VERSION, SIMULATION_DOMAIN, SIMULATION_VERSION,
    STRUCTURAL_DOMAIN, STRUCTURAL_VERSION,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sidecar_protocol::{SceneGraphEdge, SceneGraphNode};

/// Supported `hnf_version` values for spec v0.1.
pub const HNF_VERSION_V0_1: &str = "0.1";

/// Phase 0 domain IDs (see `spec/spec-v0.1.md`).
pub const PHASE0_DOMAINS: &[&str] = PHASE0_RUST_DOMAINS;

/// Top-level manifest (v8 core layer). Required on every HNF package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HnfManifest {
    pub hnf_version: String,
    pub doc_id: String,
    pub disciplines: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_revision: Option<String>,
}

/// Single manifest validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestValidationError {
    pub field: &'static str,
    pub message: String,
}

impl std::fmt::Display for ManifestValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Validate required manifest fields per HNF spec v0.1.
pub fn validate(manifest: &HnfManifest) -> Result<(), Vec<ManifestValidationError>> {
    let mut errors = Vec::new();

    if manifest.hnf_version.trim().is_empty() {
        errors.push(ManifestValidationError {
            field: "hnf_version",
            message: "required non-empty string".into(),
        });
    } else if manifest.hnf_version != HNF_VERSION_V0_1 {
        errors.push(ManifestValidationError {
            field: "hnf_version",
            message: format!("unsupported; expected \"{HNF_VERSION_V0_1}\""),
        });
    }

    if manifest.doc_id.trim().is_empty() {
        errors.push(ManifestValidationError {
            field: "doc_id",
            message: "required non-empty string".into(),
        });
    }

    if manifest.disciplines.is_empty() {
        errors.push(ManifestValidationError {
            field: "disciplines",
            message: "required non-empty array".into(),
        });
    } else {
        for (i, d) in manifest.disciplines.iter().enumerate() {
            if d.trim().is_empty() {
                errors.push(ManifestValidationError {
                    field: "disciplines",
                    message: format!("entry {i} must be non-empty"),
                });
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HnfDocument {
    pub manifest: HnfManifest,
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

    fn sample_manifest() -> HnfManifest {
        HnfManifest {
            hnf_version: HNF_VERSION_V0_1.to_string(),
            doc_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            disciplines: vec!["schematic".to_string(), "layout".to_string()],
            created_at: None,
            schema_revision: None,
        }
    }

    #[test]
    fn validate_accepts_required_manifest_fields() {
        assert!(validate(&sample_manifest()).is_ok());
    }

    #[test]
    fn validate_rejects_missing_or_invalid_manifest_fields() {
        let cases = [
            (
                HnfManifest {
                    hnf_version: "".into(),
                    ..sample_manifest()
                },
                "hnf_version",
            ),
            (
                HnfManifest {
                    hnf_version: "9.9".into(),
                    ..sample_manifest()
                },
                "hnf_version",
            ),
            (
                HnfManifest {
                    doc_id: "  ".into(),
                    ..sample_manifest()
                },
                "doc_id",
            ),
            (
                HnfManifest {
                    disciplines: vec![],
                    ..sample_manifest()
                },
                "disciplines",
            ),
            (
                HnfManifest {
                    disciplines: vec!["schematic".into(), "".into()],
                    ..sample_manifest()
                },
                "disciplines",
            ),
        ];

        for (manifest, expected_field) in cases {
            let err = validate(&manifest).expect_err("manifest should fail validation");
            assert!(
                err.iter().any(|e| e.field == expected_field),
                "expected field {expected_field}, got {err:?}"
            );
        }
    }

    #[test]
    fn minimum_document_model_serializes() {
        let doc = HnfDocument {
            manifest: sample_manifest(),
            document_uri: "hbp://docs/board.kicad".to_string(),
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
        assert!(validate(&decoded.manifest).is_ok());
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

    #[test]
    fn phase0_domains_match_rust_implementations() {
        assert_eq!(PHASE0_DOMAINS.len(), PHASE0_RUST_DOMAINS.len());
        for d in PHASE0_DOMAINS {
            assert!(PHASE0_RUST_DOMAINS.contains(d));
        }
    }
}
