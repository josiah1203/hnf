//! HNF simulation domain — solver models and probes.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    finish_validation, non_empty_ids, validate_envelope, DomainParseError, DomainValidationError,
    DOMAIN_VERSION_V0_1,
};

pub const SIMULATION_DOMAIN: &str = "simulation";
pub const SIMULATION_VERSION: &str = DOMAIN_VERSION_V0_1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationDomain {
    pub domain: String,
    pub version: String,
    pub hnf_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    pub properties: SimulationProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SimulationProperties {
    #[serde(default)]
    pub models: Vec<SimulationModel>,
    #[serde(default)]
    pub probes: Vec<SimulationProbe>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationModel {
    pub id: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solver: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netlist_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationProbe {
    pub id: String,
    pub model_id: String,
    pub signal: String,
}

pub fn parse_simulation(value: &Value) -> Result<SimulationDomain, DomainParseError> {
    let domain: SimulationDomain = serde_json::from_value(value.clone())
        .map_err(|e| DomainParseError::Serde(e.to_string()))?;
    validate_simulation(&domain).map_err(DomainParseError::Validation)?;
    Ok(domain)
}

pub fn validate_simulation(domain: &SimulationDomain) -> Result<(), Vec<DomainValidationError>> {
    let mut errors = validate_envelope(
        &domain.domain,
        SIMULATION_DOMAIN,
        &domain.version,
        SIMULATION_VERSION,
        &domain.hnf_type,
        &domain.object_id,
        &domain.content_hash,
    );

    if domain.properties.models.is_empty() {
        errors.push(DomainValidationError {
            field: "properties.models".into(),
            message: "required non-empty array".into(),
        });
    }

    errors.extend(non_empty_ids(
        domain.properties.models.iter(),
        "properties.models",
        |m| &m.id,
    ));

    let allowed_kinds = ["spice", "em", "fea", "mixed"];
    for (i, model) in domain.properties.models.iter().enumerate() {
        if model.kind.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.models[{i}].kind"),
                message: "required non-empty string".into(),
            });
        } else if !allowed_kinds.contains(&model.kind.as_str()) {
            errors.push(DomainValidationError {
                field: format!("properties.models[{i}].kind"),
                message: format!(
                    "unsupported kind \"{}\"; expected one of {}",
                    model.kind,
                    allowed_kinds.join(", ")
                ),
            });
        }
        if let Some(hash) = &model.netlist_hash {
            if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                errors.push(DomainValidationError {
                    field: format!("properties.models[{i}].netlist_hash"),
                    message: "must be 64 hex characters (sha256)".into(),
                });
            }
        }
    }

    errors.extend(non_empty_ids(
        domain.properties.probes.iter(),
        "properties.probes",
        |p| &p.id,
    ));

    let model_ids: std::collections::HashSet<_> =
        domain.properties.models.iter().map(|m| m.id.as_str()).collect();

    for (i, probe) in domain.properties.probes.iter().enumerate() {
        if probe.signal.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.probes[{i}].signal"),
                message: "required non-empty string".into(),
            });
        }
        if !model_ids.contains(probe.model_id.as_str()) {
            errors.push(DomainValidationError {
                field: format!("properties.probes[{i}].model_id"),
                message: format!("unknown model_id \"{}\"", probe.model_id),
            });
        }
    }

    finish_validation(errors)
}

pub fn serialize_simulation(domain: &SimulationDomain) -> Value {
    serde_json::to_value(domain).expect("simulation domain serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::HNF_TYPE_OBJECT;

    fn minimal_simulation() -> SimulationDomain {
        SimulationDomain {
            domain: SIMULATION_DOMAIN.into(),
            version: SIMULATION_VERSION.into(),
            hnf_type: HNF_TYPE_OBJECT.into(),
            object_id: "550e8400-e29b-41d4-a716-446655440013".into(),
            content_hash: None,
            refs: vec!["550e8400-e29b-41d4-a716-446655440001".into()],
            properties: SimulationProperties {
                models: vec![SimulationModel {
                    id: "mdl-spice-1".into(),
                    kind: "spice".into(),
                    solver: Some("ngspice".into()),
                    netlist_hash: None,
                }],
                probes: vec![SimulationProbe {
                    id: "prb-vout".into(),
                    model_id: "mdl-spice-1".into(),
                    signal: "V(out)".into(),
                }],
            },
        }
    }

    #[test]
    fn validate_accepts_minimal_simulation() {
        assert!(validate_simulation(&minimal_simulation()).is_ok());
    }

    #[test]
    fn validate_rejects_empty_models() {
        let mut doc = minimal_simulation();
        doc.properties.models.clear();
        let errs = validate_simulation(&doc).expect_err("no models");
        assert!(errs.iter().any(|e| e.field == "properties.models"));
    }

    #[test]
    fn parse_rejects_unknown_probe_model() {
        let mut doc = minimal_simulation();
        doc.properties.probes[0].model_id = "mdl-missing".into();
        let err = parse_simulation(&serialize_simulation(&doc)).expect_err("dangling probe");
        if let DomainParseError::Validation(errs) = err {
            assert!(errs.iter().any(|e| e.field.contains("model_id")));
        } else {
            panic!("expected validation error");
        }
    }

    #[test]
    fn validate_rejects_invalid_kind() {
        let mut doc = minimal_simulation();
        doc.properties.models[0].kind = "cfd".into();
        let errs = validate_simulation(&doc).expect_err("bad kind");
        assert!(errs.iter().any(|e| e.field.contains("kind")));
    }

    #[test]
    fn roundtrip_serialize_parse() {
        let doc = minimal_simulation();
        let parsed = parse_simulation(&serialize_simulation(&doc)).expect("roundtrip");
        assert_eq!(parsed, doc);
    }
}
