//! HNF IC layout domain — layers and shapes (GDS-style).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    finish_validation, non_empty_ids, validate_envelope, DomainParseError, DomainValidationError,
    DOMAIN_VERSION_V0_1,
};

pub const IC_LAYOUT_DOMAIN: &str = "ic_layout";
pub const IC_LAYOUT_VERSION: &str = DOMAIN_VERSION_V0_1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IcLayoutDomain {
    pub domain: String,
    pub version: String,
    pub hnf_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    pub properties: IcLayoutProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IcLayoutProperties {
    #[serde(default)]
    pub layers: Vec<IcLayer>,
    #[serde(default)]
    pub shapes: Vec<IcShape>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IcLayer {
    pub id: String,
    pub layer: u32,
    pub datatype: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IcShape {
    pub id: String,
    pub layer_id: String,
    /// Bounding box `[x1, y1, x2, y2]` in database units.
    pub bbox: [f64; 4],
}

pub fn parse_ic_layout(value: &Value) -> Result<IcLayoutDomain, DomainParseError> {
    let domain: IcLayoutDomain = serde_json::from_value(value.clone())
        .map_err(|e| DomainParseError::Serde(e.to_string()))?;
    validate_ic_layout(&domain).map_err(DomainParseError::Validation)?;
    Ok(domain)
}

pub fn validate_ic_layout(domain: &IcLayoutDomain) -> Result<(), Vec<DomainValidationError>> {
    let mut errors = validate_envelope(
        &domain.domain,
        IC_LAYOUT_DOMAIN,
        &domain.version,
        IC_LAYOUT_VERSION,
        &domain.hnf_type,
        &domain.object_id,
        &domain.content_hash,
    );

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

    errors.extend(non_empty_ids(
        domain.properties.shapes.iter(),
        "properties.shapes",
        |s| &s.id,
    ));

    let layer_ids: std::collections::HashSet<_> =
        domain.properties.layers.iter().map(|l| l.id.as_str()).collect();

    for (i, shape) in domain.properties.shapes.iter().enumerate() {
        if !layer_ids.contains(shape.layer_id.as_str()) {
            errors.push(DomainValidationError {
                field: format!("properties.shapes[{i}].layer_id"),
                message: format!("unknown layer_id \"{}\"", shape.layer_id),
            });
        }
        if shape.bbox[2] < shape.bbox[0] || shape.bbox[3] < shape.bbox[1] {
            errors.push(DomainValidationError {
                field: format!("properties.shapes[{i}].bbox"),
                message: "must satisfy x2 >= x1 and y2 >= y1".into(),
            });
        }
    }

    finish_validation(errors)
}

pub fn serialize_ic_layout(domain: &IcLayoutDomain) -> Value {
    serde_json::to_value(domain).expect("ic_layout domain serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::HNF_TYPE_OBJECT;

    fn minimal_ic_layout() -> IcLayoutDomain {
        IcLayoutDomain {
            domain: IC_LAYOUT_DOMAIN.into(),
            version: IC_LAYOUT_VERSION.into(),
            hnf_type: HNF_TYPE_OBJECT.into(),
            object_id: "550e8400-e29b-41d4-a716-446655440011".into(),
            content_hash: None,
            refs: vec![],
            properties: IcLayoutProperties {
                layers: vec![IcLayer {
                    id: "ly-m1".into(),
                    layer: 1,
                    datatype: 0,
                }],
                shapes: vec![IcShape {
                    id: "sh-inv".into(),
                    layer_id: "ly-m1".into(),
                    bbox: [0.0, 0.0, 100.0, 100.0],
                }],
            },
        }
    }

    #[test]
    fn validate_accepts_minimal_ic_layout() {
        assert!(validate_ic_layout(&minimal_ic_layout()).is_ok());
    }

    #[test]
    fn validate_rejects_empty_layers() {
        let mut doc = minimal_ic_layout();
        doc.properties.layers.clear();
        let errs = validate_ic_layout(&doc).expect_err("no layers");
        assert!(errs.iter().any(|e| e.field == "properties.layers"));
    }

    #[test]
    fn parse_rejects_dangling_layer_reference() {
        let mut doc = minimal_ic_layout();
        doc.properties.shapes[0].layer_id = "ly-missing".into();
        let err = parse_ic_layout(&serialize_ic_layout(&doc)).expect_err("dangling layer");
        if let DomainParseError::Validation(errs) = err {
            assert!(errs.iter().any(|e| e.field.contains("layer_id")));
        } else {
            panic!("expected validation error");
        }
    }

    #[test]
    fn validate_rejects_inverted_bbox() {
        let mut doc = minimal_ic_layout();
        doc.properties.shapes[0].bbox = [10.0, 10.0, 0.0, 0.0];
        let errs = validate_ic_layout(&doc).expect_err("bad bbox");
        assert!(errs.iter().any(|e| e.field.contains("bbox")));
    }

    #[test]
    fn roundtrip_serialize_parse() {
        let doc = minimal_ic_layout();
        let parsed = parse_ic_layout(&serialize_ic_layout(&doc)).expect("roundtrip");
        assert_eq!(parsed, doc);
    }
}
