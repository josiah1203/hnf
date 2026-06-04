//! HNF domain payloads (Phase 0 v0.1).

mod bom;
mod schematic;

pub use bom::{
    parse_bom, serialize_bom, validate_bom, BomDomain, BomLine, BomProperties, BOM_DOMAIN,
    BOM_VERSION,
};
pub use schematic::{
    parse_schematic, serialize_schematic, validate_schematic, SchematicDomain, SchematicNet,
    SchematicPin, SchematicPowerDomain, SchematicProperties, SchematicSymbol, SCHEMATIC_DOMAIN,
    SCHEMATIC_VERSION,
};

/// Shared domain envelope per `spec/spec-v0.1.md`.
pub const HNF_TYPE_OBJECT: &str = "hardware.object";
pub const DOMAIN_VERSION_V0_1: &str = "0.1";

/// Domain validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for DomainValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Parse/validate failure (serde or domain rules).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainParseError {
    Serde(String),
    Validation(Vec<DomainValidationError>),
}

impl std::fmt::Display for DomainParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serde(msg) => write!(f, "serde: {msg}"),
            Self::Validation(errs) => {
                let parts: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
                write!(f, "validation: {}", parts.join("; "))
            }
        }
    }
}

impl std::error::Error for DomainParseError {}

pub(crate) fn validate_envelope(
    domain: &str,
    expected_domain: &str,
    version: &str,
    expected_version: &str,
    hnf_type: &str,
    object_id: &str,
    content_hash: &Option<String>,
) -> Vec<DomainValidationError> {
    let mut errors = Vec::new();

    if domain != expected_domain {
        errors.push(DomainValidationError {
            field: "domain".into(),
            message: format!("expected \"{expected_domain}\", got \"{domain}\""),
        });
    }

    if version != expected_version {
        errors.push(DomainValidationError {
            field: "version".into(),
            message: format!("expected \"{expected_version}\", got \"{version}\""),
        });
    }

    if hnf_type.trim().is_empty() {
        errors.push(DomainValidationError {
            field: "hnf_type".into(),
            message: "required non-empty string".into(),
        });
    }

    if object_id.trim().is_empty() {
        errors.push(DomainValidationError {
            field: "object_id".into(),
            message: "required non-empty string".into(),
        });
    }

    if let Some(hash) = content_hash {
        let h = hash.trim();
        if h.is_empty() {
            errors.push(DomainValidationError {
                field: "content_hash".into(),
                message: "must be non-empty when present".into(),
            });
        } else if h.len() != 64 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
            errors.push(DomainValidationError {
                field: "content_hash".into(),
                message: "must be 64 hex characters (sha256)".into(),
            });
        }
    }

    errors
}

pub(crate) fn non_empty_ids<'a, I>(
    items: I,
    id_field: &'static str,
    get_id: fn(&I::Item) -> &str,
) -> Vec<DomainValidationError>
where
    I: IntoIterator,
    I::Item: 'a,
{
    let mut errors = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (i, item) in items.into_iter().enumerate() {
        let id = get_id(&item);
        if id.trim().is_empty() {
            errors.push(DomainValidationError {
                field: format!("{id_field}[{i}]"),
                message: "id must be non-empty".into(),
            });
        } else if !seen.insert(id.to_string()) {
            errors.push(DomainValidationError {
                field: format!("{id_field}[{i}]"),
                message: format!("duplicate id \"{id}\""),
            });
        }
    }
    errors
}

pub(crate) fn finish_validation(errors: Vec<DomainValidationError>) -> Result<(), Vec<DomainValidationError>> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_envelope_rejects_bad_content_hash() {
        let errs = validate_envelope(
            SCHEMATIC_DOMAIN,
            SCHEMATIC_DOMAIN,
            SCHEMATIC_VERSION,
            SCHEMATIC_VERSION,
            HNF_TYPE_OBJECT,
            "obj-1",
            &Some("not-a-hash".into()),
        );
        assert!(errs.iter().any(|e| e.field == "content_hash"));
    }
}
