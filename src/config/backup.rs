use anyhow::{Context, Result};
use fs_extra::dir::{copy, CopyOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;

pub fn backup_game_config(config: &Config) -> Result<()> {
    config.ensure_backup_dir()?;
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let backup_path = config.backup_dir.join(format!("game_config_{}", timestamp));
    
    if !Path::new(&config.game_config_path).exists() {
        return Err(anyhow::anyhow!(
            "游戏配置路径不存在: {:?}",
            config.game_config_path
        ));
    }
    
    if Path::new(&config.game_config_path).is_dir() {
        // 复制目录
        let options = CopyOptions::new();
        copy(&config.game_config_path, &backup_path, &options)
            .with_context(|| {
                format!(
                    "从 {:?} 备份游戏配置到 {:?} 失败",
                    config.game_config_path, backup_path
                )
            })?;
    } else {
        // 复制文件
        fs::copy(&config.game_config_path, &backup_path).with_context(|| {
            format!(
                "从 {:?} 备份游戏配置到 {:?} 失败",
                config.game_config_path, backup_path
            )
        })?;
    }
    
    println!("游戏配置已成功备份到 {:?}", backup_path);
    
    Ok(())
}

pub fn restore_game_config(config: &Config) -> Result<()> {
    let backups = list_game_config_backups(config)?;
    
    if backups.is_empty() {
        println!("未找到游戏配置备份。");
        return Ok(());
    }
    
    // 显示带时间戳的备份
    println!("可用的游戏配置备份:");
    for (i, backup) in backups.iter().enumerate() {
        println!("  {}. {:?}", i + 1, backup);
    }
    
    // 让用户选择一个备份
    print!("输入要恢复的备份编号 (或输入0取消): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let selection: usize = match input.trim().parse() {
        Ok(num) if num > 0 && num <= backups.len() => num - 1,
        Ok(0) => return Ok(()),
        _ => {
            println!("无效的选择。恢复已取消。");
            return Ok(());
        }
    };
    
    let selected_backup = &backups[selection];
    
    // 确保目标目录存在
    if let Some(parent) = config.game_config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建父目录失败: {:?}", parent))?;
    }
    
    // 如果存在，则删除现有配置
    if Path::new(&config.game_config_path).exists() {
        if Path::new(&config.game_config_path).is_dir() {
            fs::remove_dir_all(&config.game_config_path).with_context(|| {
                format!("删除现有游戏配置目录失败: {:?}", config.game_config_path)
            })?;
        } else {
            fs::remove_file(&config.game_config_path).with_context(|| {
                format!("删除现有游戏配置文件失败: {:?}", config.game_config_path)
            })?;
        }
    }
    
    // 从备份恢复
    if Path::new(selected_backup).is_dir() {
        let options = CopyOptions::new();
        copy(selected_backup, config.game_config_path.parent().unwrap(), &options)
            .with_context(|| {
                format!(
                    "从 {:?} 恢复游戏配置到 {:?} 失败",
                    selected_backup, config.game_config_path
                )
            })?;
    } else {
        fs::copy(selected_backup, &config.game_config_path).with_context(|| {
            format!(
                "从 {:?} 恢复游戏配置到 {:?} 失败",
                selected_backup, config.game_config_path
            )
        })?;
    }
    
    println!("游戏配置已成功从 {:?} 恢复", selected_backup);
    
    Ok(())
}

fn list_game_config_backups(config: &Config) -> Result<Vec<PathBuf>> {
    if !config.backup_dir.exists() {
        return Ok(Vec::new());
    }
    
    let entries = fs::read_dir(&config.backup_dir)
        .with_context(|| format!("读取备份目录失败: {:?}", config.backup_dir))?;
    
    let mut backups = Vec::new();
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        
        if file_name.starts_with("game_config_") {
            backups.push(path);
        }
    }
    
    // 按修改时间排序（最新的在前）
    backups.sort_by(|a, b| {
        let a_meta = fs::metadata(a).unwrap();
        let b_meta = fs::metadata(b).unwrap();
        b_meta.modified().unwrap().cmp(&a_meta.modified().unwrap())
    });
    
    Ok(backups)
} 