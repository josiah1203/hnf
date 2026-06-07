//! HNF energy_building domain — zones, systems, and schedules (Phase 1).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    finish_validation, non_empty_ids, validate_envelope, DomainParseError, DomainValidationError,
    DOMAIN_VERSION_V0_1,
};

pub const ENERGY_BUILDING_DOMAIN: &str = "energy_building";
pub const ENERGY_BUILDING_VERSION: &str = DOMAIN_VERSION_V0_1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergyBuildingDomain {
    pub domain: String,
    pub version: String,
    pub hnf_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    pub properties: EnergyBuildingProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergyBuildingProperties {
    pub model_name: String,
    #[serde(default)]
    pub zones: Vec<EnergyZone>,
    #[serde(default)]
    pub systems: Vec<EnergySystem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather_file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergyZone {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor_area_m2: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergySystem {
    pub id: String,
    pub system_type: String,
    #[serde(default)]
    pub serves_zone_ids: Vec<String>,
}

pub fn parse_energy_building(value: &Value) -> Result<EnergyBuildingDomain, DomainParseError> {
    let domain: EnergyBuildingDomain = serde_json::from_value(value.clone())
        .map_err(|e| DomainParseError::Serde(e.to_string()))?;
    validate_energy_building(&domain).map_err(DomainParseError::Validation)?;
    Ok(domain)
}

pub fn validate_energy_building(
    domain: &EnergyBuildingDomain,
) -> Result<(), Vec<DomainValidationError>> {
    let mut errors = validate_envelope(
        &domain.domain,
        ENERGY_BUILDING_DOMAIN,
        &domain.version,
        ENERGY_BUILDING_VERSION,
        &domain.hnf_type,
        &domain.object_id,
        &domain.content_hash,
    );

    if domain.properties.model_name.trim().is_empty() {
        errors.push(DomainValidationError {
            field: "properties.model_name".into(),
            message: "required non-empty string".into(),
        });
    }

    if domain.properties.zones.is_empty() {
        errors.push(DomainValidationError {
            field: "properties.zones".into(),
            message: "required non-empty array".into(),
        });
    }

    errors.extend(non_empty_ids(
        domain.properties.zones.iter(),
        "properties.zones",
        |z| &z.id,
    ));
    errors.extend(non_empty_ids(
        domain.properties.systems.iter(),
        "properties.systems",
        |s| &s.id,
    ));

    let zone_ids: std::collections::HashSet<_> = domain
        .properties
        .zones
        .iter()
        .map(|z| z.id.as_str())
        .collect();

    for (i, zone) in domain.properties.zones.iter().enumerate() {
        if zone.name.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.zones[{i}].name"),
                message: "required non-empty string".into(),
            });
        }
    }

    for (i, system) in domain.properties.systems.iter().enumerate() {
        if system.system_type.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.systems[{i}].system_type"),
                message: "required non-empty string".into(),
            });
        }
        for (j, zid) in system.serves_zone_ids.iter().enumerate() {
            if !zone_ids.contains(zid.as_str()) {
                errors.push(DomainValidationError {
                    field: format!("properties.systems[{i}].serves_zone_ids[{j}]"),
                    message: format!("unknown zone id \"{zid}\""),
                });
            }
        }
    }

    finish_validation(errors)
}

pub fn serialize_energy_building(domain: &EnergyBuildingDomain) -> Value {
    serde_json::to_value(domain).expect("energy_building domain serializes")
}
