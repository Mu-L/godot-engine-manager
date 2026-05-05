use super::error::ExecutionError;
use crate::core::config::ConfigTrait;
use crate::core::converter::PathConverter;
use crate::core::state::{
    ApplicationState, StandardCommand, StandardEngineId, ListScope, InstallOptions, ExecutionResult,
};
use crate::func::config::{self, Config};
use crate::func::{install, list, remove, switch, sync};

/// 命令执行器
pub struct CommandExecutor {
    state: ApplicationState,
    _path_converter: PathConverter,
}

impl CommandExecutor {
    pub fn new(config: Config) -> Result<Self, ExecutionError> {
        let path_converter = PathConverter::new(config.clone());
        let installed_engines = Self::load_installed_engines(&config)?;
        let current_engine = if !config.version.is_empty() {
            installed_engines
                .iter()
                .find(|e| e.to_string() == config.version)
                .cloned()
        } else {
            None
        };

        Ok(Self {
            state: ApplicationState {
                config,
                current_engine,
                installed_engines,
            },
            _path_converter: path_converter,
        })
    }

    /// 加载已安装的引擎
    fn load_installed_engines(config: &Config) -> Result<Vec<StandardEngineId>, ExecutionError> {
        let engine_strings = list::list_local_engines(&config.home)
            .map_err(|e| ExecutionError::Other(format!("无法加载本地引擎: {}", e)))?;

        let mut engines = Vec::new();
        for s in engine_strings {
            if let Ok(engine) = StandardEngineId::parse(&s) {
                engines.push(engine);
            }
        }

        Ok(engines)
    }

    /// 检查引擎是否已安装
    fn is_engine_installed(&self, engine: &StandardEngineId) -> bool {
        self.state
            .installed_engines
            .iter()
            .any(|e| e.to_string() == engine.to_string())
    }

    /// 执行标准命令
    pub async fn execute(&mut self, command: StandardCommand) -> Result<ExecutionResult, ExecutionError> {
        match command {
            StandardCommand::List { scope, filter } => {
                self.execute_list(scope, filter).await
            }
            StandardCommand::Install { engine, options } => {
                self.execute_install(engine, options).await
            }
            StandardCommand::Switch { engine } => {
                self.execute_switch(engine).await
            }
            StandardCommand::Remove { engine } => {
                self.execute_remove(engine).await
            }
            StandardCommand::Config { proxy } => {
                self.execute_config(proxy).await
            }
            StandardCommand::Sync => {
                self.execute_sync().await
            }
        }
    }

    /// 执行列表命令（核心逻辑只处理标准状态）
    async fn execute_list(
        &self,
        scope: ListScope,
        filter: Option<StandardEngineId>,
    ) -> Result<ExecutionResult, ExecutionError> {
        let engines = match scope {
            ListScope::Local => {
                // 列出本地引擎
                self.state.installed_engines.clone()
            }
            ListScope::Remote => {
                // 列出远程引擎
                let versions = list::list_remote_engines(&self.state.config.data)
                    .map_err(|e| ExecutionError::Other(format!("无法加载远程引擎: {}", e)))?;
                versions
                    .into_iter()
                    .filter_map(|v| StandardEngineId::parse(&v).ok())
                    .collect()
            }
            ListScope::Assets => {
                // 列出指定版本的资产
                if let Some(f) = filter {
                    let assets = list::list_remote_engine_assets(&self.state.config.data, &f.get_version_string())
                        .map_err(|e| ExecutionError::Other(format!("无法加载引擎资产: {}", e)))?;
                    assets
                        .into_iter()
                        .filter_map(|a| StandardEngineId::parse(&a).ok())
                        .collect()
                } else {
                    Vec::new()
                }
            }
        };

        Ok(ExecutionResult::EngineList {
            engines,
            scope,
            current: self.state.current_engine.clone(),
            config: self.state.config.clone(),
        })
    }

    /// 执行安装命令
    async fn execute_install(
        &mut self,
        engine: StandardEngineId,
        options: InstallOptions,
    ) -> Result<ExecutionResult, ExecutionError> {
        let _engine_str = engine.to_string();
        let original_file_name = engine.to_file_name(crate::core::state::FileExtension::Zip);

        let result = install::full_install_process(
            &original_file_name,
            &self.state.config,
            options.force,
            options.skip_checksum,
        )
        .await;

        match result {
            Ok(_) => {
                // 安装成功，重新加载引擎列表
                self.state.installed_engines = Self::load_installed_engines(&self.state.config)?;
                Ok(ExecutionResult::Installed(engine))
            }
            Err(e) => Err(ExecutionError::Other(format!("安装失败: {}", e))),
        }
    }

    /// 执行切换命令
    async fn execute_switch(
        &mut self,
        engine: StandardEngineId,
    ) -> Result<ExecutionResult, ExecutionError> {
        let engine_str = engine.to_string();

        let result = switch::switch_engine(&engine_str, &mut self.state.config);

        match result {
            Ok(_) => {
                self.state.current_engine = Some(engine.clone());
                self.state.config.save();
                Ok(ExecutionResult::Switched(engine))
            }
            Err(e) => Err(ExecutionError::Other(format!("切换失败: {}", e))),
        }
    }

    /// 执行移除命令
    async fn execute_remove(
        &mut self,
        engine: StandardEngineId,
    ) -> Result<ExecutionResult, ExecutionError> {
        let engine_str = engine.to_string();

        let result = remove::remove_engine(&engine_str, &mut self.state.config);

        match result {
            Ok(_) => {
                // 移除成功，重新加载引擎列表
                self.state.installed_engines = Self::load_installed_engines(&self.state.config)?;
                if let Some(current) = &self.state.current_engine {
                    if current.to_string() == engine_str {
                        self.state.current_engine = None;
                    }
                }
                Ok(ExecutionResult::Removed(engine))
            }
            Err(e) => Err(ExecutionError::Other(format!("移除失败: {}", e))),
        }
    }

    /// 执行配置命令
    async fn execute_config(
        &mut self,
        proxy: Option<String>,
    ) -> Result<ExecutionResult, ExecutionError> {
        if let Some(p) = proxy {
            self.state.config.proxy = p;
        }
        config::link_appdata(&self.state.config.data);
        self.state.config.save();

        Ok(ExecutionResult::ConfigUpdated)
    }

    /// 执行同步命令
    async fn execute_sync(&self) -> Result<ExecutionResult, ExecutionError> {
        sync::sync_data(&self.state.config).await;

        Ok(ExecutionResult::Synced)
    }
}
