use crate::core::state::{StandardEngineId, FileExtension};
use crate::func::config::Config;
use std::path::PathBuf;

/// 路径转换器
pub struct PathConverter {
    config: Config,
}

impl PathConverter {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 获取引擎安装目录
    pub fn get_engine_install_dir(&self, engine: &StandardEngineId) -> PathBuf {
        let levels = engine.get_storage_levels();
        let mut path = self.config.home.clone();
        for level in levels {
            path = path.join(level);
        }
        path.join(engine.to_file_name(FileExtension::None))
    }

    /// 获取缓存目录
    pub fn get_cache_dir(&self, engine: &StandardEngineId) -> PathBuf {
        let levels = engine.get_storage_levels();
        let mut path = self.config.cache.clone();
        for level in levels {
            path = path.join(level);
        }
        path
    }

    /// 获取缓存文件路径
    pub fn get_cache_file_path(&self, engine: &StandardEngineId) -> PathBuf {
        let cache_dir = self.get_cache_dir(engine);
        cache_dir.join(engine.to_file_name(FileExtension::Zip))
    }
}
