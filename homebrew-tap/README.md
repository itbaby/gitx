# GitX Homebrew Tap

本仓库是 GitX 的官方 Homebrew Tap，允许用户通过 Homebrew 安装 GitX。

## 安装命令

```bash
# 添加 tap（只需执行一次）
brew tap itbaby/tap

# 安装 gitx
brew install gitx
```

## 创建 Tap 仓库（管理员指南）

如果你需要创建新的 `itbaby/homebrew-tap` 仓库，请按以下步骤操作：

### 1. 在 GitHub 创建仓库

1. 登录 GitHub 账号
2. 创建新仓库，名称为 `homebrew-tap`
3. 仓库描述：`Homebrew tap for GitX - AI-Powered Git Diff Analyzer`
4. 设置为 Public
5. 初始化时不添加 README

### 2. 克隆并设置仓库

```bash
# 克隆空仓库
git clone https://github.com/itbaby/homebrew-tap.git
cd homebrew-tap

# 创建 Formula 目录
mkdir -p Formula

# 从 gitx 主仓库复制 formula 文件
cp ../gitx/homebrew-tap/Formula/gitx.rb Formula/

# 提交并推送
git add .
git commit -m "Initial Homebrew tap setup"
git push origin main
```

### 3. 配置 GitHub Token（用于自动更新）

1. 在 GitHub 创建 Personal Access Token：
   - Settings → Developer settings → Personal access tokens → Fine-grained tokens
   - 或 Tokens (classic) → Generate new token
   - 权限：repo (Full control of private repositories)

2. 在 itbaby/gitx 仓库添加 secrets：
   - Settings → Secrets and variables → Actions → New repository secret
   - 名称：`HOMEBREW_TOKEN`
   - 值：粘贴你创建的 Personal Access Token

### 4. 发布新版本时

当你在 gitx 仓库发布新版本时，GitHub Actions 会自动：
1. 下载新的安装包
2. 计算 SHA256 校验和
3. 更新 Formula
4. 创建 Pull Request 到 homebrew-tap 仓库

### 5. 手动更新 Formula（可选）

```bash
# 克隆仓库
git clone https://github.com/itbaby/homebrew-tap.git
cd homebrew-tap

# 编辑 Formula
vim Formula/gitx.rb

# 更新版本号和 SHA256
# ...

# 提交并推送
git add .
git commit -m "Update gitx to v0.x.x"
git push origin main
```

### 获取 SHA256 校验和

```bash
# 下载 macOS (Intel) DMG
curl -L https://github.com/itbaby/gitx/releases/download/v0.1.0/GitX_0.1.0_x86_64.dmg -o gitx-intel.dmg
shasum -a 256 gitx-intel.dmg

# 下载 macOS (Apple Silicon) DMG
curl -L https://github.com/itbaby/gitx/releases/download/v0.1.0/GitX_0.1.0_aarch64.dmg -o gitx-arm64.dmg
shasum -a 256 gitx-arm64.dmg

# 下载 Linux DEB
curl -L https://github.com/itbaby/gitx/releases/download/v0.1.0/gitx_0.1.0_amd64.deb -o gitx.deb
shasum -a 256 gitx.deb
```

## 卸载

```bash
brew uninstall gitx
brew untap itbaby/tap
```

## 相关链接

- [GitX 主页](https://github.com/itbaby/gitx)
- [Homebrew 文档](https://docs.brew.sh)
- [Homebrew Formula 教程](https://docs.brew.sh/Formula-Cookbook)
