//! HNF structural domain — members, loads, and materials (Phase 1).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    finish_validation, non_empty_ids, validate_envelope, DomainParseError, DomainValidationError,
    DOMAIN_VERSION_V0_1,
};

pub const STRUCTURAL_DOMAIN: &str = "structural";
pub const STRUCTURAL_VERSION: &str = DOMAIN_VERSION_V0_1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralDomain {
    pub domain: String,
    pub version: String,
    pub hnf_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    pub properties: StructuralProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralProperties {
    pub analysis_type: String,
    #[serde(default)]
    pub members: Vec<StructuralMember>,
    #[serde(default)]
    pub loads: Vec<StructuralLoad>,
    #[serde(default)]
    pub materials: Vec<StructuralMaterial>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralMember {
    pub id: String,
    pub section: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralLoad {
    pub id: String,
    #[serde(rename = "type")]
    pub load_type: String,
    pub magnitude: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralMaterial {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub youngs_modulus_gpa: Option<f64>,
}

pub fn parse_structural(value: &Value) -> Result<StructuralDomain, DomainParseError> {
    let domain: StructuralDomain = serde_json::from_value(value.clone())
        .map_err(|e| DomainParseError::Serde(e.to_string()))?;
    validate_structural(&domain).map_err(DomainParseError::Validation)?;
    Ok(domain)
}

pub fn validate_structural(domain: &StructuralDomain) -> Result<(), Vec<DomainValidationError>> {
    let mut errors = validate_envelope(
        &domain.domain,
        STRUCTURAL_DOMAIN,
        &domain.version,
        STRUCTURAL_VERSION,
        &domain.hnf_type,
        &domain.object_id,
        &domain.content_hash,
    );

    if domain.properties.analysis_type.trim().is_empty() {
        errors.push(DomainValidationError {
            field: "properties.analysis_type".into(),
            message: "required non-empty string".into(),
        });
    }

    if domain.properties.members.is_empty() {
        errors.push(DomainValidationError {
            field: "properties.members".into(),
            message: "required non-empty array".into(),
        });
    }

    errors.extend(non_empty_ids(
        domain.properties.members.iter(),
        "properties.members",
        |m| &m.id,
    ));
    errors.extend(non_empty_ids(
        domain.properties.loads.iter(),
        "properties.loads",
        |l| &l.id,
    ));
    errors.extend(non_empty_ids(
        domain.properties.materials.iter(),
        "properties.materials",
        |m| &m.id,
    ));

    let member_ids: std::collections::HashSet<_> = domain
        .properties
        .members
        .iter()
        .map(|m| m.id.as_str())
        .collect();
    let material_ids: std::collections::HashSet<_> = domain
        .properties
        .materials
        .iter()
        .map(|m| m.id.as_str())
        .collect();

    for (i, member) in domain.properties.members.iter().enumerate() {
        if member.section.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.members[{i}].section"),
                message: "required non-empty string".into(),
            });
        }
        if let Some(mid) = &member.material_id {
            if !material_ids.contains(mid.as_str()) {
                errors.push(DomainValidationError {
                    field: format!("properties.members[{i}].material_id"),
                    message: format!("unknown material_id \"{mid}\""),
                });
            }
        }
    }

    for (i, load) in domain.properties.loads.iter().enumerate() {
        if load.load_type.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.loads[{i}].type"),
                message: "required non-empty string".into(),
            });
        }
        if let Some(mid) = &load.member_id {
            if !member_ids.contains(mid.as_str()) {
                errors.push(DomainValidationError {
                    field: format!("properties.loads[{i}].member_id"),
                    message: format!("unknown member_id \"{mid}\""),
                });
            }
        }
    }

    finish_validation(errors)
}

pub fn serialize_structural(domain: &StructuralDomain) -> Value {
    serde_json::to_value(domain).expect("structural domain serializes")
}
