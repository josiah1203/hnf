//! Integration tests using JSON fixtures under `tests/fixtures/`.

use hnf_core::{parse_bom, parse_schematic, serialize_bom, serialize_schematic, DomainParseError};
use serde_json::Value;

fn load_fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

#[test]
fn fixture_schematic_valid_roundtrips() {
    let value = load_fixture("schematic_valid.json");
    let domain = parse_schematic(&value).expect("valid schematic fixture");
    assert_eq!(domain.properties.symbols.len(), 1);
    assert_eq!(domain.properties.nets[0].name, "GND");

    let encoded = serialize_schematic(&domain);
    let again = parse_schematic(&encoded).expect("roundtrip");
    assert_eq!(again, domain);
}

#[test]
fn fixture_schematic_invalid_wrong_domain() {
    let value = load_fixture("schematic_invalid_wrong_domain.json");
    let err = parse_schematic(&value).expect_err("wrong domain");
    assert!(matches!(err, DomainParseError::Validation(_)));
}

#[test]
fn fixture_schematic_invalid_dangling_pin() {
    let value = load_fixture("schematic_invalid_dangling_pin.json");
    let err = parse_schematic(&value).expect_err("dangling pin");
    if let DomainParseError::Validation(errs) = err {
        assert!(errs.iter().any(|e| e.field.contains("net_id")));
    } else {
        panic!("expected validation error");
    }
}

#[test]
fn fixture_bom_valid_roundtrips() {
    let value = load_fixture("bom_valid.json");
    let domain = parse_bom(&value).expect("valid bom fixture");
    assert_eq!(domain.properties.lines.len(), 1);
    assert_eq!(domain.properties.lines[0].mpn.as_deref(), Some("RC0603FR-0710KL"));

    let encoded = serialize_bom(&domain);
    let again = parse_bom(&encoded).expect("roundtrip");
    assert_eq!(again, domain);
}

#[test]
fn fixture_bom_invalid_empty_lines() {
    let value = load_fixture("bom_invalid_empty_lines.json");
    let err = parse_bom(&value).expect_err("empty lines");
    assert!(matches!(err, DomainParseError::Validation(_)));
}

#[test]
fn fixture_bom_invalid_zero_quantity() {
    let value = load_fixture("bom_invalid_zero_quantity.json");
    let err = parse_bom(&value).expect_err("zero quantity");
    if let DomainParseError::Validation(errs) = err {
        assert!(errs.iter().any(|e| e.field.contains("quantity")));
    } else {
        panic!("expected validation error");
    }
}
