mod account;
mod config;
mod script;
mod sync;

use anyhow::Result;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};

fn main() -> Result<()> {
    println!("英雄联盟游戏工具");
    println!("==============");
    
    // Load configuration
    let mut config = config::Config::load()?;
    
    loop {
        let term = Term::stdout();
        term.clear_screen()?;
        
        println!("英雄联盟游戏工具");
        println!("==============");
        
        let options = vec![
            "账号管理",
            "路径同步",
            "退出",
        ];
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择一个选项")
            .default(0)
            .items(&options)
            .interact()?;
            
        match selection {
            0 => account_menu(&mut config)?,
            1 => sync_menu(&mut config)?,
            2 => break,
            _ => unreachable!(),
        }
        
        println!("\n按回车键继续...");
        term.read_line()?;
    }
    
    Ok(())
}

fn account_menu(config: &mut config::Config) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("账号管理");
    println!("========");
    
    let options = vec![
        "列出账号",
        "添加账号",
        "删除账号",
        "使用账号登录",
        "返回主菜单",
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择一个选项")
        .default(0)
        .items(&options)
        .interact()?;
        
    match selection {
        0 => {
            account::list_accounts(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        1 => {
            account::add_account(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        2 => {
            account::remove_account(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        3 => {
            account::select_and_login(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        4 => return Ok(()),
        _ => unreachable!(),
    }
    
    Ok(())
}

fn sync_menu(config: &mut config::Config) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("路径同步");
    println!("========");
    
    let options = vec![
        "列出同步项目",
        "添加同步项目",
        "删除同步项目",
        "启用/禁用同步项目",
        "为项目分配组",
        "组管理",
        "备份选项",
        "恢复选项",
        "查看备份目录",
        "设置备份目录",
        "返回主菜单",
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择一个选项")
        .default(0)
        .items(&options)
        .interact()?;
        
    match selection {
        0 => {
            sync::list_sync_items(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        1 => {
            sync::add_sync_item(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        2 => {
            sync::remove_sync_item(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        3 => {
            sync::toggle_sync_item(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        4 => {
            sync::assign_group_to_item(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        5 => group_menu(config)?,
        6 => backup_menu(config)?,
        7 => restore_menu(config)?,
        8 => {
            println!("当前备份目录: {:?}", config.backup_dir);
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        9 => {
            print!("请输入新的备份目录路径: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            let path = std::path::PathBuf::from(input.trim());
            config.backup_dir = path;
            config.save()?;
            
            println!("备份目录已更新为: {:?}", config.backup_dir);
            config.ensure_backup_dir()?;
            
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        10 => return Ok(()),
        _ => unreachable!(),
    }
    
    Ok(())
}

// 组管理菜单
fn group_menu(config: &mut config::Config) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("组管理");
    println!("======");
    
    let options = vec![
        "列出所有组",
        "添加同步组",
        "删除同步组",
        "启用/禁用同步组",
        "返回同步菜单",
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择一个选项")
        .default(0)
        .items(&options)
        .interact()?;
        
    match selection {
        0 => {
            sync::list_sync_groups(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        1 => {
            sync::add_sync_group(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        2 => {
            sync::remove_sync_group(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        3 => {
            sync::toggle_sync_group(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        4 => return Ok(()),
        _ => unreachable!(),
    }
    
    Ok(())
}

// 备份选项菜单
fn backup_menu(config: &mut config::Config) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("备份选项");
    println!("========");
    
    let options = vec![
        "一键备份所有启用的项目",
        "备份指定组的项目",
        "返回同步菜单",
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择一个选项")
        .default(0)
        .items(&options)
        .interact()?;
        
    match selection {
        0 => {
            sync::backup_all(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        1 => {
            sync::backup_group(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        2 => return Ok(()),
        _ => unreachable!(),
    }
    
    Ok(())
}

// 恢复选项菜单
fn restore_menu(config: &mut config::Config) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("恢复选项");
    println!("========");
    
    let options = vec![
        "一键恢复所有启用的项目",
        "恢复指定组的项目",
        "返回同步菜单",
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择一个选项")
        .default(0)
        .items(&options)
        .interact()?;
        
    match selection {
        0 => {
            sync::restore_all(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        1 => {
            sync::restore_group(config)?;
            println!("\n按回车键继续...");
            term.read_line()?;
        },
        2 => return Ok(()),
        _ => unreachable!(),
    }
    
    Ok(())
}
