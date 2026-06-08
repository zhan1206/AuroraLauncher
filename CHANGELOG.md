# Changelog

All notable changes to Aurora Launcher will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-07

### Added

- 🎨 像素玻璃美学 UI 系统（Press Start 2P + 毛玻璃面板 + 草方块绿主题色）
- 🔑 微软正版登录（OAuth 2.0 设备流：MS → XBL → XSTS → MC Auth）
- 👤 离线模式登录支持
- 📦 版本清单获取（Mojang 官方 + BMCLAPI 镜像双源 + 30分钟缓存）
- ⚡ 多线程下载引擎（8 并发分块 + SHA-256 校验 + 下载队列管理）
- 🗃️ 实例管理系统（独立 `.minecraft` 目录隔离 + CRUD + JSON sidecar）
- 🧩 模组加载器支持（Vanilla / Fabric / Forge / NeoForge / Quilt）
- 🚀 游戏启动引擎（version.json 递归解析 + classpath 组装 + natives 提取）
- ☕ Java 运行时检测与管理
- 🔐 OS 密钥链安全 Token 存储（Windows Credential Manager / macOS Keychain / Linux Secret Service）
- ⚙️ 应用设置持久化（SQLite + JSON 双存储）
- 📊 下载进度实时推送（Tauri Event IPC）
- 🖥️ 全平台支持（Windows NSIS / macOS DMG / Linux AppImage）
- 🔄 CI/CD 自动三平台构建发布（GitHub Actions）
- 📝 完整的 PRD + 架构设计文档

### Technical

- Tauri 2.0 + Rust 后端，27 个 IPC 命令
- Vue 3 + Pinia + Vue Router + TypeScript 前端，111 模块
- Arc::clone 安全进程管理（零 unsafe 指针）
- 统一 CommandResponse 信封格式
- db_pool 共享工具模块（消除重复代码）
