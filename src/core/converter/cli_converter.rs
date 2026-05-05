use super::traits::{InputConverter, ConversionError};
use crate::cli::Commands;
use crate::core::state::{StandardCommand, StandardEngineId, ListScope, InstallOptions};

/// CLI 命令到标准命令转换器
pub struct CliToStandardCommandConverter;

impl InputConverter<Commands> for CliToStandardCommandConverter {
    type Output = StandardCommand;
    type Error = ConversionError;

    fn convert(&self, cli: Commands) -> Result<Self::Output, Self::Error> {
        match cli {
            Commands::Config { proxy } => Ok(StandardCommand::Config { proxy }),
            Commands::Sync => Ok(StandardCommand::Sync),
            Commands::List { remote, version } => {
                let scope = if remote {
                    ListScope::Remote
                } else if version.is_some() {
                    ListScope::Assets
                } else {
                    ListScope::Local
                };
                let filter = version
                    .map(|v| StandardEngineId::parse(&v))
                    .transpose()
                    .map_err(|e| ConversionError::ParseError(e.to_string()))?;
                Ok(StandardCommand::List { scope, filter })
            }
            Commands::Install {
                engine,
                force,
                skip_check,
            } => {
                let engine_id = StandardEngineId::parse(&engine)
                    .map_err(|e| ConversionError::ParseError(e.to_string()))?;
                let options = InstallOptions {
                    force,
                    skip_checksum: skip_check,
                    ..Default::default()
                };
                Ok(StandardCommand::Install {
                    engine: engine_id,
                    options,
                })
            }
            Commands::Switch { engine } => {
                let engine_id = StandardEngineId::parse(&engine)
                    .map_err(|e| ConversionError::ParseError(e.to_string()))?;
                Ok(StandardCommand::Switch { engine: engine_id })
            }
            Commands::Remove { engine } => {
                let engine_id = StandardEngineId::parse(&engine)
                    .map_err(|e| ConversionError::ParseError(e.to_string()))?;
                Ok(StandardCommand::Remove { engine: engine_id })
            }
        }
    }
}
