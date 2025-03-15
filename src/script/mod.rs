use anyhow::{Context, Result};
use fs_extra::dir::{copy, CopyOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;

pub fn add_script_path(config: &mut Config) -> Result<()> {
    print!("请输入脚本文件或目录的路径: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let path = PathBuf::from(input.trim());
    
    if !path.exists() {
        println!("路径不存在: {:?}", path);
        return Ok(());
    }
    
    config.script_paths.push(path.clone());
    config.save()?;
    
    println!("已添加脚本路径: {:?}", path);
    
    Ok(())
}

pub fn remove_script_path(config: &mut Config) -> Result<()> {
    if config.script_paths.is_empty() {
        println!("没有配置脚本路径。");
        return Ok(());
    }
    
    println!("脚本路径:");
    for (i, path) in config.script_paths.iter().enumerate() {
        println!("  {}. {:?}", i + 1, path);
    }
    
    print!("输入要删除的编号 (或输入0取消): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let selection: usize = match input.trim().parse() {
        Ok(num) if num > 0 && num <= config.script_paths.len() => num - 1,
        Ok(0) => return Ok(()),
        _ => {
            println!("无效的选择。删除已取消。");
            return Ok(());
        }
    };
    
    let removed = config.script_paths.remove(selection);
    config.save()?;
    
    println!("已删除脚本路径: {:?}", removed);
    
    Ok(())
}

pub fn backup_scripts(config: &Config) -> Result<()> {
    config.ensure_backup_dir()?;
    
    if config.script_paths.is_empty() {
        println!("没有配置脚本路径。请先添加脚本路径。");
        return Ok(());
    }
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let backup_dir = config.backup_dir.join(format!("scripts_{}", timestamp));
    fs::create_dir_all(&backup_dir)
        .with_context(|| format!("创建备份目录失败: {:?}", backup_dir))?;
    
    for (i, path) in config.script_paths.iter().enumerate() {
        if !path.exists() {
            println!("警告: 脚本路径不存在: {:?}", path);
            continue;
        }
        
        let target_name = path.file_name().unwrap_or_else(|| Path::new("unknown").as_os_str());
        let target_path = backup_dir.join(format!("{}_{}", i, target_name.to_string_lossy()));
        
        if path.is_dir() {
            // 复制目录
            let options = CopyOptions::new();
            copy(path, &target_path, &options).with_context(|| {
                format!(
                    "从 {:?} 备份脚本目录到 {:?} 失败",
                    path, target_path
                )
            })?;
        } else {
            // 复制文件
            fs::copy(path, &target_path).with_context(|| {
                format!(
                    "从 {:?} 备份脚本文件到 {:?} 失败",
                    path, target_path
                )
            })?;
        }
        
        println!("已备份: {:?} -> {:?}", path, target_path);
    }
    
    println!("脚本已成功备份到 {:?}", backup_dir);
    
    Ok(())
}

pub fn restore_scripts(config: &Config) -> Result<()> {
    let backups = list_script_backups(config)?;
    
    if backups.is_empty() {
        println!("未找到脚本备份。");
        return Ok(());
    }
    
    // 显示带时间戳的备份
    println!("可用的脚本备份:");
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
    
    // 获取备份目录中的所有项目
    let entries = fs::read_dir(selected_backup)
        .with_context(|| format!("读取备份目录失败: {:?}", selected_backup))?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // 从文件名中提取索引（格式："{index}_{name}"）
        let file_name = path.file_name().unwrap().to_string_lossy();
        let parts: Vec<&str> = file_name.splitn(2, '_').collect();
        
        if parts.len() < 2 {
            println!("警告: 无效的备份文件名格式: {}", file_name);
            continue;
        }
        
        let index: usize = match parts[0].parse() {
            Ok(idx) => idx,
            Err(_) => {
                println!("警告: 备份文件名中的索引无效: {}", file_name);
                continue;
            }
        };
        
        if index >= config.script_paths.len() {
            println!("警告: 备份索引 {} 超出范围 (最大: {})", index, config.script_paths.len() - 1);
            continue;
        }
        
        let target_path = &config.script_paths[index];
        
        // 确保父目录存在
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("创建父目录失败: {:?}", parent))?;
        }
        
        // 如果存在，则删除现有脚本
        if target_path.exists() {
            if target_path.is_dir() {
                fs::remove_dir_all(target_path).with_context(|| {
                    format!("删除现有脚本目录失败: {:?}", target_path)
                })?;
            } else {
                fs::remove_file(target_path).with_context(|| {
                    format!("删除现有脚本文件失败: {:?}", target_path)
                })?;
            }
        }
        
        // 从备份恢复
        if path.is_dir() {
            let options = CopyOptions::new();
            copy(&path, target_path.parent().unwrap(), &options).with_context(|| {
                format!(
                    "从 {:?} 恢复脚本目录到 {:?} 失败",
                    path, target_path
                )
            })?;
        } else {
            fs::copy(&path, target_path).with_context(|| {
                format!(
                    "从 {:?} 恢复脚本文件到 {:?} 失败",
                    path, target_path
                )
            })?;
        }
        
        println!("已恢复: {:?} -> {:?}", path, target_path);
    }
    
    println!("脚本已成功从 {:?} 恢复", selected_backup);
    
    Ok(())
}

fn list_script_backups(config: &Config) -> Result<Vec<PathBuf>> {
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
        
        if file_name.starts_with("scripts_") && path.is_dir() {
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