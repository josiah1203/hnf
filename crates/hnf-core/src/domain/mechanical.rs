//! HNF mechanical domain — MCAD solids and constraints.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    finish_validation, non_empty_ids, validate_envelope, DomainParseError, DomainValidationError,
    DOMAIN_VERSION_V0_1,
};

pub const MECHANICAL_DOMAIN: &str = "mechanical";
pub const MECHANICAL_VERSION: &str = DOMAIN_VERSION_V0_1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanicalDomain {
    pub domain: String,
    pub version: String,
    pub hnf_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    pub properties: MechanicalProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MechanicalProperties {
    #[serde(default)]
    pub solids: Vec<MechanicalSolid>,
    #[serde(default)]
    pub constraints: Vec<MechanicalConstraint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanicalSolid {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_mm3: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanicalConstraint {
    pub id: String,
    #[serde(rename = "type")]
    pub constraint_type: String,
    pub solid_a: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solid_b: Option<String>,
}

pub fn parse_mechanical(value: &Value) -> Result<MechanicalDomain, DomainParseError> {
    let domain: MechanicalDomain = serde_json::from_value(value.clone())
        .map_err(|e| DomainParseError::Serde(e.to_string()))?;
    validate_mechanical(&domain).map_err(DomainParseError::Validation)?;
    Ok(domain)
}

pub fn validate_mechanical(domain: &MechanicalDomain) -> Result<(), Vec<DomainValidationError>> {
    let mut errors = validate_envelope(
        &domain.domain,
        MECHANICAL_DOMAIN,
        &domain.version,
        MECHANICAL_VERSION,
        &domain.hnf_type,
        &domain.object_id,
        &domain.content_hash,
    );

    if domain.properties.solids.is_empty() {
        errors.push(DomainValidationError {
            field: "properties.solids".into(),
            message: "required non-empty array".into(),
        });
    }

    errors.extend(non_empty_ids(
        domain.properties.solids.iter(),
        "properties.solids",
        |s| &s.id,
    ));

    for (i, solid) in domain.properties.solids.iter().enumerate() {
        if solid.name.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.solids[{i}].name"),
                message: "required non-empty string".into(),
            });
        }
    }

    errors.extend(non_empty_ids(
        domain.properties.constraints.iter(),
        "properties.constraints",
        |c| &c.id,
    ));

    let solid_ids: std::collections::HashSet<_> =
        domain.properties.solids.iter().map(|s| s.id.as_str()).collect();

    for (i, c) in domain.properties.constraints.iter().enumerate() {
        if c.constraint_type.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.constraints[{i}].type"),
                message: "required non-empty string".into(),
            });
        }
        if !solid_ids.contains(c.solid_a.as_str()) {
            errors.push(DomainValidationError {
                field: format!("properties.constraints[{i}].solid_a"),
                message: format!("unknown solid_a \"{}\"", c.solid_a),
            });
        }
        if let Some(sb) = &c.solid_b {
            if !solid_ids.contains(sb.as_str()) {
                errors.push(DomainValidationError {
                    field: format!("properties.constraints[{i}].solid_b"),
                    message: format!("unknown solid_b \"{sb}\""),
                });
            }
        }
    }

    finish_validation(errors)
}

pub fn serialize_mechanical(domain: &MechanicalDomain) -> Value {
    serde_json::to_value(domain).expect("mechanical domain serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::HNF_TYPE_OBJECT;

    fn minimal_mechanical() -> MechanicalDomain {
        MechanicalDomain {
            domain: MECHANICAL_DOMAIN.into(),
            version: MECHANICAL_VERSION.into(),
            hnf_type: HNF_TYPE_OBJECT.into(),
            object_id: "550e8400-e29b-41d4-a716-446655440012".into(),
            content_hash: None,
            refs: vec![],
            properties: MechanicalProperties {
                solids: vec![MechanicalSolid {
                    id: "solid-bracket-1".into(),
                    name: "Mounting Bracket".into(),
                    material: Some("Aluminum".into()),
                    volume_mm3: Some(128.5),
                }],
                constraints: vec![MechanicalConstraint {
                    id: "c-fix-1".into(),
                    constraint_type: "fixed".into(),
                    solid_a: "solid-bracket-1".into(),
                    solid_b: None,
                }],
            },
        }
    }

    #[test]
    fn validate_accepts_minimal_mechanical() {
        assert!(validate_mechanical(&minimal_mechanical()).is_ok());
    }

    #[test]
    fn validate_rejects_empty_solids() {
        let mut doc = minimal_mechanical();
        doc.properties.solids.clear();
        let errs = validate_mechanical(&doc).expect_err("no solids");
        assert!(errs.iter().any(|e| e.field == "properties.solids"));
    }

    #[test]
    fn parse_rejects_unknown_constraint_solid() {
        let mut doc = minimal_mechanical();
        doc.properties.constraints[0].solid_a = "solid-missing".into();
        let err = parse_mechanical(&serialize_mechanical(&doc)).expect_err("bad solid ref");
        if let DomainParseError::Validation(errs) = err {
            assert!(errs.iter().any(|e| e.field.contains("solid_a")));
        } else {
            panic!("expected validation error");
        }
    }

    #[test]
    fn roundtrip_serialize_parse() {
        let doc = minimal_mechanical();
        let parsed = parse_mechanical(&serialize_mechanical(&doc)).expect("roundtrip");
        assert_eq!(parsed, doc);
    }
}
