# gdem 项目优化方案 - 基于状态转换思想

## 1. 优化目标

通过引入**标准状态转换思想**，简化项目架构，提高代码可维护性和扩展性：

- 定义统一的**标准状态**作为处理基准
- 所有复杂输入都通过**转换器**转换为标准状态
- 核心业务逻辑只处理标准状态
- 统一的错误处理和状态管理

## 2. 当前架构问题分析

### 2.1 输入格式多样性

- 用户输入格式不统一：
  - 完整文件名：`Godot_v4.4.1-stable_win64.exe.zip`
  - 版本号：`4.4-stable`、`4.4.1`
  - 主版本号：`3`、`4`
- 缺少统一的输入标准化层

### 2.2 状态分散

- 配置状态（Config）
- 命令状态（Commands）
- 安装过程状态（分散在各函数中）
- 没有统一的状态机管理

### 2.3 业务逻辑耦合

- 各功能模块（install、list、switch、remove）逻辑独立
- 缺少统一的执行流程
- 重复代码较多（如路径处理、版本解析）

## 3. 优化方案设计

### 3.1 核心概念

```
输入 → [转换器] → 标准状态 → [核心引擎] → 输出
           ↑
      多种输入格式
```

### 3.2 标准状态定义

#### 3.2.1 统一引擎标识（StandardEngineId）

```rust
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

impl StandardEngineId {
    /// 从各种输入格式解析为标准状态
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        // 尝试多种解析策略
        Self::from_full_name(input)
            .or_else(|| Self::from_version_tag(input))
            .or_else(|| Self::from_short_version(input))
            .ok_or_else(|| ParseError::InvalidFormat(input.to_string()))
    }
    
    /// 转换为原始文件名
    pub fn to_file_name(&self, extension: FileExtension) -> String {
        // 根据标准状态构建文件名
    }
    
    /// 获取存储路径层级
    pub fn get_storage_levels(&self) -> Vec<String> {
        vec![
            format!("{}.x", self.major),
            self.get_version_string(),
        ]
    }
}
```

#### 3.2.2 统一命令状态（StandardCommandState）

```rust
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
```

#### 3.2.3 应用状态（ApplicationState）

```rust
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
```

### 3.3 转换器层设计

#### 3.3.1 输入转换器

```rust
/// 输入转换器 trait
pub trait InputConverter<T> {
    type Output;
    type Error;
    
    fn convert(&self, input: T) -> Result<Self::Output, Self::Error>;
}

/// 字符串到标准引擎标识转换器
pub struct StringToEngineIdConverter;

impl InputConverter<&str> for StringToEngineIdConverter {
    type Output = StandardEngineId;
    type Error = ParseError;
    
    fn convert(&self, input: &str) -> Result<Self::Output, Self::Error> {
        StandardEngineId::parse(input)
    }
}

/// CLI 命令到标准命令转换器
pub struct CliToStandardCommandConverter;

impl InputConverter<Commands> for CliToStandardCommandConverter {
    type Output = StandardCommand;
    type Error = ConversionError;
    
    fn convert(&self, cli: Commands) -> Result<Self::Output, Self::Error> {
        match cli {
            Commands::List { remote, version } => {
                let scope = if remote {
                    ListScope::Remote
                } else if version.is_some() {
                    ListScope::Assets
                } else {
                    ListScope::Local
                };
                let filter = version.map(|v| StandardEngineId::parse(&v)).transpose()?;
                Ok(StandardCommand::List { scope, filter })
            }
            Commands::Install { engine, force, skip_check, .. } => {
                let engine_id = StandardEngineId::parse(&engine)?;
                let options = InstallOptions {
                    force,
                    skip_checksum: skip_check,
                    ..Default::default()
                };
                Ok(StandardCommand::Install { engine: engine_id, options })
            }
            // ... 其他命令转换
        }
    }
}
```

#### 3.3.2 路径转换器

```rust
/// 路径转换器
pub struct PathConverter {
    config: Config,
}

impl PathConverter {
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
}
```

### 3.4 核心引擎层设计

#### 3.4.1 命令执行器

```rust
/// 命令执行器
pub struct CommandExecutor {
    state: ApplicationState,
    path_converter: PathConverter,
}

impl CommandExecutor {
    pub fn new(config: Config) -> Result<Self, ExecutionError> {
        let path_converter = PathConverter { config: config.clone() };
        let installed_engines = Self::load_installed_engines(&config)?;
        let current_engine = config.version.parse().ok().and_then(|v| {
            installed_engines.iter().find(|e| e.get_version_string() == v).cloned()
        });
        
        Ok(Self {
            state: ApplicationState {
                config,
                current_engine,
                installed_engines,
            },
            path_converter,
        })
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
    async fn execute_list(&self, scope: ListScope, filter: Option<StandardEngineId>) -> Result<ExecutionResult, ExecutionError> {
        let engines = match scope {
            ListScope::Local => {
                self.state.installed_engines.clone()
            }
            ListScope::Remote => {
                self.load_remote_engines(filter.as_ref()).await?
            }
            ListScope::Assets => {
                self.load_remote_assets(filter.as_ref()).await?
            }
        };
        Ok(ExecutionResult::EngineList(engines))
    }
    
    /// 执行安装命令
    async fn execute_install(&mut self, engine: StandardEngineId, options: InstallOptions) -> Result<ExecutionResult, ExecutionError> {
        // 1. 检查是否已安装
        if self.is_engine_installed(&engine) && !options.force {
            return Err(ExecutionError::AlreadyInstalled(engine));
        }
        
        // 2. 下载
        let file_path = self.download_engine(&engine, &options).await?;
        
        // 3. 校验
        if !options.skip_checksum {
            self.verify_checksum(&engine, &file_path).await?;
        }
        
        // 4. 解压
        self.extract_engine(&engine, &file_path, &options).await?;
        
        // 5. 更新状态
        self.state.installed_engines.push(engine.clone());
        
        Ok(ExecutionResult::Installed(engine))
    }
}
```

### 3.5 新的目录结构

```
src/
├── main.rs                          # 入口 - 精简为 CLI 解析 + 转换 + 执行
├── lib.rs
├── core/
│   ├── mod.rs
│   ├── state/                       # 新增：状态定义
│   │   ├── mod.rs
│   │   ├── engine_id.rs             # StandardEngineId
│   │   ├── command.rs               # StandardCommand
│   │   └── application.rs           # ApplicationState
│   ├── converter/                   # 新增：转换器层
│   │   ├── mod.rs
│   │   ├── input_converter.rs       # 输入转换
│   │   ├── path_converter.rs        # 路径转换
│   │   └── cli_converter.rs         # CLI 转换
│   ├── engine/                      # 新增：核心引擎
│   │   ├── mod.rs
│   │   ├── executor.rs              # CommandExecutor
│   │   ├── installer.rs             # 安装引擎
│   │   ├── lister.rs                # 列表引擎
│   │   └── synchronizer.rs          # 数据同步
│   ├── handler.rs                   # 保留：DocumentHandler
│   ├── config.rs                    # 简化：Config
│   ├── tags.rs
│   ├── utils.rs
│   └── style.rs
└── func/                            # 简化：兼容层或删除
    ├── mod.rs
    └── ... (逐步迁移到 core/engine)
```

### 3.6 主函数重构

```rust
#[tokio::main]
async fn main() {
    // 1. 解析 CLI
    let cli = Cli::parse();
    
    // 2. 转换为标准命令
    let converter = CliToStandardCommandConverter;
    let command = match converter.convert(cli.command) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("转换错误: {}", e);
            std::process::exit(1);
        }
    };
    
    // 3. 初始化执行器
    let config = Config::init();
    let mut executor = match CommandExecutor::new(config) {
        Ok(exec) => exec,
        Err(e) => {
            eprintln!("初始化错误: {}", e);
            std::process::exit(1);
        }
    };
    
    // 4. 执行命令（核心逻辑只处理标准状态）
    match executor.execute(command).await {
        Ok(result) => {
            result.display();
        }
        Err(e) => {
            eprintln!("执行错误: {}", e);
            std::process::exit(1);
        }
    }
}
```

## 4. 迁移策略

### 4.1 分阶段迁移

1. **阶段一：定义标准状态**
   - 创建 `core/state/` 模块
   - 定义 `StandardEngineId`、`StandardCommand`
   - 编写单元测试

2. **阶段二：实现转换器**
   - 创建 `core/converter/` 模块
   - 实现输入转换逻辑
   - 确保所有现有输入格式都能正确转换

3. **阶段三：实现核心引擎**
   - 创建 `core/engine/` 模块
   - 逐步将 `func/` 中的逻辑迁移过来
   - 保持向后兼容

4. **阶段四：重构主函数**
   - 更新 `main.rs` 使用新架构
   - 集成测试
   - 移除旧代码

### 4.2 向后兼容

- 在迁移期间保留现有 API
- 使用新架构实现，旧接口作为兼容层
- 完全迁移后再删除旧代码

## 5. 优化收益

### 5.1 可维护性提升

- 核心逻辑只处理标准状态，更简单清晰
- 输入格式变化只需要修改转换器
- 新增输入格式只需添加新的转换策略

### 5.2 可扩展性提升

- 新增命令只需：
  1. 在 `StandardCommand` 中添加变体
  2. 实现转换逻辑
  3. 实现执行逻辑
- 新增支持的平台只需在转换器中添加识别规则

### 5.3 可测试性提升

- 标准状态可以独立测试
- 转换器可以独立测试
- 执行器可以用标准状态进行单元测试

## 6. 总结

通过引入**标准状态转换思想**，我们将：

1. **简化复杂性**：所有输入转换为统一的标准状态
2. **解耦业务逻辑**：核心引擎只处理标准状态
3. **提高可维护性**：职责清晰，修改影响范围小
4. **增强可扩展性**：新增功能更容易实现
5. **改善测试性**：各层可以独立测试

这种架构设计遵循了**开闭原则**（对扩展开放，对修改关闭）和**单一职责原则**，为项目的长期发展奠定了良好基础。
