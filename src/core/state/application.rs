use super::engine_id::StandardEngineId;
use crate::func::config::Config;

/// 应用全局状态
#[derive(Debug, Clone)]
pub struct ApplicationState {
    /// 配置
    pub config: Config,
    /// 当前激活的引擎
    pub current_engine: Option<StandardEngineId>,
    /// 已安装的引擎列表
    pub installed_engines: Vec<StandardEngineId>,
}
