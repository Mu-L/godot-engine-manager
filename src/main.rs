use clap::Parser;
use gdem::cli::Cli;
use gdem::core::config::ConfigTrait;
use gdem::core::converter::CliToStandardCommandConverter;
use gdem::core::converter::InputConverter;
use gdem::core::engine::CommandExecutor;
use gdem::func::config::Config;

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
