use regex::Regex;
use std::error::Error;
use std::fmt;

/// 标准引擎标识 - 所有引擎相关操作的核心状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardEngineId {
    /// 主版本号（如 4）
    pub major: u32,
    /// 次版本号（如 4）
    pub minor: u32,
    /// 补丁版本号（如 1）
    pub patch: Option<u32>,
    /// 版本类型（stable、beta、rc 等）
    pub release_type: String,
    /// 操作系统（win64、linux、macos 等）
    pub os: Option<String>,
    /// 架构（x86_64、arm64 等）
    pub arch: Option<String>,
    /// 是否包含 Mono 支持
    pub has_mono: bool,
    /// 是否为导出模板
    pub is_export_template: bool,
}

/// 解析错误
#[derive(Debug)]
pub enum ParseError {
    InvalidFormat(String),
    InvalidVersion(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidFormat(s) => write!(f, "无效的格式: {}", s),
            ParseError::InvalidVersion(s) => write!(f, "无效的版本号: {}", s),
        }
    }
}

impl Error for ParseError {}

/// 文件扩展名
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileExtension {
    Zip,
    Tpz,
    None,
}

impl StandardEngineId {
    /// 从各种输入格式解析为标准状态
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        // 尝试多种解析策略
        Self::from_full_name(input)
            .or_else(|| Self::from_version_tag(input))
            .or_else(|| Self::from_short_version(input))
            .ok_or_else(|| ParseError::InvalidFormat(input.to_string()))
    }

    /// 从完整文件名解析（如 "Godot_v4.4.1-stable_win64.exe.zip"）
    fn from_full_name(input: &str) -> Option<Self> {
        // 移除扩展名
        let name = input
            .replace(".zip", "")
            .replace(".tpz", "")
            .replace(".exe", "");

        // 检查是否是导出模板
        let is_export_template = name.contains("export_templates");

        // 解析版本号
        let re = Regex::new(r"v(\d+)\.(\d+)(?:\.(\d+))?(?:-([a-z]+))?").ok()?;
        let captures = re.captures(&name)?;

        let major = captures.get(1)?.as_str().parse().ok()?;
        let minor = captures.get(2)?.as_str().parse().ok()?;
        let patch = captures.get(3).and_then(|m| m.as_str().parse().ok());
        let release_type = captures
            .get(4)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "stable".to_string());

        // 检查 Mono
        let has_mono = name.contains("_mono_");

        // 解析 OS 和架构
        let (os, arch) = Self::parse_os_arch(&name);

        Some(Self {
            major,
            minor,
            patch,
            release_type,
            os,
            arch,
            has_mono,
            is_export_template,
        })
    }

    /// 从版本标签解析（如 "4.4.1-stable"）
    fn from_version_tag(input: &str) -> Option<Self> {
        let re = Regex::new(r"(\d+)\.(\d+)(?:\.(\d+))?(?:-([a-z]+))?").ok()?;
        let captures = re.captures(input)?;

        let major = captures.get(1)?.as_str().parse().ok()?;
        let minor = captures.get(2)?.as_str().parse().ok()?;
        let patch = captures.get(3).and_then(|m| m.as_str().parse().ok());
        let release_type = captures
            .get(4)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "stable".to_string());

        Some(Self {
            major,
            minor,
            patch,
            release_type,
            os: None,
            arch: None,
            has_mono: false,
            is_export_template: false,
        })
    }

    /// 从简短版本号解析（如 "4.4"、"3"）
    fn from_short_version(input: &str) -> Option<Self> {
        let re = Regex::new(r"(\d+)(?:\.(\d+))?").ok()?;
        let captures = re.captures(input)?;

        let major = captures.get(1)?.as_str().parse().ok()?;
        let minor = captures
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);

        Some(Self {
            major,
            minor,
            patch: None,
            release_type: "stable".to_string(),
            os: None,
            arch: None,
            has_mono: false,
            is_export_template: false,
        })
    }

    /// 解析操作系统和架构
    fn parse_os_arch(name: &str) -> (Option<String>, Option<String>) {
        let os = if name.contains("win64") {
            Some("win64".to_string())
        } else if name.contains("linux") {
            Some("linux".to_string())
        } else if name.contains("macos") || name.contains("osx") {
            Some("macos".to_string())
        } else {
            None
        };

        let arch = if name.contains("x86_64") || name.contains("win64") || name.contains("linux.x86_64") {
            Some("x86_64".to_string())
        } else if name.contains("arm64") || name.contains("aarch64") {
            Some("arm64".to_string())
        } else {
            None
        };

        (os, arch)
    }

    /// 转换为原始文件名
    pub fn to_file_name(&self, extension: FileExtension) -> String {
        let mut parts = vec!["Godot".to_string()];

        // 版本部分
        let version_part = if let Some(patch) = self.patch {
            format!("v{}.{}.{}-{}", self.major, self.minor, patch, self.release_type)
        } else {
            format!("v{}.{}-{}", self.major, self.minor, self.release_type)
        };
        parts.push(version_part);

        // Mono
        if self.has_mono {
            parts.push("mono".to_string());
        }

        // 导出模板
        if self.is_export_template {
            parts.push("export_templates".to_string());
        } else if let Some(os) = &self.os {
            parts.push(os.clone());
        }

        // 构建基础文件名
        let base_name = parts.join("_");

        // 添加扩展名
        match extension {
            FileExtension::Zip => format!("{}.exe.zip", base_name),
            FileExtension::Tpz => format!("{}.tpz", base_name),
            FileExtension::None => base_name,
        }
    }

    /// 获取版本字符串（如 "4.4.1-stable"）
    pub fn get_version_string(&self) -> String {
        if let Some(patch) = self.patch {
            format!("{}.{}.{}-{}", self.major, self.minor, patch, self.release_type)
        } else {
            format!("{}.{}-{}", self.major, self.minor, self.release_type)
        }
    }

    /// 获取存储路径层级
    pub fn get_storage_levels(&self) -> Vec<String> {
        vec![
            format!("{}.x", self.major),
            self.get_version_string(),
        ]
    }

    /// 获取主版本标识
    pub fn get_major_tag(&self) -> String {
        format!("{}.x", self.major)
    }
}

impl fmt::Display for StandardEngineId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_file_name(FileExtension::None))
    }
}
