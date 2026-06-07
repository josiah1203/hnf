//! HNF geospatial domain — CRS, layers, and features (Phase 1).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    finish_validation, non_empty_ids, validate_envelope, DomainParseError, DomainValidationError,
    DOMAIN_VERSION_V0_1,
};

pub const GEOSPATIAL_DOMAIN: &str = "geospatial";
pub const GEOSPATIAL_VERSION: &str = DOMAIN_VERSION_V0_1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeospatialDomain {
    pub domain: String,
    pub version: String,
    pub hnf_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    pub properties: GeospatialProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeospatialProperties {
    pub crs: String,
    #[serde(default)]
    pub layers: Vec<GeospatialLayer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeospatialLayer {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub feature_count: u32,
}

pub fn parse_geospatial(value: &Value) -> Result<GeospatialDomain, DomainParseError> {
    let domain: GeospatialDomain = serde_json::from_value(value.clone())
        .map_err(|e| DomainParseError::Serde(e.to_string()))?;
    validate_geospatial(&domain).map_err(DomainParseError::Validation)?;
    Ok(domain)
}

pub fn validate_geospatial(domain: &GeospatialDomain) -> Result<(), Vec<DomainValidationError>> {
    let mut errors = validate_envelope(
        &domain.domain,
        GEOSPATIAL_DOMAIN,
        &domain.version,
        GEOSPATIAL_VERSION,
        &domain.hnf_type,
        &domain.object_id,
        &domain.content_hash,
    );

    if domain.properties.crs.trim().is_empty() {
        errors.push(DomainValidationError {
            field: "properties.crs".into(),
            message: "required non-empty string (e.g. EPSG:4326)".into(),
        });
    }

    if domain.properties.layers.is_empty() {
        errors.push(DomainValidationError {
            field: "properties.layers".into(),
            message: "required non-empty array".into(),
        });
    }

    errors.extend(non_empty_ids(
        domain.properties.layers.iter(),
        "properties.layers",
        |l| &l.id,
    ));

    for (i, layer) in domain.properties.layers.iter().enumerate() {
        if layer.name.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.layers[{i}].name"),
                message: "required non-empty string".into(),
            });
        }
    }

    finish_validation(errors)
}

pub fn serialize_geospatial(domain: &GeospatialDomain) -> Value {
    serde_json::to_value(domain).expect("geospatial domain serializes")
}
