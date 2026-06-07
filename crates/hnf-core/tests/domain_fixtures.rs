//! Integration tests using JSON fixtures under `tests/fixtures/`.

use hnf_core::{
    parse_bim, parse_bom, parse_domain, parse_energy_building, parse_firmware, parse_geospatial,
    parse_ic_layout, parse_layout, parse_mechanical, parse_schematic, parse_simulation,
    parse_structural, serialize_bim, serialize_bom, serialize_energy_building, serialize_firmware,
    serialize_geospatial, serialize_ic_layout, serialize_layout, serialize_mechanical,
    serialize_schematic, serialize_simulation, serialize_structural, DomainParseError,
    PHASE0_RUST_DOMAINS, PHASE1_RUST_DOMAINS,
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
fn phase1_fixtures_cover_all_rust_domains() {
    assert_eq!(PHASE1_RUST_DOMAINS.len(), 4);
}

#[test]
fn fixture_bim_valid_roundtrips() {
    let value = load_fixture("bim_valid.json");
    let domain = parse_bim(&value).expect("valid bim fixture");
    assert_eq!(domain.properties.elements[0].ifc_class, "IfcWall");
    let again = parse_bim(&serialize_bim(&domain)).expect("roundtrip");
    assert_eq!(again, domain);
}

#[test]
fn fixture_bim_invalid_unknown_storey() {
    let value = load_fixture("bim_invalid_unknown_storey.json");
    let err = parse_bim(&value).expect_err("unknown storey");
    if let DomainParseError::Validation(errs) = err {
        assert!(errs.iter().any(|e| e.field.contains("storey_id")));
    } else {
        panic!("expected validation error");
    }
}

#[test]
fn fixture_geospatial_valid_roundtrips() {
    let value = load_fixture("geospatial_valid.json");
    let domain = parse_geospatial(&value).expect("valid geospatial fixture");
    assert_eq!(domain.properties.crs, "EPSG:4326");
    let again = parse_geospatial(&serialize_geospatial(&domain)).expect("roundtrip");
    assert_eq!(again, domain);
}

#[test]
fn fixture_geospatial_invalid_empty_layers() {
    let value = load_fixture("geospatial_invalid_empty_layers.json");
    let err = parse_geospatial(&value).expect_err("empty layers");
    assert!(matches!(err, DomainParseError::Validation(_)));
}

#[test]
fn fixture_structural_valid_roundtrips() {
    let value = load_fixture("structural_valid.json");
    let domain = parse_structural(&value).expect("valid structural fixture");
    assert_eq!(domain.properties.members[0].section, "W12x26");
    let again = parse_structural(&serialize_structural(&domain)).expect("roundtrip");
    assert_eq!(again, domain);
}

#[test]
fn fixture_structural_invalid_empty_members() {
    let value = load_fixture("structural_invalid_empty_members.json");
    let err = parse_structural(&value).expect_err("empty members");
    assert!(matches!(err, DomainParseError::Validation(_)));
}

#[test]
fn fixture_energy_building_valid_roundtrips() {
    let value = load_fixture("energy_building_valid.json");
    let domain = parse_energy_building(&value).expect("valid energy_building fixture");
    assert_eq!(domain.properties.zones[0].name, "Open Office");
    let again = parse_energy_building(&serialize_energy_building(&domain)).expect("roundtrip");
    assert_eq!(again, domain);
}

#[test]
fn fixture_energy_building_invalid_empty_zones() {
    let value = load_fixture("energy_building_invalid_empty_zones.json");
    let err = parse_energy_building(&value).expect_err("empty zones");
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
        "bim_valid.json",
        "geospatial_valid.json",
        "structural_valid.json",
        "energy_building_valid.json",
    ] {
        let value = load_fixture(name);
        parse_domain(&value).unwrap_or_else(|e| panic!("parse_domain {name}: {e}"));
    }
}
