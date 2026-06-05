//! Integration tests using JSON fixtures under `tests/fixtures/`.

use hnf_core::{
    parse_bom, parse_domain, parse_firmware, parse_ic_layout, parse_layout, parse_mechanical,
    parse_schematic, parse_simulation, serialize_bom, serialize_firmware, serialize_ic_layout,
    serialize_layout, serialize_mechanical, serialize_schematic, serialize_simulation,
    DomainParseError, PHASE0_RUST_DOMAINS,
};
use serde_json::Value;

fn load_fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

#[test]
fn phase0_fixtures_cover_all_rust_domains() {
    assert_eq!(PHASE0_RUST_DOMAINS.len(), 7);
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

#[test]
fn fixture_layout_valid_roundtrips() {
    let value = load_fixture("layout_valid.json");
    let domain = parse_layout(&value).expect("valid layout fixture");
    assert_eq!(domain.properties.footprints[0].refdes, "U1");
    let again = parse_layout(&serialize_layout(&domain)).expect("roundtrip");
    assert_eq!(again, domain);
}

#[test]
fn fixture_layout_invalid_empty_geometry() {
    let value = load_fixture("layout_invalid_empty_geometry.json");
    let err = parse_layout(&value).expect_err("empty geometry");
    assert!(matches!(err, DomainParseError::Validation(_)));
}

#[test]
fn fixture_ic_layout_valid_roundtrips() {
    let value = load_fixture("ic_layout_valid.json");
    let domain = parse_ic_layout(&value).expect("valid ic_layout fixture");
    assert_eq!(domain.properties.shapes.len(), 1);
    let again = parse_ic_layout(&serialize_ic_layout(&domain)).expect("roundtrip");
    assert_eq!(again, domain);
}

#[test]
fn fixture_ic_layout_invalid_dangling_layer() {
    let value = load_fixture("ic_layout_invalid_dangling_layer.json");
    let err = parse_ic_layout(&value).expect_err("dangling layer");
    if let DomainParseError::Validation(errs) = err {
        assert!(errs.iter().any(|e| e.field.contains("layer_id")));
    } else {
        panic!("expected validation error");
    }
}

#[test]
fn fixture_mechanical_valid_roundtrips() {
    let value = load_fixture("mechanical_valid.json");
    let domain = parse_mechanical(&value).expect("valid mechanical fixture");
    assert_eq!(domain.properties.solids[0].name, "Mounting Bracket");
    let again = parse_mechanical(&serialize_mechanical(&domain)).expect("roundtrip");
    assert_eq!(again, domain);
}

#[test]
fn fixture_mechanical_invalid_empty_solids() {
    let value = load_fixture("mechanical_invalid_empty_solids.json");
    let err = parse_mechanical(&value).expect_err("empty solids");
    assert!(matches!(err, DomainParseError::Validation(_)));
}

#[test]
fn fixture_simulation_valid_roundtrips() {
    let value = load_fixture("simulation_valid.json");
    let domain = parse_simulation(&value).expect("valid simulation fixture");
    assert_eq!(domain.properties.models[0].solver.as_deref(), Some("ngspice"));
    let again = parse_simulation(&serialize_simulation(&domain)).expect("roundtrip");
    assert_eq!(again, domain);
}

#[test]
fn fixture_simulation_invalid_empty_models() {
    let value = load_fixture("simulation_invalid_zero_nodes.json");
    let err = parse_simulation(&value).expect_err("empty models");
    if let DomainParseError::Validation(errs) = err {
        assert!(errs.iter().any(|e| e.field == "properties.models"));
    } else {
        panic!("expected validation error");
    }
}

#[test]
fn fixture_firmware_valid_roundtrips() {
    let value = load_fixture("firmware_valid.json");
    let domain = parse_firmware(&value).expect("valid firmware fixture");
    assert_eq!(domain.properties.targets[0].board.as_deref(), Some("esp32dev"));
    let again = parse_firmware(&serialize_firmware(&domain)).expect("roundtrip");
    assert_eq!(again, domain);
}

#[test]
fn fixture_firmware_invalid_empty_targets() {
    let value = load_fixture("firmware_invalid_empty_targets.json");
    let err = parse_firmware(&value).expect_err("empty targets");
    assert!(matches!(err, DomainParseError::Validation(_)));
}

#[test]
fn parse_domain_dispatches_from_fixtures() {
    for name in [
        "schematic_valid.json",
        "layout_valid.json",
        "ic_layout_valid.json",
        "mechanical_valid.json",
        "simulation_valid.json",
        "bom_valid.json",
        "firmware_valid.json",
    ] {
        let value = load_fixture(name);
        parse_domain(&value).unwrap_or_else(|e| panic!("parse_domain {name}: {e}"));
    }
}
