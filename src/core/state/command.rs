use super::engine_id::StandardEngineId;
use crate::func::config::Config;

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
    EngineList {
        engines: Vec<StandardEngineId>,
        scope: ListScope,
        current: Option<StandardEngineId>,
        config: Config,
    },
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
            ExecutionResult::EngineList {
                engines,
                scope,
                current,
                config: _config,
            } => {
                let engine_strings: Vec<String> = engines.iter().map(|e| e.to_string()).collect();
                let current_str = current.as_ref().map(|e| e.to_string()).unwrap_or_default();

                match scope {
                    ListScope::Local => {
                        let table = crate::core::style::show_tree(
                            &engine_strings,
                            &current_str,
                            "本地引擎",
                        );
                        println!("{}", table);
                    }
                    ListScope::Remote => {
                        let table = crate::core::style::show_list(&engine_strings, "远程引擎");
                        println!("{}", table);
                    }
                    ListScope::Assets => {
                        let table = crate::core::style::show_list(&engine_strings, "引擎资产");
                        println!("{}", table);
                    }
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
