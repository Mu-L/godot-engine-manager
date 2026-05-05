use std::error::Error;
use std::fmt;

/// 输入转换器 trait
pub trait InputConverter<T> {
    type Output;
    type Error;
    
    fn convert(&self, input: T) -> Result<Self::Output, Self::Error>;
}

/// 转换错误
#[derive(Debug)]
pub enum ConversionError {
    ParseError(String),
    InvalidCommand(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConversionError::ParseError(s) => write!(f, "解析错误: {}", s),
            ConversionError::InvalidCommand(s) => write!(f, "无效的命令: {}", s),
        }
    }
}

impl Error for ConversionError {}
