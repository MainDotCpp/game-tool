# 英雄联盟游戏工具

一个为Windows平台上的英雄联盟玩家设计的控制台工具。

## 功能

### 账号管理
- 存储多个账号，包括用户名、密码和可选备注
- 选择账号并自动登录（模拟键盘输入）
- 列出、添加和删除账号

### 游戏配置管理
- 备份游戏配置文件/目录
- 从备份中恢复游戏配置
- 配置游戏配置的路径

### 脚本管理
- 管理多个脚本路径（文件或目录）
- 将所有脚本备份到带时间戳的目录
- 从备份中恢复脚本

## 使用方法

1. 运行应用程序：
   ```
   cargo run
   ```

2. 通过交互式菜单使用各种功能。

## 配置

应用程序将其配置存储在以下位置：
- Windows: `%APPDATA%\game_tool\config.json`

备份存储在：
- Windows: `%LOCALAPPDATA%\game_tool_backups\`

## 从源代码构建

```
git clone https://github.com/yourusername/game_tool.git
cd game_tool
cargo build --release
```

编译后的二进制文件将位于 `target/release/game_tool.exe`。

## 系统要求

- Windows操作系统
- Rust编译器（用于从源代码构建）

## 许可证

MIT # game-tool
