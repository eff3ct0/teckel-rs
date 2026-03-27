use thiserror::Error;

/// Error codes from the Teckel v2.0 specification (Section 25).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeckelErrorCode {
    // Schema
    EReq001, // Missing required field
    // Naming
    EName001, // Duplicate AssetRef
    EName002, // Invalid AssetRef format
    EName003, // Column name collision after rename
    // Reference
    ERef001,   // Undefined asset reference
    ERef002,   // Invalid output reference (output references output)
    ECycle001, // Circular dependency detected
    // Format
    EFmt001,  // Unknown data format
    EMode001, // Unknown write mode
    // Transformation
    EOp001, // Zero or multiple operation keys in transformation
    EOp002, // Unknown operation key
    // Schema validation
    EList001, // Empty list where NonEmptyList required
    EEnum001, // Invalid enum value
    // Column
    ECol001, // Column not found in dataset
    // Join
    EJoin001, // Ambiguous column reference in join condition
    // Aggregation
    EAgg001, // Non-aggregate expression in group-by output
    // Expression
    EExpr001, // Expression type mismatch
    // Schema compatibility
    ESchema001, // Incompatible schemas in set operation
    ESchema002, // Operation would produce empty schema
    ESchema003, // Unexpected extra columns in strict mode
    ESchema004, // Missing expected columns in strict mode
    // Type
    EType001, // Incompatible types, cannot widen
    // I/O
    EIo001, // Input path not found or unreadable
    EIo002, // Output destination already exists (error mode)
    // Substitution
    EVar001, // Unresolved variable with no default
    // Secrets
    ESecret001, // Unresolved secret reference
    // Hooks
    EHook001, // Pre-execution hook failed
    // Custom
    EComp001, // Unregistered custom component
    // Quality
    EQuality001, // Assertion or quality check failed
    EQuality002, // Unknown quality check type
    EQuality003, // Invalid threshold value
    EQuality004, // Freshness check failed
    EQuality005, // Referential integrity check failed
    // Metadata
    EMeta001, // Invalid owner type
    EMeta002, // Invalid maturity value
    EMeta003, // Invalid freshness duration
    EMeta004, // Column metadata references non-existent column
    // Exposures
    EExpose001, // Exposure depends_on references undefined asset
    EExpose002, // Unknown exposure type
    // Version
    EVersion001, // Missing or unsupported version field
}

impl std::fmt::Display for TeckelErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            Self::EReq001 => "E-REQ-001",
            Self::EName001 => "E-NAME-001",
            Self::EName002 => "E-NAME-002",
            Self::EName003 => "E-NAME-003",
            Self::ERef001 => "E-REF-001",
            Self::ERef002 => "E-REF-002",
            Self::ECycle001 => "E-CYCLE-001",
            Self::EFmt001 => "E-FMT-001",
            Self::EMode001 => "E-MODE-001",
            Self::EOp001 => "E-OP-001",
            Self::EOp002 => "E-OP-002",
            Self::EList001 => "E-LIST-001",
            Self::EEnum001 => "E-ENUM-001",
            Self::ECol001 => "E-COL-001",
            Self::EJoin001 => "E-JOIN-001",
            Self::EAgg001 => "E-AGG-001",
            Self::EExpr001 => "E-EXPR-001",
            Self::ESchema001 => "E-SCHEMA-001",
            Self::ESchema002 => "E-SCHEMA-002",
            Self::ESchema003 => "E-SCHEMA-003",
            Self::ESchema004 => "E-SCHEMA-004",
            Self::EType001 => "E-TYPE-001",
            Self::EIo001 => "E-IO-001",
            Self::EIo002 => "E-IO-002",
            Self::EVar001 => "E-VAR-001",
            Self::ESecret001 => "E-SECRET-001",
            Self::EHook001 => "E-HOOK-001",
            Self::EComp001 => "E-COMP-001",
            Self::EQuality001 => "E-QUALITY-001",
            Self::EQuality002 => "E-QUALITY-002",
            Self::EQuality003 => "E-QUALITY-003",
            Self::EQuality004 => "E-QUALITY-004",
            Self::EQuality005 => "E-QUALITY-005",
            Self::EMeta001 => "E-META-001",
            Self::EMeta002 => "E-META-002",
            Self::EMeta003 => "E-META-003",
            Self::EMeta004 => "E-META-004",
            Self::EExpose001 => "E-EXPOSE-001",
            Self::EExpose002 => "E-EXPOSE-002",
            Self::EVersion001 => "E-VERSION-001",
        };
        write!(f, "{code}")
    }
}

/// Top-level error type for the Teckel pipeline.
#[derive(Debug, Error)]
pub enum TeckelError {
    #[error("[{code}] {message}")]
    Spec {
        code: TeckelErrorCode,
        message: String,
    },

    #[error("YAML parse error: {0}")]
    Yaml(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation failed with {count} error(s)")]
    Validation {
        count: usize,
        errors: Vec<TeckelError>,
    },

    #[error("Execution error: {0}")]
    Execution(String),
}

impl TeckelError {
    pub fn spec(code: TeckelErrorCode, message: impl Into<String>) -> Self {
        Self::Spec {
            code,
            message: message.into(),
        }
    }
}
