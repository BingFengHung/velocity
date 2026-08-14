# Velocity (`vl` / `velocity`)

[English](README.md) | [繁體中文](README.zh-TW.md)

**Velocity** 是以 Rust 打造的高效、極速、現代化終端雙欄檔案瀏覽器。具備雙欄佈局、程式碼真彩色語法高亮、類 fzf 模糊搜尋、Git 狀態整合、24-bit 圖片與壓縮包即時預覽、多維度排序、輕量檔案操作與 Nerd Font 圖示支援。

專為終端重度使用者、開發者工作流程以及 AI 輔助開發工具（如 Copilot / Antigravity CLI）終端交接打造之極速、零相依檔案瀏覽與管理神器。

---

## ✨ 主要特色

- ⚡ **極速效能與零相依**：以純 Rust 編寫，原生二進制執行檔，毫秒級秒速啟動與極低記憶體佔用。
- 🪟 **邊框雙欄設計**：左欄為檔案目錄樹狀導航，右欄提供即時內容預覽。
- 🌈 **程式碼真彩色語法高亮**：內建針對 Rust、Python、JavaScript/TypeScript、Go、C/C++、JSON、TOML、YAML、Markdown、HTML、CSS、Shell、SQL 等程式碼的語法高亮著色引擎。
- 🖼️ **24-bit True Color 圖片即時縮圖預覽**：內建像素級色塊渲染引擎，直接在終端機內即時預覽 PNG、JPEG、GIF、WebP、BMP 與 ICO 圖片。
- 📦 **ZIP 壓縮包結構即時檢視**：游標停在 `.zip` 檔案時，右欄直接顯示壓縮包內部檔案樹與解壓後大小，無須解壓。
- 🌿 **Git 狀態與分支整合**：即時顯示目前 Git 分支名稱，並於清單標註檔案版本狀態（`M` 修改、`+` 暫存、`?` 未追蹤、`D` 刪除）。
- 🔍 **類 fzf 智慧模糊搜尋**：按 `/` 即可輸入縮寫進行字元級模糊搜尋，支援連續匹配加分與命中字元色彩高亮。
- 🔄 **多維度即時排序**：按 `s` 鍵即時切換排序方式（名稱 A-Z → 修改時間最新 → 檔案大小最大 → 副檔名分組）。
- 🛠️ **輕量檔案管理操作**：
  - `y`：一鍵將檔案/目錄絕對路徑複製到系統剪貼簿。
  - `a` / `n`：新增檔案（輸入結尾為 `/` 則自動建立資料夾）。
  - `c`：快速重新命名。
  - `d`：刪除選取項目（附確認提示防誤刪）。
- 🔤 **多種圖示模式**：內建支援 **Nerd Font** 圖示、**Emoji** 圖示以及相容性最佳的 **ASCII** 模式。
- 🌐 **精確 CJK 與 Unicode 寬度計算**：避免繁體中文等寬字元造成終端邊框破版。
- 📝 **文字編輯器整合**：按 `e` 即可直接以 `$EDITOR`、VS Code 或系統預設文字編輯器開啟檔案。
- 🌍 **全平台相容**：完美支援 Windows、Linux 與 macOS。

---

## 📦 安裝說明

### 預編譯二進制檔下載

您可以直接從 GitHub Releases 下載適用於您作業系統的預先編譯版本。

### 雲端自動建置 (GitHub Actions)

本專案嚴格遵循**地端零編譯原則**。所有跨平台（Windows、Linux、macOS）的發布用二進制檔皆在推送版本標籤（`v*.*.*`）時，由 GitHub Actions 雲端 CI/CD 自動編譯、打包並發布至 Release。

---

## 🚀 快速上手

```bash
# 從目前工作目錄啟動 Velocity
velocity

# （推薦）設定終端別名 'vl' 享受極速輸入體驗：
# PowerShell: Set-Alias vl velocity
# Bash/Zsh:   alias vl="velocity"
vl

# 指定瀏覽起點目錄
velocity D:\projects\my_repo

# 使用 Emoji 圖示（未安裝 Nerd Font 字型時適用）
velocity --icons emoji
velocity -i emoji

# 使用純 ASCII 相容模式
velocity --icons ascii
velocity -i ascii

# 顯示隱藏檔案與目錄（以 '.' 開頭之項目）
velocity --all
velocity -a
```

---

## ⌨️ 操作快捷鍵

| 按鍵 | 功能說明 |
|---|---|
| `↑` `↓` / `k` `j` | 上下移動選取游標 |
| `PgUp` `PgDn` | 向上 / 向下翻頁 |
| `Home` `End` | 快速跳至最頂端 / 最底端 |
| `→` / `l` / `Enter` | 進入選取的目錄 |
| `←` / `h` / `Backspace` | 回到上一層目錄（游標自動選取剛剛退出的目錄） |
| `/` | 進入即時模糊搜尋過濾模式（`Enter` 定案、`Esc` 取消） |
| `s` / `o` | 切換排序模式（名稱 → 修改時間 → 檔案大小 → 副檔名） |
| `y` | 複製選取項目的絕對路徑至剪貼簿 |
| `a` / `n` | 新增檔案或資料夾（名稱以 `/` 結尾則建立目錄） |
| `c` | 重新命名選取的檔案或資料夾 |
| `d` | 刪除選取的項目（彈出確認提示） |
| `e` | 使用 `$EDITOR`、VS Code 或預設編輯器開啟檔案 |
| `r` | 重新整理目錄內容與 Git 狀態 |
| `.` | 切換 顯示 / 隱藏 隱藏檔案 |
| `q` / `Esc` | 離開 Velocity |

---

## 🛠️ CLI 命令列參數

```
用法: velocity.exe [OPTIONS] [PATH]

參數:
  [PATH]  起始資料夾路徑 [預設: 目前工作目錄]

選項:
  -i, --icons <ICONS>  圖示風格樣式 [預設: nerd] [可選值: nerd, emoji, ascii]
  -a, --all            顯示隱藏檔案與目錄 (以 '.' 開頭之名稱)
  -h, --help           顯示說明資訊
  -V, --version        顯示版本號
```

---

## 📄 授權條款

本專案採用 [MIT License](LICENSE-MIT) 或 [Apache License 2.0](LICENSE-APACHE) 雙重授權。
