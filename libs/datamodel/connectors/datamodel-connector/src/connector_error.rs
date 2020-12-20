use dml::native_type_instance::NativeTypeInstance;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
#[error("{}", kind)]
pub struct ConnectorError {
    /// The error information for internal use.
    pub kind: ErrorKind,
}

pub struct ConnectorErrorFactory {
    pub native_type: String,
    pub connector: String,
}

impl ConnectorErrorFactory {
    pub fn new(tpe: NativeTypeInstance, connector: String) -> Self {
        ConnectorErrorFactory {
            native_type: tpe.render(),
            connector,
        }
    }

    pub fn new_scale_larger_than_precision_error(self) -> Result<(), ConnectorError> {
        Err(ConnectorError::from_kind(ErrorKind::ScaleLargerThanPrecisionError {
            native_type: self.native_type,
            connector_name: self.connector,
        }))
    }

    pub fn new_incompatible_native_type_with_index(self) -> Result<(), ConnectorError> {
        Err(ConnectorError::from_kind(
            ErrorKind::IncompatibleNativeTypeWithIndexAttribute {
                native_type: self.native_type,
                connector_name: self.connector,
            },
        ))
    }

    pub fn new_incompatible_native_type_with_unique(self) -> Result<(), ConnectorError> {
        Err(ConnectorError::from_kind(
            ErrorKind::IncompatibleNativeTypeWithUniqueAttribute {
                native_type: self.native_type,
                connector_name: self.connector,
            },
        ))
    }

    pub fn new_incompatible_native_type_with_id(self) -> Result<(), ConnectorError> {
        Err(ConnectorError::from_kind(
            ErrorKind::IncompatibleNativeTypeWithIdAttribute {
                native_type: self.native_type,
                connector_name: self.connector,
            },
        ))
    }

    pub fn new_incompatible_sequential_type_with_static_default_value_error(self) -> Result<(), ConnectorError> {
        Err(ConnectorError::from_kind(
            ErrorKind::IncompatibleSequentialTypeWithStaticDefaultValue {
                native_type: self.native_type,
                connector_name: self.connector,
            },
        ))
    }

    pub fn new_argument_m_out_of_range_error(self, message: &str) -> Result<(), ConnectorError> {
        Err(ConnectorError::from_kind(ErrorKind::ArgumentOutOfRangeError {
            native_type: self.native_type,
            connector_name: self.connector,
            message: String::from(message),
        }))
    }
}

impl ConnectorError {
    pub fn from_kind(kind: ErrorKind) -> Self {
        ConnectorError { kind }
    }

    pub fn new_argument_count_mismatch_error(
        native_type: &str,
        required_count: usize,
        given_count: usize,
    ) -> ConnectorError {
        ConnectorError::from_kind(ErrorKind::ArgumentCountMisMatchError {
            native_type: String::from(native_type),
            required_count,
            given_count,
        })
    }

    pub fn new_value_parser_error(expected_type: &str, parser_error: &str, raw: &str) -> ConnectorError {
        ConnectorError::from_kind(ErrorKind::ValueParserError {
            expected_type: String::from(expected_type),
            parser_error: String::from(parser_error),
            raw: String::from(raw),
        })
    }
}

#[derive(Debug, Error, Clone)]
pub enum ErrorKind {
    #[error("Native types are not supported with {} connector", connector_name)]
    ConnectorNotSupportedForNativeTypes { connector_name: String },

    #[error(
        "The prefix {} is invalid. It must be equal to the name of an existing datasource e.g. {}. Did you mean to use {}?",
        given_prefix,
        expected_prefix,
        suggestion
    )]
    InvalidPrefixForNativeTypes {
        given_prefix: String,
        expected_prefix: String,
        suggestion: String,
    },

    #[error(
        "Native type {} is not compatible with declared field type {}, expected field type {}.",
        native_type,
        field_type,
        expected_types
    )]
    IncompatibleNativeType {
        native_type: String,
        field_type: String,
        expected_types: String,
    },

    #[error("Attribute @{} is defined twice.", attribute_name)]
    DuplicateAttributeError { attribute_name: String },

    #[error("Native type {} is not supported for {} connector.", native_type, connector_name)]
    NativeTypeNameUnknown {
        native_type: String,
        connector_name: String,
    },

    #[error(
        "Native type {} takes {} arguments, but received {}.",
        native_type,
        required_count,
        given_count
    )]
    ArgumentCountMisMatchError {
        native_type: String,
        required_count: usize,
        given_count: usize,
    },

    #[error(
        "Native type {} takes {} optional arguments, but received {}.",
        native_type,
        optional_count,
        given_count
    )]
    OptionalArgumentCountMismatchError {
        native_type: String,
        optional_count: usize,
        given_count: usize,
    },

    #[error(
    "Native types can only be used if the corresponding feature flag is enabled. Please add this field in your generator block: `previewFeatures = [\"nativeTypes\"]`"
    )]
    NativeFlagsPreviewFeatureDisabled,

    #[error("Native type {} can not be unique in {}.", native_type, connector_name)]
    IncompatibleNativeTypeWithUniqueAttribute {
        native_type: String,
        connector_name: String,
    },

    #[error(
        "Native type {} of {} can not be used on a field that is `@id` or `@@id`.",
        native_type,
        connector_name
    )]
    IncompatibleNativeTypeWithIdAttribute {
        native_type: String,
        connector_name: String,
    },

    #[error(
        "You can not define an index on fields with Native type {} of {}.",
        native_type,
        connector_name
    )]
    IncompatibleNativeTypeWithIndexAttribute {
        native_type: String,
        connector_name: String,
    },

    #[error(
        "Expected a {} value, but failed while parsing \"{}\": {}.",
        expected_type,
        raw,
        parser_error
    )]
    ValueParserError {
        expected_type: String,
        parser_error: String,
        raw: String,
    },

    #[error(
        "The scale must not be larger than the precision for the {} native type in {}.",
        native_type,
        connector_name
    )]
    ScaleLargerThanPrecisionError {
        native_type: String,
        connector_name: String,
    },

    #[error(
        "Sequential native type {} of {} must not have a static default value.",
        native_type,
        connector_name
    )]
    IncompatibleSequentialTypeWithStaticDefaultValue {
        native_type: String,
        connector_name: String,
    },

    #[error(
        "Argument M is out of range for Native type {} of {}: {}",
        native_type,
        connector_name,
        message
    )]
    ArgumentOutOfRangeError {
        native_type: String,
        connector_name: String,
        message: String,
    },
}
