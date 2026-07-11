use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    IOError(std::io::Error),
    CodecError(rule_codec::CodecError),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> ConfigError {
        ConfigError::IOError(err)
    }
}

impl From<rule_codec::CodecError> for ConfigError {
    fn from(err: rule_codec::CodecError) -> ConfigError {
        ConfigError::CodecError(err)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::IOError(err) => write!(f, "IO error: {}", err),
            ConfigError::CodecError(err) => write!(f, "Codec error: {}", err),
        }
    }
}

impl std::error::Error for ConfigError {}