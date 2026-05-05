use super::engine_id::StandardEngineId;

/// 列表范围
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListScope {
    Local,
    Remote,
    Assets,
}

/// 安装选项
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    pub force: bool,
    pub skip_checksum: bool,
    pub self_contained: bool,
}

/// 标准命令状态
#[derive(Debug, Clone)]
pub enum StandardCommand {
    /// 列出引擎
    List {
        scope: ListScope,
        filter: Option<StandardEngineId>,
    },
    /// 安装引擎
    Install {
        engine: StandardEngineId,
        options: InstallOptions,
    },
    /// 切换引擎
    Switch {
        engine: StandardEngineId,
    },
    /// 移除引擎
    Remove {
        engine: StandardEngineId,
    },
    /// 配置管理
    Config {
        proxy: Option<String>,
    },
    /// 数据同步
    Sync,
}

/// 执行结果
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// 引擎列表
    EngineList(Vec<StandardEngineId>),
    /// 安装成功
    Installed(StandardEngineId),
    /// 切换成功
    Switched(StandardEngineId),
    /// 移除成功
    Removed(StandardEngineId),
    /// 配置更新
    ConfigUpdated,
    /// 同步成功
    Synced,
}

impl ExecutionResult {
    /// 显示结果
    pub fn display(&self) {
        match self {
            ExecutionResult::EngineList(engines) => {
                for engine in engines {
                    println!("{}", engine);
                }
            }
            ExecutionResult::Installed(engine) => {
                println!("安装成功: {}", engine);
            }
            ExecutionResult::Switched(engine) => {
                println!("切换成功: {}", engine);
            }
            ExecutionResult::Removed(engine) => {
                println!("移除成功: {}", engine);
            }
            ExecutionResult::ConfigUpdated => {
                println!("配置更新成功");
            }
            ExecutionResult::Synced => {
                println!("数据同步成功");
            }
        }
    }
}
