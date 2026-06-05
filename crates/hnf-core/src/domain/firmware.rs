//! HNF firmware domain — build targets, sources, and HNFT artifact refs.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    finish_validation, non_empty_ids, validate_envelope, DomainParseError, DomainValidationError,
    DOMAIN_VERSION_V0_1,
};

pub const FIRMWARE_DOMAIN: &str = "firmware";
pub const FIRMWARE_VERSION: &str = DOMAIN_VERSION_V0_1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirmwareDomain {
    pub domain: String,
    pub version: String,
    pub hnf_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    pub properties: FirmwareProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FirmwareProperties {
    #[serde(default)]
    pub targets: Vec<FirmwareTarget>,
    #[serde(default)]
    pub sources: Vec<FirmwareSource>,
    #[serde(default)]
    pub artifacts: Vec<FirmwareArtifact>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirmwareTarget {
    pub id: String,
    pub platform: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirmwareSource {
    pub id: String,
    pub path: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirmwareArtifact {
    pub id: String,
    pub name: String,
    pub content_hash: String,
}

pub fn parse_firmware(value: &Value) -> Result<FirmwareDomain, DomainParseError> {
    let domain: FirmwareDomain = serde_json::from_value(value.clone())
        .map_err(|e| DomainParseError::Serde(e.to_string()))?;
    validate_firmware(&domain).map_err(DomainParseError::Validation)?;
    Ok(domain)
}

pub fn validate_firmware(domain: &FirmwareDomain) -> Result<(), Vec<DomainValidationError>> {
    let mut errors = validate_envelope(
        &domain.domain,
        FIRMWARE_DOMAIN,
        &domain.version,
        FIRMWARE_VERSION,
        &domain.hnf_type,
        &domain.object_id,
        &domain.content_hash,
    );

    if domain.properties.targets.is_empty() {
        errors.push(DomainValidationError {
            field: "properties.targets".into(),
            message: "required non-empty array".into(),
        });
    }

    errors.extend(non_empty_ids(
        domain.properties.targets.iter(),
        "properties.targets",
        |t| &t.id,
    ));

    for (i, target) in domain.properties.targets.iter().enumerate() {
        if target.platform.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.targets[{i}].platform"),
                message: "required non-empty string".into(),
            });
        }
    }

    errors.extend(non_empty_ids(
        domain.properties.sources.iter(),
        "properties.sources",
        |s| &s.id,
    ));

    let allowed_langs = ["c", "cpp", "asm", "rust", "verilog", "python"];
    for (i, source) in domain.properties.sources.iter().enumerate() {
        if source.path.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.sources[{i}].path"),
                message: "required non-empty string".into(),
            });
        }
        if !allowed_langs.contains(&source.language.as_str()) {
            errors.push(DomainValidationError {
                field: format!("properties.sources[{i}].language"),
                message: format!(
                    "unsupported language \"{}\"; expected one of {}",
                    source.language,
                    allowed_langs.join(", ")
                ),
            });
        }
    }

    errors.extend(non_empty_ids(
        domain.properties.artifacts.iter(),
        "properties.artifacts",
        |a| &a.id,
    ));

    for (i, artifact) in domain.properties.artifacts.iter().enumerate() {
        if artifact.name.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("properties.artifacts[{i}].name"),
                message: "required non-empty string".into(),
            });
        }
        let h = artifact.content_hash.trim();
        if h.len() != 64 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
            errors.push(DomainValidationError {
                field: format!("properties.artifacts[{i}].content_hash"),
                message: "must be 64 hex characters (sha256 HNFT ref)".into(),
            });
        }
    }

    if domain.properties.sources.is_empty() && domain.properties.artifacts.is_empty() {
        errors.push(DomainValidationError {
            field: "properties".into(),
            message: "requires at least one source or artifact".into(),
        });
    }

    finish_validation(errors)
}

pub fn serialize_firmware(domain: &FirmwareDomain) -> Value {
    serde_json::to_value(domain).expect("firmware domain serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::HNF_TYPE_OBJECT;

    const SAMPLE_HASH: &str =
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn minimal_firmware() -> FirmwareDomain {
        FirmwareDomain {
            domain: FIRMWARE_DOMAIN.into(),
            version: FIRMWARE_VERSION.into(),
            hnf_type: HNF_TYPE_OBJECT.into(),
            object_id: "550e8400-e29b-41d4-a716-446655440014".into(),
            content_hash: None,
            refs: vec![],
            properties: FirmwareProperties {
                targets: vec![FirmwareTarget {
                    id: "tgt-esp32".into(),
                    platform: "espressif32".into(),
                    board: Some("esp32dev".into()),
                }],
                sources: vec![FirmwareSource {
                    id: "src-main".into(),
                    path: "src/main.cpp".into(),
                    language: "cpp".into(),
                }],
                artifacts: vec![FirmwareArtifact {
                    id: "art-bin".into(),
                    name: "firmware.bin".into(),
                    content_hash: SAMPLE_HASH.into(),
                }],
            },
        }
    }

    #[test]
    fn validate_accepts_minimal_firmware() {
        assert!(validate_firmware(&minimal_firmware()).is_ok());
    }

    #[test]
    fn validate_rejects_empty_targets() {
        let mut doc = minimal_firmware();
        doc.properties.targets.clear();
        let errs = validate_firmware(&doc).expect_err("no targets");
        assert!(errs.iter().any(|e| e.field == "properties.targets"));
    }

    #[test]
    fn validate_rejects_bad_artifact_hash() {
        let mut doc = minimal_firmware();
        doc.properties.artifacts[0].content_hash = "short".into();
        let errs = validate_firmware(&doc).expect_err("bad hash");
        assert!(errs.iter().any(|e| e.field.contains("content_hash")));
    }

    #[test]
    fn artifacts_only_firmware_valid() {
        let doc = FirmwareDomain {
            properties: FirmwareProperties {
                targets: vec![FirmwareTarget {
                    id: "tgt-1".into(),
                    platform: "native".into(),
                    board: None,
                }],
                sources: vec![],
                artifacts: vec![FirmwareArtifact {
                    id: "art-hex".into(),
                    name: "image.hex".into(),
                    content_hash: SAMPLE_HASH.into(),
                }],
            },
            ..minimal_firmware()
        };
        assert!(validate_firmware(&doc).is_ok());
    }

    #[test]
    fn roundtrip_serialize_parse() {
        let doc = minimal_firmware();
        let parsed = parse_firmware(&serialize_firmware(&doc)).expect("roundtrip");
        assert_eq!(parsed, doc);
    }
}
