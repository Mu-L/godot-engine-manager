use super::error::ExecutionError;
use crate::core::converter::PathConverter;
use crate::core::state::{
    ApplicationState, StandardCommand, StandardEngineId, ListScope, InstallOptions, ExecutionResult,
};
use crate::func::config::Config;

/// 命令执行器
pub struct CommandExecutor {
    state: ApplicationState,
    path_converter: PathConverter,
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
            path_converter,
        })
    }

    /// 加载已安装的引擎
    fn load_installed_engines(_config: &Config) -> Result<Vec<StandardEngineId>, ExecutionError> {
        let engines = Vec::new();
        // 这里暂时返回空，实际需要从目录读取
        // 我们会在后续完善这个功能
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
        _scope: ListScope,
        _filter: Option<StandardEngineId>,
    ) -> Result<ExecutionResult, ExecutionError> {
        // 这里暂时返回空列表，实际需要调用 list 模块的功能
        Ok(ExecutionResult::EngineList(vec![]))
    }

    /// 执行安装命令
    async fn execute_install(
        &mut self,
        engine: StandardEngineId,
        _options: InstallOptions,
    ) -> Result<ExecutionResult, ExecutionError> {
        // 这里暂时直接返回成功，实际需要调用 install 模块的功能
        self.state.installed_engines.push(engine.clone());
        Ok(ExecutionResult::Installed(engine))
    }

    /// 执行切换命令
    async fn execute_switch(
        &mut self,
        engine: StandardEngineId,
    ) -> Result<ExecutionResult, ExecutionError> {
        if !self.is_engine_installed(&engine) {
            return Err(ExecutionError::NotInstalled(engine.to_string()));
        }
        self.state.current_engine = Some(engine.clone());
        Ok(ExecutionResult::Switched(engine))
    }

    /// 执行移除命令
    async fn execute_remove(
        &mut self,
        engine: StandardEngineId,
    ) -> Result<ExecutionResult, ExecutionError> {
        if !self.is_engine_installed(&engine) {
            return Err(ExecutionError::NotInstalled(engine.to_string()));
        }
        self.state
            .installed_engines
            .retain(|e| e.to_string() != engine.to_string());
        Ok(ExecutionResult::Removed(engine))
    }

    /// 执行配置命令
    async fn execute_config(
        &mut self,
        _proxy: Option<String>,
    ) -> Result<ExecutionResult, ExecutionError> {
        // 这里暂时直接返回成功，实际需要调用 config 模块的功能
        Ok(ExecutionResult::ConfigUpdated)
    }

    /// 执行同步命令
    async fn execute_sync(&self) -> Result<ExecutionResult, ExecutionError> {
        // 这里暂时直接返回成功，实际需要调用 sync 模块的功能
        Ok(ExecutionResult::Synced)
    }
}
