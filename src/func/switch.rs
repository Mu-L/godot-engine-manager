use super::config::Config;
use crate::core::config::ConfigTrait;
use crate::core::utils::symlink;
use crate::func::tool::{format_engine_name, get_levels_dir};
use std::error::Error;

pub fn switch_engine(engine: &str, cfg: &mut Config) -> Result<String, Box<dyn Error>> {
    let link_path = cfg.root.join("default");
    let home_dir = get_levels_dir(&cfg.home, engine);
    // filename 去除zip和exe
    let engine = format_engine_name(engine);
    let engine_path = home_dir.join(&engine);

    // 对 engine_path 中所有exe文件创建软链接到default文件夹
    // 例如：Godot_v4.4.1-stable_mono_win64.exe 软链接到 default/Godot.exe
    // 如果是 Godot_v4.5-stable_win64_console.exe 软链接到 default/Godot_console.exe

    for entry in engine_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().unwrap_or_default() == "exe" {
            let filename = if path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with("_console.exe")
            {
                "godot_console.exe"
            } else {
                "godot.exe"
            };
            let link = link_path.join(filename);
            if let Err(e) = symlink(&path, &link) {
                eprintln!("Create link failed: {}", e);
                return Err(e.into());
            }
        }
    }

    cfg.switch_version(&engine);
    cfg.save();
    Ok(engine)
}
