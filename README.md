# Aurora Launcher

> 下一代全平台智能 Minecraft 启动器 ✨

**像素玻璃美学 · Tauri 2.0 · Vue 3 · 微软 OAuth**

---

## ✨ 特性

- 🚀 **原生性能**：Tauri 2.0（Rust 后端 + 系统 WebView），安装包 < 10 MB
- 🎨 **像素玻璃 UI**：Press Start 2P 字体 + 毛玻璃面板，致敬 Minecraft 美学
- 🔑 **微软正版登录**：OAuth 2.0 设备流完整链路（MS → XBL → XSTS → MC Auth）
- 📦 **智能版本管理**：Mojang/BMCLAPI 双镜像 + 30分钟缓存
- ⚡ **多线程下载**：8 并发分块下载 + SHA-256 校验 + 断点续传
- 🧩 **Mod 加载器**：内置 Fabric / Forge / NeoForge / Quilt 安装支持
- 🗃️ **实例隔离**：每实例独立 `.minecraft` 目录，互不干扰
- 🔐 **安全存储**：OS 密钥链（keyring）保存 Token，不落明文磁盘
- 🖥️ **全平台**：Windows (NSIS) · macOS (DMG) · Linux (AppImage)

## 🛠️ 技术栈

| 层 | 技术 |
|----|------|
| 后端 | Tauri 2.0 + Rust + tokio + sqlx (SQLite) |
| 前端 | Vue 3 + Vite + TypeScript + Pinia |
| 样式 | TailwindCSS + 自定义像素玻璃主题 |
| 认证 | Microsoft OAuth 2.0 Device Flow |
| 打包 | NSIS (Windows) · DMG (macOS) · AppImage (Linux) |

## 📥 下载安装

### Windows

1. 下载 `Aurora.Launcher_x64-setup.exe`
2. 双击运行，选择安装目录
3. 安装完成后从桌面或开始菜单启动

> 首次运行可能需要安装 WebView2 运行时，安装器会自动提示下载。

### macOS

1. 下载 `Aurora.Launcher_universal.dmg`
2. 打开 DMG，将 Aurora Launcher 拖入 Applications 文件夹
3. 首次打开需右键 → 打开（绕过 Gatekeeper）

### Linux

```bash
# AppImage（推荐）
chmod +x aurora-launcher_amd64.AppImage
./aurora-launcher_amd64.AppImage

# 或安装 deb 包
sudo dpkg -i aurora-launcher_amd64.deb
```

## 🏗️ 从源码构建

### 环境要求

| 工具 | 最低版本 |
|------|---------|
| [Node.js](https://nodejs.org/) | >= 20 |
| [Rust](https://rustup.rs/) | >= 1.77 (stable) |
| Tauri CLI | `cargo install tauri-cli --version "^2.0"` |

### Linux 额外依赖

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev pkg-config
```

### 开发模式

```bash
# 克隆仓库
git clone https://github.com/your-org/aurora-launcher.git
cd aurora-launcher

# 安装前端依赖
npm install

# 启动 Tauri 开发服务器（含热更新）
npm run tauri dev
```

### 构建发布包

```bash
# 构建当前平台的安装包
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`：
- **Windows**：`nsis/*.exe`
- **macOS**：`dmg/*.dmg`
- **Linux**：`appimage/*.AppImage`

## 📁 项目结构

```
aurora-launcher/
├── src/                    # Vue 3 前端
│   ├── components/         # 组件库（像素玻璃系列）
│   │   ├── common/         # PixelButton / PixelCard / PixelDialog 等
│   │   ├── layout/         # AppLayout / AppSidebar / AppHeader
│   │   ├── instance/       # 实例管理组件
│   │   ├── account/        # 账号登录组件
│   │   ├── download/       # 下载队列组件
│   │   └── launch/         # 启动按钮组件
│   ├── stores/             # Pinia 状态管理
│   ├── views/              # 页面视图
│   ├── composables/        # Vue Composition 工具函数
│   └── styles/             # 全局样式（像素玻璃 CSS 系统）
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── commands/       # Tauri IPC 命令层（27个命令）
│   │   ├── services/       # 业务逻辑（版本/实例/下载/账号/启动）
│   │   ├── models/         # 数据模型（sqlx + serde）
│   │   └── utils/          # 工具函数
│   ├── icons/              # 应用图标（多格式）
│   └── Cargo.toml
├── scripts/                # 构建脚本（图标生成等）
├── .github/workflows/      # CI/CD（三平台自动构建发布）
└── deliverables/           # 交付文档（PRD / 架构设计）
```

## 🏛️ 架构概述

```
┌─────────────────────────────────────────┐
│           Vue 3 Frontend                │
│  Components → Stores → Composables     │
│  useTauriCommand  →  Tauri IPC invoke  │
└────────────────┬────────────────────────┘
                 │  CommandResponse<T> 信封
┌────────────────▼────────────────────────┐
│            Rust Backend                 │
│  Commands → Services → DB(SQLite)      │
│  AppState(Arc) · AppError · tokio      │
└─────────────────────────────────────────┘
```

所有 IPC 调用返回统一信封：
```typescript
interface CommandResponse<T> {
  code: number;   // 0 = 成功，1xxxx 网络，2xxxx 文件，3xxxx 认证，4xxxx 启动
  data: T | null;
  message: string;
}
```

## 🚀 发布流程

推送 `v*` 标签自动触发三平台构建并创建 GitHub Release：

```bash
git tag v0.1.0
git push origin v0.1.0
```

CI 会自动：
1. 运行前端类型检查 + 构建
2. 运行 Rust cargo check + clippy
3. 三平台（Windows/macOS/Linux）并行构建安装包
4. 创建 GitHub Release 草稿，附带所有安装包

## 🤝 贡献指南

1. Fork 本仓库
2. 创建功能分支：`git checkout -b feat/your-feature`
3. 提交代码：`git commit -m 'feat: add your feature'`
4. 推送分支并创建 PR

## 📄 许可证

[MIT License](LICENSE) © 2026 Aurora Launcher Team
