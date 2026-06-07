//! HNF BIM domain — spatial structure and building elements (Phase 1).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    finish_validation, non_empty_ids, validate_envelope, DomainParseError, DomainValidationError,
    DOMAIN_VERSION_V0_1,
};

pub const BIM_DOMAIN: &str = "bim";
pub const BIM_VERSION: &str = DOMAIN_VERSION_V0_1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BimDomain {
    pub domain: String,
    pub version: String,
    pub hnf_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    pub properties: BimProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BimProperties {
    pub project_name: String,
    #[serde(default)]
    pub storeys: Vec<BimStorey>,
    #[serde(default)]
    pub elements: Vec<BimElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BimStorey {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevation_m: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BimElement {
    pub id: String,
    pub ifc_class: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storey_id: Option<String>,
}

pub fn parse_bim(value: &Value) -> Result<BimDomain, DomainParseError> {
    let domain: BimDomain = serde_json::from_value(value.clone())
        .map_err(|e| DomainParseError::Serde(e.to_string()))?;
    validate_bim(&domain).map_err(DomainParseError::Validation)?;
    Ok(domain)
}

pub fn validate_bim(domain: &BimDomain) -> Result<(), Vec<DomainValidationError>> {
    let mut errors = validate_envelope(
        &domain.domain,
        BIM_DOMAIN,
        &domain.version,
        BIM_VERSION,
        &domain.hnf_type,
        &domain.object_id,
        &domain.content_hash,
    );

    if domain.properties.project_name.trim().is_empty() {
        errors.push(DomainValidationError {
            field: "properties.project_name".into(),
            message: "required non-empty string".into(),
        });
    }

    errors.extend(non_empty_ids(
        domain.properties.storeys.iter(),
        "properties.storeys",
        |s| &s.id,
    ));
    errors.extend(non_empty_ids(
        domain.properties.elements.iter(),
        "properties.elements",
        |e| &e.id,
    ));

    let storey_ids: std::collections::HashSet<_> = domain
        .properties
        .storeys
        .iter()
        .map(|s| s.id.as_str())
        .collect();

    for (i, el) in domain.properties.elements.iter().enumerate() {
        if el.ifc_class.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.elements[{i}].ifc_class"),
                message: "required non-empty string".into(),
            });
        }
        if let Some(sid) = &el.storey_id {
            if !storey_ids.contains(sid.as_str()) {
                errors.push(DomainValidationError {
                    field: format!("properties.elements[{i}].storey_id"),
                    message: format!("unknown storey_id \"{sid}\""),
                });
            }
        }
    }

    finish_validation(errors)
}

pub fn serialize_bim(domain: &BimDomain) -> Value {
    serde_json::to_value(domain).expect("bim domain serializes")
}
