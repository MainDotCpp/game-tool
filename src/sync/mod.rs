use anyhow::{Context, Result};
use fs_extra::dir::{copy, CopyOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{Config, SyncItem, SyncGroup};  // 导入SyncGroup

// 添加同步项目
pub fn add_sync_item(config: &mut Config) -> Result<()> {
    print!("请输入同步项目名称: ");
    let mut name = String::new();
    std::io::stdin().read_line(&mut name)?;
    let name = name.trim().to_string();
    
    print!("请输入源路径（文件或文件夹）: ");
    let mut source = String::new();
    std::io::stdin().read_line(&mut source)?;
    let source_path = PathBuf::from(source.trim());
    
    if !source_path.exists() {
        println!("源路径不存在: {:?}", source_path);
        return Ok(());
    }
    
    // 询问是否添加到组
    let mut group = None;
    if !config.sync_groups.is_empty() {
        println!("可用的同步组:");
        for (i, g) in config.sync_groups.iter().enumerate() {
            println!("  {}. {}", i + 1, g.name);
        }
        
        print!("请选择要添加到的组 (输入编号，或0表示不添加到任何组): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if let Ok(selection) = input.trim().parse::<usize>() {
            if selection > 0 && selection <= config.sync_groups.len() {
                group = Some(config.sync_groups[selection - 1].name.clone());
            }
        }
    }
    
    // 备份路径默认在备份目录下以项目名称命名
    let backup_path = config.backup_dir.join(&name);
    
    let item_name = name.clone(); // 克隆名称用于后面的输出
    
    let sync_item = SyncItem {
        name,
        source_path: source_path.clone(),
        backup_path: backup_path.clone(),
        is_enabled: true,
        group: group.clone(), // 使用克隆而不是移动
    };
    
    config.sync_items.push(sync_item);
    config.save()?;
    
    // 显示组信息
    let group_info = if let Some(g) = &group {
        format!("组: {}", g)
    } else {
        "无组".to_string()
    };
    
    println!("已添加同步项目: {} (源路径: {:?}, 备份路径: {:?}, {})", 
             item_name, source_path, backup_path, group_info);
    
    Ok(())
}

// 列出所有同步项目
pub fn list_sync_items(config: &Config) -> Result<()> {
    println!("同步项目列表:");
    if config.sync_items.is_empty() {
        println!("  没有配置同步项目。");
        return Ok(());
    }
    
    // 按组归类显示
    let mut by_group: std::collections::HashMap<Option<String>, Vec<&SyncItem>> = std::collections::HashMap::new();
    
    for item in &config.sync_items {
        by_group.entry(item.group.clone()).or_default().push(item);
    }
    
    // 先显示没有组的项目
    if let Some(items) = by_group.get(&None) {
        println!("无组项目:");
        for (i, item) in items.iter().enumerate() {
            let status = if item.is_enabled { "启用" } else { "禁用" };
            println!("  {}. {} [{}]", i + 1, item.name, status);
            println!("     源路径: {:?}", item.source_path);
            println!("     备份路径: {:?}", item.backup_path);
        }
    }
    
    // 然后按组显示其他项目
    for (group_name_opt, items) in by_group.iter() {
        if let Some(group_name) = group_name_opt {
            println!("\n组: {}:", group_name);
            for (i, item) in items.iter().enumerate() {
                let status = if item.is_enabled { "启用" } else { "禁用" };
                println!("  {}. {} [{}]", i + 1, item.name, status);
                println!("     源路径: {:?}", item.source_path);
                println!("     备份路径: {:?}", item.backup_path);
            }
        }
    }
    
    Ok(())
}

// 删除同步项目
pub fn remove_sync_item(config: &mut Config) -> Result<()> {
    if config.sync_items.is_empty() {
        println!("没有同步项目可以删除。");
        return Ok(());
    }
    
    println!("同步项目列表:");
    for (i, item) in config.sync_items.iter().enumerate() {
        println!("  {}. {}", i + 1, item.name);
    }
    
    print!("请输入要删除的项目编号 (或输入0取消): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let selection: usize = match input.trim().parse() {
        Ok(num) if num > 0 && num <= config.sync_items.len() => num - 1,
        Ok(0) => return Ok(()),
        _ => {
            println!("无效的选择。删除已取消。");
            return Ok(());
        }
    };
    
    let removed = config.sync_items.remove(selection);
    config.save()?;
    
    println!("已删除同步项目: {}", removed.name);
    
    Ok(())
}

// 启用或禁用同步项目
pub fn toggle_sync_item(config: &mut Config) -> Result<()> {
    if config.sync_items.is_empty() {
        println!("没有同步项目可以修改。");
        return Ok(());
    }
    
    println!("同步项目列表:");
    for (i, item) in config.sync_items.iter().enumerate() {
        let status = if item.is_enabled { "启用" } else { "禁用" };
        println!("  {}. {} [{}]", i + 1, item.name, status);
    }
    
    print!("请输入要切换状态的项目编号 (或输入0取消): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let selection: usize = match input.trim().parse() {
        Ok(num) if num > 0 && num <= config.sync_items.len() => num - 1,
        Ok(0) => return Ok(()),
        _ => {
            println!("无效的选择。操作已取消。");
            return Ok(());
        }
    };
    
    // 切换启用状态
    config.sync_items[selection].is_enabled = !config.sync_items[selection].is_enabled;
    let status = if config.sync_items[selection].is_enabled { "启用" } else { "禁用" };
    
    config.save()?;
    
    println!("已将同步项目 {} {}", config.sync_items[selection].name, status);
    
    Ok(())
}

// 一键备份所有启用的同步项目
pub fn backup_all(config: &Config) -> Result<()> {
    config.ensure_backup_dir()?;
    
    let enabled_items: Vec<_> = config.sync_items
        .iter()
        .filter(|item| item.is_enabled)
        .collect();
    
    if enabled_items.is_empty() {
        println!("没有启用的同步项目。");
        return Ok(());
    }
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    println!("正在备份所有启用的同步项目...");
    
    for item in enabled_items {
        if !Path::new(&item.source_path).exists() {
            println!("警告: 源路径不存在: {:?}", item.source_path);
            continue;
        }
        
        // 创建带时间戳的备份目录
        let backup_dir = config.backup_dir.join(format!("{}_{}", item.name, timestamp));
        fs::create_dir_all(&backup_dir)
            .with_context(|| format!("创建备份目录失败: {:?}", backup_dir))?;
        
        if Path::new(&item.source_path).is_dir() {
            // 复制目录
            let options = CopyOptions::new();
            copy(&item.source_path, &backup_dir, &options).with_context(|| {
                format!(
                    "从 {:?} 备份到 {:?} 失败",
                    item.source_path, backup_dir
                )
            })?;
        } else {
            // 复制文件
            let file_name = item.source_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("backup"));
            let target_path = backup_dir.join(file_name);
            
            fs::copy(&item.source_path, &target_path).with_context(|| {
                format!(
                    "从 {:?} 备份到 {:?} 失败",
                    item.source_path, target_path
                )
            })?;
        }
        
        println!("已备份 {}: {:?} -> {:?}", item.name, item.source_path, backup_dir);
    }
    
    println!("所有同步项目备份完成！");
    
    Ok(())
}

// 一键恢复所有启用的同步项目
pub fn restore_all(config: &Config) -> Result<()> {
    let enabled_items: Vec<_> = config.sync_items
        .iter()
        .filter(|item| item.is_enabled)
        .collect();
    
    if enabled_items.is_empty() {
        println!("没有启用的同步项目。");
        return Ok(());
    }
    
    println!("要恢复所有启用的同步项目吗？这将覆盖现有文件。");
    print!("请输入'yes'确认: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() != "yes" {
        println!("恢复操作已取消。");
        return Ok(());
    }
    
    println!("正在恢复所有启用的同步项目...");
    
    for item in enabled_items {
        // 查找最新的备份
        let backups = list_backups_for_item(config, &item.name)?;
        
        if backups.is_empty() {
            println!("警告: 没有找到 {} 的备份", item.name);
            continue;
        }
        
        // 使用最新的备份（第一个）
        let latest_backup = &backups[0];
        
        // 确保源路径的父目录存在
        if let Some(parent) = item.source_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("创建源路径的父目录失败: {:?}", parent))?;
        }
        
        // 如果存在，则删除当前源路径
        if Path::new(&item.source_path).exists() {
            if Path::new(&item.source_path).is_dir() {
                fs::remove_dir_all(&item.source_path).with_context(|| {
                    format!("删除现有目录失败: {:?}", item.source_path)
                })?;
            } else {
                fs::remove_file(&item.source_path).with_context(|| {
                    format!("删除现有文件失败: {:?}", item.source_path)
                })?;
            }
        }
        
        // 从备份恢复
        if Path::new(latest_backup).is_dir() {
            // 如果备份是目录，我们需要找到里面的内容（通常是一个文件夹）
            let entries = fs::read_dir(latest_backup)?;
            let entry_paths: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
            
            if entry_paths.len() == 1 && entry_paths[0].is_dir() {
                // 如果只有一个子目录，复制其内容
                let source_dir = &entry_paths[0];
                let options = CopyOptions::new();
                
                if item.source_path.is_dir() {
                    // 目标是目录，我们复制整个目录
                    copy(source_dir, item.source_path.parent().unwrap(), &options).with_context(|| {
                        format!(
                            "从 {:?} 恢复到 {:?} 失败",
                            source_dir, item.source_path
                        )
                    })?;
                } else {
                    // 目标是文件，我们需要找到备份中的对应文件
                    let source_entries = fs::read_dir(source_dir)?;
                    for entry in source_entries.filter_map(Result::ok) {
                        if entry.path().is_file() {
                            fs::copy(entry.path(), &item.source_path).with_context(|| {
                                format!(
                                    "从 {:?} 恢复到 {:?} 失败",
                                    entry.path(), item.source_path
                                )
                            })?;
                            break;
                        }
                    }
                }
            } else {
                // 复制整个备份目录
                let options = CopyOptions::new();
                copy(latest_backup, item.source_path.parent().unwrap(), &options).with_context(|| {
                    format!(
                        "从 {:?} 恢复到 {:?} 失败",
                        latest_backup, item.source_path
                    )
                })?;
            }
        } else {
            // 备份是单个文件
            fs::copy(latest_backup, &item.source_path).with_context(|| {
                format!(
                    "从 {:?} 恢复到 {:?} 失败",
                    latest_backup, item.source_path
                )
            })?;
        }
        
        println!("已恢复 {}: {:?} <- {:?}", item.name, item.source_path, latest_backup);
    }
    
    println!("所有同步项目恢复完成！");
    
    Ok(())
}

// 列出指定项目的所有备份，按照时间排序（最新的在前）
fn list_backups_for_item(config: &Config, item_name: &str) -> Result<Vec<PathBuf>> {
    if !config.backup_dir.exists() {
        return Ok(Vec::new());
    }
    
    let entries = fs::read_dir(&config.backup_dir)
        .with_context(|| format!("读取备份目录失败: {:?}", config.backup_dir))?;
    
    let mut backups = Vec::new();
    
    // 查找格式为 "item_name_timestamp" 的备份
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        
        if file_name.starts_with(&format!("{}_", item_name)) {
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

// 添加同步组
pub fn add_sync_group(config: &mut Config) -> Result<()> {
    print!("请输入同步组名称: ");
    let mut name = String::new();
    std::io::stdin().read_line(&mut name)?;
    let name = name.trim().to_string();
    
    // 检查组名是否已存在
    if config.sync_groups.iter().any(|g| g.name == name) {
        println!("错误: 同步组 '{}' 已存在", name);
        return Ok(());
    }
    
    print!("请输入组描述 (可选): ");
    let mut description = String::new();
    std::io::stdin().read_line(&mut description)?;
    let description = description.trim();
    let description = if description.is_empty() { None } else { Some(description.to_string()) };
    
    let group_name = name.clone(); // 克隆名称用于输出
    
    config.sync_groups.push(SyncGroup {
        name,
        description,
        is_enabled: true,
    });
    
    config.save()?;
    println!("已添加同步组: {}", group_name);
    
    Ok(())
}

// 列出所有同步组
pub fn list_sync_groups(config: &Config) -> Result<()> {
    println!("同步组列表:");
    if config.sync_groups.is_empty() {
        println!("  没有配置同步组。");
        return Ok(());
    }
    
    for (i, group) in config.sync_groups.iter().enumerate() {
        let status = if group.is_enabled { "启用" } else { "禁用" };
        println!("  {}. {} [{}]", i + 1, group.name, status);
        if let Some(desc) = &group.description {
            println!("     描述: {}", desc);
        }
        
        // 计算组内项目数量
        let items_count = config.sync_items.iter()
            .filter(|item| item.group.as_deref() == Some(&group.name))
            .count();
        println!("     包含 {} 个同步项目", items_count);
    }
    
    Ok(())
}

// 删除同步组
pub fn remove_sync_group(config: &mut Config) -> Result<()> {
    if config.sync_groups.is_empty() {
        println!("没有同步组可以删除。");
        return Ok(());
    }
    
    println!("同步组列表:");
    for (i, group) in config.sync_groups.iter().enumerate() {
        println!("  {}. {}", i + 1, group.name);
    }
    
    print!("请输入要删除的组编号 (或输入0取消): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let selection: usize = match input.trim().parse() {
        Ok(num) if num > 0 && num <= config.sync_groups.len() => num - 1,
        Ok(0) => return Ok(()),
        _ => {
            println!("无效的选择。删除已取消。");
            return Ok(());
        }
    };
    
    let group_name = config.sync_groups[selection].name.clone();
    
    // 询问如何处理该组中的项目
    println!("该组中的同步项目将如何处理?");
    println!("1. 将它们设为无组");
    println!("2. 一并删除这些项目");
    
    print!("请选择 (1/2): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => {
            // 将项目设为无组
            for item in &mut config.sync_items {
                if item.group.as_deref() == Some(&group_name) {
                    item.group = None;
                }
            }
        },
        "2" => {
            // 删除组内项目
            config.sync_items.retain(|item| item.group.as_deref() != Some(&group_name));
        },
        _ => {
            println!("无效的选择。删除已取消。");
            return Ok(());
        }
    }
    
    // 删除组
    config.sync_groups.remove(selection);
    config.save()?;
    
    println!("已删除同步组: {}", group_name);
    
    Ok(())
}

// 启用或禁用同步组
pub fn toggle_sync_group(config: &mut Config) -> Result<()> {
    if config.sync_groups.is_empty() {
        println!("没有同步组可以修改。");
        return Ok(());
    }
    
    println!("同步组列表:");
    for (i, group) in config.sync_groups.iter().enumerate() {
        let status = if group.is_enabled { "启用" } else { "禁用" };
        println!("  {}. {} [{}]", i + 1, group.name, status);
    }
    
    print!("请输入要切换状态的组编号 (或输入0取消): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let selection: usize = match input.trim().parse() {
        Ok(num) if num > 0 && num <= config.sync_groups.len() => num - 1,
        Ok(0) => return Ok(()),
        _ => {
            println!("无效的选择。操作已取消。");
            return Ok(());
        }
    };
    
    // 切换启用状态
    config.sync_groups[selection].is_enabled = !config.sync_groups[selection].is_enabled;
    let status = if config.sync_groups[selection].is_enabled { "启用" } else { "禁用" };
    
    config.save()?;
    
    println!("已将同步组 {} {}", config.sync_groups[selection].name, status);
    
    Ok(())
}

// 备份指定组的所有项目
pub fn backup_group(config: &Config) -> Result<()> {
    if config.sync_groups.is_empty() {
        println!("没有同步组可以备份。");
        return Ok(());
    }
    
    println!("同步组列表:");
    for (i, group) in config.sync_groups.iter().enumerate() {
        let status = if group.is_enabled { "启用" } else { "禁用" };
        println!("  {}. {} [{}]", i + 1, group.name, status);
    }
    
    print!("请输入要备份的组编号 (或输入0取消): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let selection: usize = match input.trim().parse() {
        Ok(num) if num > 0 && num <= config.sync_groups.len() => num - 1,
        Ok(0) => return Ok(()),
        _ => {
            println!("无效的选择。操作已取消。");
            return Ok(());
        }
    };
    
    let group_name = &config.sync_groups[selection].name;
    
    // 确保组是启用的
    if !config.sync_groups[selection].is_enabled {
        println!("警告: 组 '{}' 当前已禁用，是否继续?", group_name);
        print!("请输入 'yes' 确认: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() != "yes" {
            println!("备份已取消。");
            return Ok(());
        }
    }
    
    // 找到该组中的所有启用项目
    let group_items: Vec<_> = config.sync_items
        .iter()
        .filter(|item| item.is_enabled && item.group.as_deref() == Some(group_name))
        .collect();
    
    if group_items.is_empty() {
        println!("组 '{}' 中没有启用的同步项目。", group_name);
        return Ok(());
    }
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    println!("正在备份组 '{}' 中的 {} 个项目...", group_name, group_items.len());
    
    config.ensure_backup_dir()?;
    
    for item in &group_items {
        if !Path::new(&item.source_path).exists() {
            println!("警告: 源路径不存在: {:?}", item.source_path);
            continue;
        }
        
        // 创建带时间戳的备份目录
        let backup_dir = config.backup_dir.join(format!("{}_{}", item.name, timestamp));
        fs::create_dir_all(&backup_dir)
            .with_context(|| format!("创建备份目录失败: {:?}", backup_dir))?;
        
        if Path::new(&item.source_path).is_dir() {
            // 复制目录
            let options = CopyOptions::new();
            copy(&item.source_path, &backup_dir, &options).with_context(|| {
                format!(
                    "从 {:?} 备份到 {:?} 失败",
                    item.source_path, backup_dir
                )
            })?;
        } else {
            // 复制文件
            let file_name = item.source_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("backup"));
            let target_path = backup_dir.join(file_name);
            
            fs::copy(&item.source_path, &target_path).with_context(|| {
                format!(
                    "从 {:?} 备份到 {:?} 失败",
                    item.source_path, target_path
                )
            })?;
        }
        
        println!("已备份 {}: {:?} -> {:?}", item.name, item.source_path, backup_dir);
    }
    
    println!("组 '{}' 的备份完成！", group_name);
    
    Ok(())
}

// 恢复指定组的所有项目
pub fn restore_group(config: &Config) -> Result<()> {
    if config.sync_groups.is_empty() {
        println!("没有同步组可以恢复。");
        return Ok(());
    }
    
    println!("同步组列表:");
    for (i, group) in config.sync_groups.iter().enumerate() {
        let status = if group.is_enabled { "启用" } else { "禁用" };
        println!("  {}. {} [{}]", i + 1, group.name, status);
    }
    
    print!("请输入要恢复的组编号 (或输入0取消): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let selection: usize = match input.trim().parse() {
        Ok(num) if num > 0 && num <= config.sync_groups.len() => num - 1,
        Ok(0) => return Ok(()),
        _ => {
            println!("无效的选择。操作已取消。");
            return Ok(());
        }
    };
    
    let group_name = &config.sync_groups[selection].name;
    
    // 找到该组中的所有启用项目
    let group_items: Vec<_> = config.sync_items
        .iter()
        .filter(|item| item.is_enabled && item.group.as_deref() == Some(group_name))
        .collect();
    
    if group_items.is_empty() {
        println!("组 '{}' 中没有启用的同步项目。", group_name);
        return Ok(());
    }
    
    println!("要恢复组 '{}' 中的 {} 个项目吗？这将覆盖现有文件。", 
             group_name, group_items.len());
    print!("请输入'yes'确认: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() != "yes" {
        println!("恢复操作已取消。");
        return Ok(());
    }
    
    println!("正在恢复组 '{}' 中的项目...", group_name);
    
    for item in group_items {
        // 查找最新的备份
        let backups = list_backups_for_item(config, &item.name)?;
        
        if backups.is_empty() {
            println!("警告: 没有找到 {} 的备份", item.name);
            continue;
        }
        
        // 使用最新的备份（第一个）
        let latest_backup = &backups[0];
        
        // 确保源路径的父目录存在
        if let Some(parent) = item.source_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("创建源路径的父目录失败: {:?}", parent))?;
        }
        
        // 如果存在，则删除当前源路径
        if Path::new(&item.source_path).exists() {
            if Path::new(&item.source_path).is_dir() {
                fs::remove_dir_all(&item.source_path).with_context(|| {
                    format!("删除现有目录失败: {:?}", item.source_path)
                })?;
            } else {
                fs::remove_file(&item.source_path).with_context(|| {
                    format!("删除现有文件失败: {:?}", item.source_path)
                })?;
            }
        }
        
        // 从备份恢复，这部分代码与restore_all相同
        if Path::new(latest_backup).is_dir() {
            // 如果备份是目录，我们需要找到里面的内容（通常是一个文件夹）
            let entries = fs::read_dir(latest_backup)?;
            let entry_paths: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
            
            if entry_paths.len() == 1 && entry_paths[0].is_dir() {
                // 如果只有一个子目录，复制其内容
                let source_dir = &entry_paths[0];
                let options = CopyOptions::new();
                
                if item.source_path.is_dir() {
                    // 目标是目录，我们复制整个目录
                    copy(source_dir, item.source_path.parent().unwrap(), &options).with_context(|| {
                        format!(
                            "从 {:?} 恢复到 {:?} 失败",
                            source_dir, item.source_path
                        )
                    })?;
                } else {
                    // 目标是文件，我们需要找到备份中的对应文件
                    let source_entries = fs::read_dir(source_dir)?;
                    for entry in source_entries.filter_map(Result::ok) {
                        if entry.path().is_file() {
                            fs::copy(entry.path(), &item.source_path).with_context(|| {
                                format!(
                                    "从 {:?} 恢复到 {:?} 失败",
                                    entry.path(), item.source_path
                                )
                            })?;
                            break;
                        }
                    }
                }
            } else {
                // 复制整个备份目录
                let options = CopyOptions::new();
                copy(latest_backup, item.source_path.parent().unwrap(), &options).with_context(|| {
                    format!(
                        "从 {:?} 恢复到 {:?} 失败",
                        latest_backup, item.source_path
                    )
                })?;
            }
        } else {
            // 备份是单个文件
            fs::copy(latest_backup, &item.source_path).with_context(|| {
                format!(
                    "从 {:?} 恢复到 {:?} 失败",
                    latest_backup, item.source_path
                )
            })?;
        }
        
        println!("已恢复 {}: {:?} <- {:?}", item.name, item.source_path, latest_backup);
    }
    
    println!("组 '{}' 的所有项目恢复完成！", group_name);
    
    Ok(())
}

// 为同步项目分配组
pub fn assign_group_to_item(config: &mut Config) -> Result<()> {
    if config.sync_items.is_empty() {
        println!("没有同步项目可以修改。");
        return Ok(());
    }
    
    if config.sync_groups.is_empty() {
        println!("没有可用的同步组，请先创建组。");
        return Ok(());
    }
    
    println!("同步项目列表:");
    for (i, item) in config.sync_items.iter().enumerate() {
        let group_info = if let Some(group) = &item.group {
            format!("组: {}", group)
        } else {
            "无组".to_string()
        };
        println!("  {}. {} ({})", i + 1, item.name, group_info);
    }
    
    print!("请输入要修改的项目编号 (或输入0取消): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let item_idx: usize = match input.trim().parse() {
        Ok(num) if num > 0 && num <= config.sync_items.len() => num - 1,
        Ok(0) => return Ok(()),
        _ => {
            println!("无效的选择。操作已取消。");
            return Ok(());
        }
    };
    
    println!("可用的同步组:");
    println!("  0. 无组 (从组中移除)");
    for (i, group) in config.sync_groups.iter().enumerate() {
        println!("  {}. {}", i + 1, group.name);
    }
    
    print!("请选择要分配的组 (或输入0表示无组): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let group_selection = input.trim().parse::<usize>();
    
    match group_selection {
        Ok(0) => {
            config.sync_items[item_idx].group = None;
            println!("已将 {} 从组中移除", config.sync_items[item_idx].name);
        },
        Ok(num) if num <= config.sync_groups.len() => {
            let group_name = config.sync_groups[num - 1].name.clone();
            config.sync_items[item_idx].group = Some(group_name.clone());
            println!("已将 {} 分配到组 {}", config.sync_items[item_idx].name, group_name);
        },
        _ => {
            println!("无效的选择。操作已取消。");
            return Ok(());
        }
    }
    
    config.save()?;
    
    Ok(())
} 