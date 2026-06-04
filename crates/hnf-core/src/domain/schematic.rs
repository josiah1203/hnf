//! HNF schematic domain — nets, symbols, pins, power domains.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    finish_validation, non_empty_ids, validate_envelope, DomainParseError, DomainValidationError,
    DOMAIN_VERSION_V0_1,
};

pub const SCHEMATIC_DOMAIN: &str = "schematic";
pub const SCHEMATIC_VERSION: &str = DOMAIN_VERSION_V0_1;

/// Full schematic domain document (envelope + typed properties).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchematicDomain {
    pub domain: String,
    pub version: String,
    pub hnf_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    pub properties: SchematicProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SchematicProperties {
    #[serde(default)]
    pub symbols: Vec<SchematicSymbol>,
    #[serde(default)]
    pub nets: Vec<SchematicNet>,
    #[serde(default)]
    pub pins: Vec<SchematicPin>,
    #[serde(default, rename = "power_domains")]
    pub power_domains: Vec<SchematicPowerDomain>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchematicSymbol {
    pub id: String,
    pub refdes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lib_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchematicNet {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchematicPin {
    pub symbol_id: String,
    pub pin_number: String,
    pub net_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchematicPowerDomain {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voltage: Option<String>,
}

/// Deserialize JSON and validate schematic domain rules.
pub fn parse_schematic(value: &Value) -> Result<SchematicDomain, DomainParseError> {
    let domain: SchematicDomain = serde_json::from_value(value.clone()).map_err(|e| {
        DomainParseError::Serde(e.to_string())
    })?;
    validate_schematic(&domain).map_err(DomainParseError::Validation)?;
    Ok(domain)
}

/// Validate schematic domain envelope and cross-references.
pub fn validate_schematic(domain: &SchematicDomain) -> Result<(), Vec<DomainValidationError>> {
    let mut errors = validate_envelope(
        &domain.domain,
        SCHEMATIC_DOMAIN,
        &domain.version,
        SCHEMATIC_VERSION,
        &domain.hnf_type,
        &domain.object_id,
        &domain.content_hash,
    );

    errors.extend(non_empty_ids(
        domain.properties.symbols.iter(),
        "properties.symbols",
        |s| &s.id,
    ));

    let mut refdes_seen = std::collections::HashSet::new();
    for (i, sym) in domain.properties.symbols.iter().enumerate() {
        if sym.refdes.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.symbols[{i}].refdes"),
                message: "required non-empty string".into(),
            });
        } else if !refdes_seen.insert(sym.refdes.clone()) {
            errors.push(DomainValidationError {
                field: format!("properties.symbols[{i}].refdes"),
                message: format!("duplicate refdes \"{}\"", sym.refdes),
            });
        }
    }

    errors.extend(non_empty_ids(
        domain.properties.nets.iter(),
        "properties.nets",
        |n| &n.id,
    ));

    let mut net_names = std::collections::HashSet::new();
    for (i, net) in domain.properties.nets.iter().enumerate() {
        if net.name.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.nets[{i}].name"),
                message: "required non-empty string".into(),
            });
        } else if !net_names.insert(net.name.clone()) {
            errors.push(DomainValidationError {
                field: format!("properties.nets[{i}].name"),
                message: format!("duplicate net name \"{}\"", net.name),
            });
        }
    }

    errors.extend(non_empty_ids(
        domain.properties.power_domains.iter(),
        "properties.power_domains",
        |p| &p.id,
    ));

    for (i, pd) in domain.properties.power_domains.iter().enumerate() {
        if pd.name.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.power_domains[{i}].name"),
                message: "required non-empty string".into(),
            });
        }
    }

    let symbol_ids: std::collections::HashSet<_> =
        domain.properties.symbols.iter().map(|s| s.id.as_str()).collect();
    let net_ids: std::collections::HashSet<_> =
        domain.properties.nets.iter().map(|n| n.id.as_str()).collect();

    for (i, pin) in domain.properties.pins.iter().enumerate() {
        if pin.pin_number.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.pins[{i}].pin_number"),
                message: "required non-empty string".into(),
            });
        }
        if !symbol_ids.contains(pin.symbol_id.as_str()) {
            errors.push(DomainValidationError {
                field: format!("properties.pins[{i}].symbol_id"),
                message: format!("unknown symbol_id \"{}\"", pin.symbol_id),
            });
        }
        if !net_ids.contains(pin.net_id.as_str()) {
            errors.push(DomainValidationError {
                field: format!("properties.pins[{i}].net_id"),
                message: format!("unknown net_id \"{}\"", pin.net_id),
            });
        }
    }

    finish_validation(errors)
}

/// Serialize schematic domain to JSON value.
pub fn serialize_schematic(domain: &SchematicDomain) -> Value {
    serde_json::to_value(domain).expect("schematic domain serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::HNF_TYPE_OBJECT;

    fn minimal_schematic() -> SchematicDomain {
        SchematicDomain {
            domain: SCHEMATIC_DOMAIN.into(),
            version: SCHEMATIC_VERSION.into(),
            hnf_type: HNF_TYPE_OBJECT.into(),
            object_id: "550e8400-e29b-41d4-a716-446655440001".into(),
            content_hash: None,
            refs: vec![],
            properties: SchematicProperties {
                symbols: vec![SchematicSymbol {
                    id: "sym-r1".into(),
                    refdes: "R1".into(),
                    lib_id: Some("Device:R".into()),
                    value: Some("10k".into()),
                }],
                nets: vec![SchematicNet {
                    id: "net-gnd".into(),
                    name: "GND".into(),
                    net_class: Some("power".into()),
                }],
                pins: vec![SchematicPin {
                    symbol_id: "sym-r1".into(),
                    pin_number: "1".into(),
                    net_id: "net-gnd".into(),
                }],
                power_domains: vec![SchematicPowerDomain {
                    id: "pd-3v3".into(),
                    name: "+3V3".into(),
                    voltage: Some("3.3V".into()),
                }],
            },
        }
    }

    #[test]
    fn validate_accepts_minimal_schematic() {
        assert!(validate_schematic(&minimal_schematic()).is_ok());
    }

    #[test]
    fn parse_rejects_wrong_domain() {
        let mut doc = minimal_schematic();
        doc.domain = "layout".into();
        let value = serialize_schematic(&doc);
        let err = parse_schematic(&value).expect_err("wrong domain");
        assert!(matches!(err, DomainParseError::Validation(_)));
    }

    #[test]
    fn parse_rejects_dangling_pin_reference() {
        let mut doc = minimal_schematic();
        doc.properties.pins[0].net_id = "net-missing".into();
        let err = parse_schematic(&serialize_schematic(&doc)).expect_err("dangling pin");
        if let DomainParseError::Validation(errs) = err {
            assert!(errs.iter().any(|e| e.field.contains("net_id")));
        } else {
            panic!("expected validation error");
        }
    }

    #[test]
    fn roundtrip_serialize_parse() {
        let doc = minimal_schematic();
        let value = serialize_schematic(&doc);
        let parsed = parse_schematic(&value).expect("roundtrip");
        assert_eq!(parsed, doc);
    }

    #[test]
    fn duplicate_refdes_rejected() {
        let mut doc = minimal_schematic();
        doc.properties.symbols.push(SchematicSymbol {
            id: "sym-r2".into(),
            refdes: "R1".into(),
            lib_id: None,
            value: None,
        });
        let errs = validate_schematic(&doc).expect_err("duplicate refdes");
        assert!(errs.iter().any(|e| e.field.contains("refdes")));
    }
}
