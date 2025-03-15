use anyhow::Result;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};
use enigo::{Enigo, Key, KeyboardControllable};
use std::{thread, time::Duration};

use crate::config::{Account, Config};

pub fn list_accounts(config: &Config) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("账号列表:");
    if config.accounts.is_empty() {
        println!("  没有配置账号。");
        return Ok(());
    }
    
    for (i, account) in config.accounts.iter().enumerate() {
        let note = account.note.as_deref().unwrap_or("");
        println!("  {}. {} {}", i + 1, account.username, note);
    }
    
    Ok(())
}

pub fn add_account(config: &mut Config) -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("添加新账号:");
    
    print!("用户名: ");
    let username = term.read_line()?;
    
    print!("密码: ");
    let password = term.read_line()?;
    
    print!("备注 (可选): ");
    let note = term.read_line()?;
    let note = if note.trim().is_empty() { None } else { Some(note) };
    
    config.accounts.push(Account {
        username,
        password,
        note,
    });
    
    config.save()?;
    println!("账号添加成功！");
    
    Ok(())
}

pub fn remove_account(config: &mut Config) -> Result<()> {
    if config.accounts.is_empty() {
        println!("没有账号可删除。");
        return Ok(());
    }
    
    let selections: Vec<String> = config.accounts
        .iter()
        .enumerate()
        .map(|(i, acc)| {
            let note = acc.note.as_deref().unwrap_or("");
            format!("{}. {} {}", i + 1, acc.username, note)
        })
        .collect();
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择要删除的账号")
        .default(0)
        .items(&selections)
        .interact()?;
    
    let removed = config.accounts.remove(selection);
    config.save()?;
    
    println!("已删除账号: {}", removed.username);
    
    Ok(())
}

pub fn select_and_login(config: &Config) -> Result<()> {
    if config.accounts.is_empty() {
        println!("没有配置账号。请先添加账号。");
        return Ok(());
    }
    
    let selections: Vec<String> = config.accounts
        .iter()
        .enumerate()
        .map(|(i, acc)| {
            let note = acc.note.as_deref().unwrap_or("");
            format!("{}. {} {}", i + 1, acc.username, note)
        })
        .collect();
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择要登录的账号")
        .default(0)
        .items(&selections)
        .interact()?;
    
    let account = &config.accounts[selection];
    
    println!("已选择账号: {}", account.username);
    println!("请在3秒内将焦点切换到英雄联盟登录窗口...");
    thread::sleep(Duration::from_secs(3));
    
    let mut enigo = Enigo::new();
    
    // 清除用户名字段（按Ctrl+A然后Delete）
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('a'));
    enigo.key_up(Key::Control);
    enigo.key_click(Key::Delete);
    
    // 输入用户名
    enigo.key_sequence(&account.username);
    
    // Tab切换到密码字段
    enigo.key_click(Key::Tab);
    
    // 清除密码字段
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('a'));
    enigo.key_up(Key::Control);
    enigo.key_click(Key::Delete);
    
    // 输入密码
    enigo.key_sequence(&account.password);
    
    println!("登录信息输入成功！");
    
    Ok(())
} 