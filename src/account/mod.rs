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

// 将字符转换为扫描码的辅助函数
fn char_to_scan_code(c: char) -> Option<u16> {
    match c {
        'a'..='z' => Some((c as u16) - ('a' as u16) + 0x04),
        'A'..='Z' => Some((c as u16) - ('A' as u16) + 0x04),
        '1'..='9' => Some((c as u16) - ('1' as u16) + 0x1E),
        '0' => Some(0x27),
        ' ' => Some(0x2C), // 空格
        '!' => Some(0x1E), // 1 + shift
        '@' => Some(0x1F), // 2 + shift
        '#' => Some(0x20), // 3 + shift
        '$' => Some(0x21), // 4 + shift
        '%' => Some(0x22), // 5 + shift
        '^' => Some(0x23), // 6 + shift
        '&' => Some(0x24), // 7 + shift
        '*' => Some(0x25), // 8 + shift
        '(' => Some(0x26), // 9 + shift
        ')' => Some(0x27), // 0 + shift
        '-' => Some(0x2D),
        '_' => Some(0x2D), // - + shift
        '=' => Some(0x2E),
        '+' => Some(0x2E), // = + shift
        '[' => Some(0x2F),
        '{' => Some(0x2F), // [ + shift
        ']' => Some(0x30),
        '}' => Some(0x30), // ] + shift
        '\\' => Some(0x31),
        '|' => Some(0x31), // \ + shift
        ';' => Some(0x33),
        ':' => Some(0x33), // ; + shift
        '\'' => Some(0x34),
        '"' => Some(0x34), // ' + shift
        ',' => Some(0x36),
        '<' => Some(0x36), // , + shift
        '.' => Some(0x37),
        '>' => Some(0x37), // . + shift
        '/' => Some(0x38),
        '?' => Some(0x38), // / + shift
        _ => None,
    }
}

// 判断字符是否需要按住Shift键
fn needs_shift(c: char) -> bool {
    match c {
        'A'..='Z' | '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '(' | ')' |
        '_' | '+' | '{' | '}' | '|' | ':' | '"' | '<' | '>' | '?' => true,
        _ => false,
    }
}

// 使用扫描码输入字符串
fn input_string_with_scan_codes(enigo: &mut Enigo, input: &str) {
    for c in input.chars() {
        if let Some(scan_code) = char_to_scan_code(c) {
            let shift_needed = needs_shift(c);
            
            if shift_needed {
                enigo.key_down(Key::Shift);
            }
            
            // 使用raw_key方法来发送扫描码
            enigo.key_down(Key::Raw(scan_code));
            thread::sleep(Duration::from_millis(5));
            enigo.key_up(Key::Raw(scan_code));
            thread::sleep(Duration::from_millis(5));
            
            if shift_needed {
                enigo.key_up(Key::Shift);
            }
            
            thread::sleep(Duration::from_millis(15));
        }
    }
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
    thread::sleep(Duration::from_millis(100));
    
    // 使用扫描码输入用户名
    input_string_with_scan_codes(&mut enigo, &account.username);
    
    // Tab切换到密码字段
    enigo.key_click(Key::Tab);
    thread::sleep(Duration::from_millis(100));
    
    // 清除密码字段
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('a'));
    enigo.key_up(Key::Control);
    enigo.key_click(Key::Delete);
    thread::sleep(Duration::from_millis(100));
    
    // 使用扫描码输入密码
    input_string_with_scan_codes(&mut enigo, &account.password);
    
    println!("登录信息输入成功！");
    
    Ok(())
} 