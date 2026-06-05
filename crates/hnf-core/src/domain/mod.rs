//! HNF domain payloads (Phase 0 v0.1).

use serde_json::Value;

mod bom;
mod firmware;
mod ic_layout;
mod layout;
mod mechanical;
mod schematic;
mod simulation;

pub use bom::{
    parse_bom, serialize_bom, validate_bom, BomDomain, BomLine, BomProperties, BOM_DOMAIN,
    BOM_VERSION,
};
pub use firmware::{
    parse_firmware, serialize_firmware, validate_firmware, FirmwareArtifact, FirmwareDomain,
    FirmwareProperties, FirmwareSource, FirmwareTarget, FIRMWARE_DOMAIN, FIRMWARE_VERSION,
};
pub use ic_layout::{
    parse_ic_layout, serialize_ic_layout, validate_ic_layout, IcLayer, IcLayoutDomain,
    IcLayoutProperties, IcShape, IC_LAYOUT_DOMAIN, IC_LAYOUT_VERSION,
};
pub use layout::{
    parse_layout, serialize_layout, validate_layout, LayoutDomain, LayoutFootprint,
    LayoutProperties, LayoutTrack, LAYOUT_DOMAIN, LAYOUT_VERSION,
};
pub use mechanical::{
    parse_mechanical, serialize_mechanical, validate_mechanical, MechanicalConstraint,
    MechanicalDomain, MechanicalProperties, MechanicalSolid, MECHANICAL_DOMAIN,
    MECHANICAL_VERSION,
};
pub use schematic::{
    parse_schematic, serialize_schematic, validate_schematic, SchematicDomain, SchematicNet,
    SchematicPin, SchematicPowerDomain, SchematicProperties, SchematicSymbol, SCHEMATIC_DOMAIN,
    SCHEMATIC_VERSION,
};
pub use simulation::{
    parse_simulation, serialize_simulation, validate_simulation, SimulationDomain,
    SimulationModel, SimulationProbe, SimulationProperties, SIMULATION_DOMAIN,
    SIMULATION_VERSION,
};

/// Phase 0 domains implemented in Rust (`hnf-core`).
pub const PHASE0_RUST_DOMAINS: &[&str] = &[
    SCHEMATIC_DOMAIN,
    LAYOUT_DOMAIN,
    IC_LAYOUT_DOMAIN,
    MECHANICAL_DOMAIN,
    SIMULATION_DOMAIN,
    BOM_DOMAIN,
    FIRMWARE_DOMAIN,
];

/// Parsed Phase 0 domain document (typed per domain).
#[derive(Debug, Clone, PartialEq)]
pub enum DomainDocument {
    Schematic(SchematicDomain),
    Layout(LayoutDomain),
    IcLayout(IcLayoutDomain),
    Mechanical(MechanicalDomain),
    Simulation(SimulationDomain),
    Bom(BomDomain),
    Firmware(FirmwareDomain),
}

/// Deserialize and validate any Phase 0 domain payload by `domain` field.
pub fn parse_domain(value: &Value) -> Result<DomainDocument, DomainParseError> {
    let domain_id = value
        .get("domain")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DomainParseError::Serde("missing domain field".into()))?;

    match domain_id {
        SCHEMATIC_DOMAIN => parse_schematic(value).map(DomainDocument::Schematic),
        LAYOUT_DOMAIN => parse_layout(value).map(DomainDocument::Layout),
        IC_LAYOUT_DOMAIN => parse_ic_layout(value).map(DomainDocument::IcLayout),
        MECHANICAL_DOMAIN => parse_mechanical(value).map(DomainDocument::Mechanical),
        SIMULATION_DOMAIN => parse_simulation(value).map(DomainDocument::Simulation),
        BOM_DOMAIN => parse_bom(value).map(DomainDocument::Bom),
        FIRMWARE_DOMAIN => parse_firmware(value).map(DomainDocument::Firmware),
        other => Err(DomainParseError::Serde(format!(
            "unsupported domain \"{other}\""
        ))),
    }
}

/// Shared domain envelope per `spec/spec-v0.1.md`.
pub const HNF_TYPE_OBJECT: &str = "hardware.object";
pub const DOMAIN_VERSION_V0_1: &str = "0.1";

/// Domain validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for DomainValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Parse/validate failure (serde or domain rules).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainParseError {
    Serde(String),
    Validation(Vec<DomainValidationError>),
}

impl std::fmt::Display for DomainParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serde(msg) => write!(f, "serde: {msg}"),
            Self::Validation(errs) => {
                let parts: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
                write!(f, "validation: {}", parts.join("; "))
            }
        }
    }
}

impl std::error::Error for DomainParseError {}

pub(crate) fn validate_envelope(
    domain: &str,
    expected_domain: &str,
    version: &str,
    expected_version: &str,
    hnf_type: &str,
    object_id: &str,
    content_hash: &Option<String>,
) -> Vec<DomainValidationError> {
    let mut errors = Vec::new();

    if domain != expected_domain {
        errors.push(DomainValidationError {
            field: "domain".into(),
            message: format!("expected \"{expected_domain}\", got \"{domain}\""),
        });
    }

    if version != expected_version {
        errors.push(DomainValidationError {
            field: "version".into(),
            message: format!("expected \"{expected_version}\", got \"{version}\""),
        });
    }

    if hnf_type.trim().is_empty() {
        errors.push(DomainValidationError {
            field: "hnf_type".into(),
            message: "required non-empty string".into(),
        });
    }

    if object_id.trim().is_empty() {
        errors.push(DomainValidationError {
            field: "object_id".into(),
            message: "required non-empty string".into(),
        });
    }

    if let Some(hash) = content_hash {
        let h = hash.trim();
        if h.is_empty() {
            errors.push(DomainValidationError {
                field: "content_hash".into(),
                message: "must be non-empty when present".into(),
            });
        } else if h.len() != 64 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
            errors.push(DomainValidationError {
                field: "content_hash".into(),
                message: "must be 64 hex characters (sha256)".into(),
            });
        }
    }

    errors
}

pub(crate) fn non_empty_ids<'a, I>(
    items: I,
    id_field: &'static str,
    get_id: fn(&I::Item) -> &str,
) -> Vec<DomainValidationError>
where
    I: IntoIterator,
    I::Item: 'a,
{
    let mut errors = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (i, item) in items.into_iter().enumerate() {
        let id = get_id(&item);
        if id.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("{id_field}[{i}]"),
                message: "id must be non-empty".into(),
            });
        } else if !seen.insert(id.to_string()) {
            errors.push(DomainValidationError {
                field: format!("{id_field}[{i}]"),
                message: format!("duplicate id \"{id}\""),
            });
        }
    }
    errors
}

pub(crate) fn finish_validation(errors: Vec<DomainValidationError>) -> Result<(), Vec<DomainValidationError>> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_envelope_rejects_bad_content_hash() {
        let errs = validate_envelope(
            SCHEMATIC_DOMAIN,
            SCHEMATIC_DOMAIN,
            SCHEMATIC_VERSION,
            SCHEMATIC_VERSION,
            HNF_TYPE_OBJECT,
            "obj-1",
            &Some("not-a-hash".into()),
        );
        assert!(errs.iter().any(|e| e.field == "content_hash"));
    }

    #[test]
    fn phase0_domain_constants_are_unique() {
        let domains = [
            SCHEMATIC_DOMAIN,
            LAYOUT_DOMAIN,
            IC_LAYOUT_DOMAIN,
            MECHANICAL_DOMAIN,
            SIMULATION_DOMAIN,
            BOM_DOMAIN,
            FIRMWARE_DOMAIN,
        ];
        let mut seen = std::collections::HashSet::new();
        for d in domains {
            assert!(seen.insert(d), "duplicate domain id {d}");
        }
    }
}
