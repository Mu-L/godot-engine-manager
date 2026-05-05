use std::error::Error;
use std::fmt;

/// 执行错误
#[derive(Debug)]
pub enum ExecutionError {
    AlreadyInstalled(String),
    NotInstalled(String),
    DownloadFailed(String),
    ChecksumFailed,
    ExtractionFailed(String),
    IoError(std::io::Error),
    Other(String),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExecutionError::AlreadyInstalled(s) => write!(f, "引擎已安装: {}", s),
            ExecutionError::NotInstalled(s) => write!(f, "引擎未安装: {}", s),
            ExecutionError::DownloadFailed(s) => write!(f, "下载失败: {}", s),
            ExecutionError::ChecksumFailed => write!(f, "校验失败"),
            ExecutionError::ExtractionFailed(s) => write!(f, "解压失败: {}", s),
            ExecutionError::IoError(e) => write!(f, "IO错误: {}", e),
            ExecutionError::Other(s) => write!(f, "错误: {}", s),
        }
    }
}

impl Error for ExecutionError {}

impl From<std::io::Error> for ExecutionError {
    fn from(e: std::io::Error) -> Self {
        ExecutionError::IoError(e)
    }
}
