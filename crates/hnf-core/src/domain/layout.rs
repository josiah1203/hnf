//! HNF layout domain — PCB footprints and tracks.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    finish_validation, non_empty_ids, validate_envelope, DomainParseError, DomainValidationError,
    DOMAIN_VERSION_V0_1,
};

pub const LAYOUT_DOMAIN: &str = "layout";
pub const LAYOUT_VERSION: &str = DOMAIN_VERSION_V0_1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutDomain {
    pub domain: String,
    pub version: String,
    pub hnf_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    pub properties: LayoutProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LayoutProperties {
    #[serde(default)]
    pub footprints: Vec<LayoutFootprint>,
    #[serde(default)]
    pub tracks: Vec<LayoutTrack>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutFootprint {
    pub id: String,
    pub refdes: String,
    pub layer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_x: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_y: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_deg: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutTrack {
    pub id: String,
    pub net: String,
    pub layer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width_mm: Option<f64>,
}

pub fn parse_layout(value: &Value) -> Result<LayoutDomain, DomainParseError> {
    let domain: LayoutDomain = serde_json::from_value(value.clone())
        .map_err(|e| DomainParseError::Serde(e.to_string()))?;
    validate_layout(&domain).map_err(DomainParseError::Validation)?;
    Ok(domain)
}

pub fn validate_layout(domain: &LayoutDomain) -> Result<(), Vec<DomainValidationError>> {
    let mut errors = validate_envelope(
        &domain.domain,
        LAYOUT_DOMAIN,
        &domain.version,
        LAYOUT_VERSION,
        &domain.hnf_type,
        &domain.object_id,
        &domain.content_hash,
    );

    if domain.properties.footprints.is_empty() && domain.properties.tracks.is_empty() {
        errors.push(DomainValidationError {
            field: "properties".into(),
            message: "requires at least one footprint or track".into(),
        });
    }

    errors.extend(non_empty_ids(
        domain.properties.footprints.iter(),
        "properties.footprints",
        |f| &f.id,
    ));

    let mut refdes_seen = std::collections::HashSet::new();
    for (i, fp) in domain.properties.footprints.iter().enumerate() {
        if fp.refdes.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.footprints[{i}].refdes"),
                message: "required non-empty string".into(),
            });
        } else if !refdes_seen.insert(fp.refdes.clone()) {
            errors.push(DomainValidationError {
                field: format!("properties.footprints[{i}].refdes"),
                message: format!("duplicate refdes \"{}\"", fp.refdes),
            });
        }
        if fp.layer.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.footprints[{i}].layer"),
                message: "required non-empty string".into(),
            });
        }
    }

    errors.extend(non_empty_ids(
        domain.properties.tracks.iter(),
        "properties.tracks",
        |t| &t.id,
    ));

    for (i, track) in domain.properties.tracks.iter().enumerate() {
        if track.net.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.tracks[{i}].net"),
                message: "required non-empty string".into(),
            });
        }
        if track.layer.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.tracks[{i}].layer"),
                message: "required non-empty string".into(),
            });
        }
    }

    finish_validation(errors)
}

pub fn serialize_layout(domain: &LayoutDomain) -> Value {
    serde_json::to_value(domain).expect("layout domain serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::HNF_TYPE_OBJECT;

    fn minimal_layout() -> LayoutDomain {
        LayoutDomain {
            domain: LAYOUT_DOMAIN.into(),
            version: LAYOUT_VERSION.into(),
            hnf_type: HNF_TYPE_OBJECT.into(),
            object_id: "550e8400-e29b-41d4-a716-446655440010".into(),
            content_hash: None,
            refs: vec!["550e8400-e29b-41d4-a716-446655440001".into()],
            properties: LayoutProperties {
                footprints: vec![LayoutFootprint {
                    id: "fp-u1".into(),
                    refdes: "U1".into(),
                    layer: "F.Cu".into(),
                    position_x: Some(10.0),
                    position_y: Some(20.0),
                    rotation_deg: Some(90.0),
                }],
                tracks: vec![LayoutTrack {
                    id: "tr-gnd".into(),
                    net: "GND".into(),
                    layer: "B.Cu".into(),
                    width_mm: Some(0.2),
                }],
            },
        }
    }

    #[test]
    fn validate_accepts_minimal_layout() {
        assert!(validate_layout(&minimal_layout()).is_ok());
    }

    #[test]
    fn validate_rejects_empty_geometry() {
        let doc = LayoutDomain {
            properties: LayoutProperties::default(),
            ..minimal_layout()
        };
        let errs = validate_layout(&doc).expect_err("empty geometry");
        assert!(errs.iter().any(|e| e.field == "properties"));
    }

    #[test]
    fn parse_rejects_wrong_domain() {
        let mut doc = minimal_layout();
        doc.domain = "bom".into();
        let err = parse_layout(&serialize_layout(&doc)).expect_err("wrong domain");
        assert!(matches!(err, DomainParseError::Validation(_)));
    }

    #[test]
    fn tracks_only_layout_valid() {
        let doc = LayoutDomain {
            properties: LayoutProperties {
                footprints: vec![],
                tracks: vec![LayoutTrack {
                    id: "tr-vcc".into(),
                    net: "VCC".into(),
                    layer: "F.Cu".into(),
                    width_mm: None,
                }],
            },
            ..minimal_layout()
        };
        assert!(validate_layout(&doc).is_ok());
    }

    #[test]
    fn roundtrip_serialize_parse() {
        let doc = minimal_layout();
        let parsed = parse_layout(&serialize_layout(&doc)).expect("roundtrip");
        assert_eq!(parsed, doc);
    }
}
