mod account;
mod config;
mod script;

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
            "游戏配置",
            "脚本管理",
            "退出",
        ];
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择一个选项")
            .default(0)
            .items(&options)
            .interact()?;
            
        match selection {
            0 => account_menu(&mut config)?,
            1 => game_config_menu(&mut config)?,
            2 => script_menu(&mut config)?,
            3 => break,
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
        0 => account::list_accounts(config)?,
        1 => account::add_account(config)?,
        2 => account::remove_account(config)?,
        3 => account::select_and_login(config)?,
        4 => return Ok(()),
        _ => unreachable!(),
    }
    
    Ok(())
}

fn game_config_menu(config: &mut config::Config) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("游戏配置");
    println!("========");
    
    let options = vec![
        "查看游戏配置路径",
        "设置游戏配置路径",
        "备份游戏配置",
        "恢复游戏配置",
        "返回主菜单",
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择一个选项")
        .default(0)
        .items(&options)
        .interact()?;
        
    match selection {
        0 => {
            println!("当前游戏配置路径: {:?}", config.game_config_path);
        },
        1 => {
            print!("请输入新的游戏配置路径: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            config.game_config_path = std::path::PathBuf::from(input.trim());
            config.save()?;
            
            println!("游戏配置路径已更新为: {:?}", config.game_config_path);
        },
        2 => config::backup::backup_game_config(config)?,
        3 => config::backup::restore_game_config(config)?,
        4 => return Ok(()),
        _ => unreachable!(),
    }
    
    Ok(())
}

fn script_menu(config: &mut config::Config) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("脚本管理");
    println!("========");
    
    let options = vec![
        "列出脚本路径",
        "添加脚本路径",
        "删除脚本路径",
        "备份脚本",
        "恢复脚本",
        "返回主菜单",
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择一个选项")
        .default(0)
        .items(&options)
        .interact()?;
        
    match selection {
        0 => {
            println!("脚本路径:");
            if config.script_paths.is_empty() {
                println!("  没有配置脚本路径。");
            } else {
                for (i, path) in config.script_paths.iter().enumerate() {
                    println!("  {}. {:?}", i + 1, path);
                }
            }
        },
        1 => script::add_script_path(config)?,
        2 => script::remove_script_path(config)?,
        3 => script::backup_scripts(config)?,
        4 => script::restore_scripts(config)?,
        5 => return Ok(()),
        _ => unreachable!(),
    }
    
    Ok(())
}
