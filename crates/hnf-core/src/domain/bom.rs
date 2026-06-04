//! HNF BOM domain — component records, supplier refs, lifecycle.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    finish_validation, non_empty_ids, validate_envelope, DomainParseError, DomainValidationError,
    DOMAIN_VERSION_V0_1,
};

pub const BOM_DOMAIN: &str = "bom";
pub const BOM_VERSION: &str = DOMAIN_VERSION_V0_1;

/// Full BOM domain document (envelope + typed properties).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BomDomain {
    pub domain: String,
    pub version: String,
    pub hnf_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    pub properties: BomProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BomProperties {
    pub lines: Vec<BomLine>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BomLine {
    pub line_id: String,
    pub quantity: u32,
    #[serde(default)]
    pub refdes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mpn: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supplier_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Deserialize JSON and validate BOM domain rules.
pub fn parse_bom(value: &Value) -> Result<BomDomain, DomainParseError> {
    let domain: BomDomain = serde_json::from_value(value.clone())
        .map_err(|e| DomainParseError::Serde(e.to_string()))?;
    validate_bom(&domain).map_err(DomainParseError::Validation)?;
    Ok(domain)
}

/// Validate BOM domain envelope and line items.
pub fn validate_bom(domain: &BomDomain) -> Result<(), Vec<DomainValidationError>> {
    let mut errors = validate_envelope(
        &domain.domain,
        BOM_DOMAIN,
        &domain.version,
        BOM_VERSION,
        &domain.hnf_type,
        &domain.object_id,
        &domain.content_hash,
    );

    if domain.properties.lines.is_empty() {
        errors.push(DomainValidationError {
            field: "properties.lines".into(),
            message: "required non-empty array".into(),
        });
    }

    errors.extend(non_empty_ids(
        domain.properties.lines.iter(),
        "properties.lines",
        |l| &l.line_id,
    ));

    for (i, line) in domain.properties.lines.iter().enumerate() {
        if line.quantity == 0 {
            errors.push(DomainValidationError {
                field: format!("properties.lines[{i}].quantity"),
                message: "must be >= 1".into(),
            });
        }

        let has_refdes = line.refdes.iter().any(|r| !r.trim().is_empty());
        let has_mpn = line.mpn.as_ref().is_some_and(|m| !m.trim().is_empty());
        if !has_refdes && !has_mpn {
            errors.push(DomainValidationError {
                field: format!("properties.lines[{i}]"),
                message: "each line requires at least one refdes or mpn".into(),
            });
        }

        for (j, refdes) in line.refdes.iter().enumerate() {
            if refdes.trim().is_empty() {
                errors.push(DomainValidationError {
                    field: format!("properties.lines[{i}].refdes[{j}]"),
                    message: "must be non-empty when present".into(),
                });
            }
        }
    }

    finish_validation(errors)
}

/// Serialize BOM domain to JSON value.
pub fn serialize_bom(domain: &BomDomain) -> Value {
    serde_json::to_value(domain).expect("bom domain serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::HNF_TYPE_OBJECT;

    fn minimal_bom() -> BomDomain {
        BomDomain {
            domain: BOM_DOMAIN.into(),
            version: BOM_VERSION.into(),
            hnf_type: HNF_TYPE_OBJECT.into(),
            object_id: "550e8400-e29b-41d4-a716-446655440002".into(),
            content_hash: None,
            refs: vec!["550e8400-e29b-41d4-a716-446655440001".into()],
            properties: BomProperties {
                lines: vec![BomLine {
                    line_id: "line-1".into(),
                    quantity: 1,
                    refdes: vec!["R1".into()],
                    mpn: Some("RC0603FR-0710KL".into()),
                    manufacturer: Some("Yageo".into()),
                    supplier_ref: Some("digkey:311-10.0KCRCT-ND".into()),
                    lifecycle: Some("active".into()),
                    description: Some("10k 1% 0603".into()),
                }],
            },
        }
    }

    #[test]
    fn validate_accepts_minimal_bom() {
        assert!(validate_bom(&minimal_bom()).is_ok());
    }

    #[test]
    fn validate_rejects_empty_lines() {
        let mut doc = minimal_bom();
        doc.properties.lines.clear();
        let errs = validate_bom(&doc).expect_err("empty lines");
        assert!(errs.iter().any(|e| e.field == "properties.lines"));
    }

    #[test]
    fn validate_rejects_zero_quantity() {
        let mut doc = minimal_bom();
        doc.properties.lines[0].quantity = 0;
        let errs = validate_bom(&doc).expect_err("zero qty");
        assert!(errs.iter().any(|e| e.field.contains("quantity")));
    }

    #[test]
    fn parse_requires_mpn_or_refdes() {
        let mut doc = minimal_bom();
        doc.properties.lines[0].refdes.clear();
        doc.properties.lines[0].mpn = None;
        let err = parse_bom(&serialize_bom(&doc)).expect_err("no identifier");
        assert!(matches!(err, DomainParseError::Validation(_)));
    }

    #[test]
    fn roundtrip_serialize_parse() {
        let doc = minimal_bom();
        let parsed = parse_bom(&serialize_bom(&doc)).expect("roundtrip");
        assert_eq!(parsed, doc);
    }

    #[test]
    fn mpn_only_line_valid() {
        let doc = BomDomain {
            domain: BOM_DOMAIN.into(),
            version: BOM_VERSION.into(),
            hnf_type: HNF_TYPE_OBJECT.into(),
            object_id: "550e8400-e29b-41d4-a716-446655440003".into(),
            content_hash: None,
            refs: vec![],
            properties: BomProperties {
                lines: vec![BomLine {
                    line_id: "line-cap".into(),
                    quantity: 10,
                    refdes: vec![],
                    mpn: Some("GRM188R71C104KA01D".into()),
                    manufacturer: None,
                    supplier_ref: None,
                    lifecycle: None,
                    description: None,
                }],
            },
        };
        assert!(validate_bom(&doc).is_ok());
    }
}
