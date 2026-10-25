# BingeGet - 影集 m3u8 自動抓取與極速下載器 (桌面版 GUI) 專案企劃書

## 一、 專案簡介與目標
本專案名為 **BingeGet**，旨在將原本分散的人工抓取串流、手動維護清單（`list.txt`）以及基於 Shell 腳本（`download_from_list.sh`）的下載流程，全面升級為一個**一體化、現代化且跨平台**的桌面應用程式（GUI）。

### 核心目標：
1. **捨棄 .sh 腳本與文字檔依賴**：所有網路爬蟲、串流解析、多線程切片下載、mp4 自動封裝與進度反饋，全部收斂在單一桌面軟體內。
2. **解決「字幕與碼率混亂」問題**：深入解析多來源影視站點結構，建立「播放渠道標籤鎖定（Channel Lock）」機制，保證全集下載皆來自同一壓制源。
3. **極致模組化架構 (Plugin Pattern)**：將「網站串流解析」與「下載引擎/GUI」完全解耦，未來可以低成本擴充新影視站點。
4. **極致輕量與現代化 GUI**：採用 **Tauri 2.0 (Rust + Web)**，實現秒開啟動、記憶體超低佔用（~30-50MB）與現代視覺體驗。

---

## 二、 核心產品決策與規格定案（Confirmed Specifications）

根據需求評估與確認，本專案的關鍵決策已明確如下：

| 項目 | 決策方案 | 詳細說明 |
| :--- | :--- | :--- |
| **劇名獲取與命名規範** | **用戶完全自訂（不使用爬蟲雜訊名稱）** | 網站爬取的標題往往夾帶廣告、完結或雜訊字串。GUI 提供劇名輸入框由用戶指定劇名，命名規範採標準媒體庫格式：`{用戶輸入劇名} - S{季數:02d}E{集數:02d}.mp4`（預設 Season 1，如 `繁花 - S01E01.mp4`）。不下載海報。 |
| **渠道缺集容錯機制** | **半自動偵測與單集手動指定 (Semi-Auto)** | 當用戶鎖定特定渠道（如 `ikzy`）時，系統預先比對各集渠道可用性。若某集缺失該渠道，UI 標記警告並允許用戶為該集單獨指派替代片源（如 `hnzy`）；未指派前跳過下載，確保片源品質統一。 |
| **下載併發與隊列控制** | **集數隊列控制 + 單集多線程** | 限制**同時下載集數（預設 1 ~ 2 集）**，避免片源 CDN 觸發 429 / 403 封鎖。單集內部線程數可調（預設 16 線程）。支援任務暫停、取消與單集失敗重試。 |
| **外部工具調度** | **系統 PATH 優先 + 自訂路徑設定** | 預設自動偵測 `/usr/bin/n-m3u8dl-re` 與 `/usr/bin/ffmpeg`，並在設定介面提供路徑手動配置與連線測試功能。 |
| **MVP 範圍** | **ChinaQ 完整貫通 + 模組化 Extractor Trait** | 第一階段全力跑通 ChinaQ 的完整流程（解析 $\rightarrow$ 渠道鎖定 $\rightarrow$ 排隊下載 $\rightarrow$ 即時進度 $\rightarrow$ 封裝），並將 `SiteExtractor` 介面抽像化以利未來擴充。 |

---

## 三、 目標網站逆向分析結論（以 `chinaq.fun` 為例）

經實際封包與端點逆向分析驗證：
* **主頁與集數抓取**：請求劇集首頁（如 `https://chinaq.fun/tv-cn/202614235/`），解析 HTML 鏈結 `href="/tv-cn/{drama_id}/ep{n}.html"`，即可完整獲取總集數及各集識別碼。
* **分集渠道獲取**：前端或後端呼叫 API：`GET /qplays/{drama_id}/ep{n}`。
* **API 回傳結構**：
  ```json
  {
    "video_plays": [
      {
        "play_data": "https://hn.bfvvs.com/play/eVO78xMa/index.m3u8",
        "src_site": "hnzy",
        "language": null
      },
      {
        "play_data": "https://bfikuncdn.com/20260809/XpZySbA2/index.m3u8",
        "src_site": "ikzy",
        "language": null
      }
    ]
  }
  ```
* **關鍵邏輯**：
  * `play_data` 即為乾淨可直接下載的 `.m3u8` 連結。
  * 不同集數回傳的 `video_plays` 順序會浮動，因此必須依據 `src_site` 進行渠道匹配與鎖定（Channel Lock）。

---

## 四、 系統架構設計

```mermaid
flowchart TD
    UserInput["用戶輸入: 影集網址 (URL) + 自訂劇名 + 季數 (S01)"] --> Registry["ExtractorRegistry (解析器分發中心)"]
    
    subgraph 模組化網站解析器 (Site Extractors)
        Registry -->|匹配 chinaq.fun| ExtChinaQ["ChinaQExtractor (API / qplays 模式)"]
        Registry -->|匹配 generic| ExtGeneric["GenericExtractor (HTML/m3u8 靜態嗅探)"]
        Registry -->|未來擴展| ExtCustom["...其他站點插件"]
    end
    
    ExtChinaQ --> SeriesData["影集資料結構 (總集數、各集可用渠道清單)"]
    ExtGeneric --> SeriesData
    
    SeriesData --> Frontend["Tauri GUI 介面"]
    Frontend --> Config["用戶配置: 鎖定渠道 (ikzy) / 勾選集數 / 儲存路徑"]
    Config --> DownloadQueue["Rust 下載排程管理器 (Queue Manager, 同步 1~2 集)"]
    
    DownloadQueue --> ChildProcess["原生呼叫 n-m3u8dl-re + ffmpeg"]
    ChildProcess --> LocalFiles["本地媒體庫 (.mp4, 如: 繁花 - S01E01.mp4)"]
    ChildProcess -.->|實時抓取 stdout (下載速度、百分比)| IPC["Tauri Event 推播"]
    IPC -.-> Frontend
```

---

## 五、 核心模組架構規格

### 1. 模組化解析器 (SiteExtractor Trait)
採用插件式抽象架構：

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelOption {
    pub id: String,          // 如 "ikzy"
    pub name: String,        // 如 "iKun 資源"
    pub available_count: u32,// 該渠道可用集數
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeInfo {
    pub ep_number: u32,
    pub ep_id: String,       // 如 "ep1"
    pub available_channels: Vec<String>, // 如 ["ikzy", "hnzy"]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesMetadata {
    pub drama_id: String,
    pub original_title: String,
    pub episodes: Vec<EpisodeInfo>,
    pub channels: Vec<ChannelOption>,
}

#[async_trait]
pub trait SiteExtractor: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn can_handle(&self, url: &str) -> bool;

    /// 階段一：解析影集基礎清單與所有可用渠道標籤
    async fn fetch_series(&self, url: &str) -> anyhow::Result<SeriesMetadata>;

    /// 階段二：依據鎖定的渠道標籤取得特定集數的真實 m3u8
    async fn fetch_m3u8(&self, drama_id: &str, ep_id: &str, channel: &str) -> anyhow::Result<String>;
}
```

### 2. 下載隊列引擎 (DownloadEngine & TaskQueue)
管理集數的隊列、併發與外部進程調度：
* **併發控制**：使用 `tokio::sync::Semaphore` 限制同時活躍的下載任務（預設為 2）。
* **參數組裝**：
  * `--header "User-Agent: Mozilla/5.0 ..."`
  * `--header "Referer: ..."`
  * `--save-dir <輸出目錄>`
  * `--save-name <自訂命名格式>`（例如 `劇名 - S01E01`）
  * `--tmp-dir "<目錄>/tmp_chunks"`
  * `--thread-count <線程數>`（預設 16）
  * `--download-retry-count 5`
  * `--auto-select`
  * `-M format=mp4`
  * `--del-after-done`
* **進度監聽**：透過非同步讀取 `stdout` / `stderr`，以 Regex 解析即時下載速度與進度百分比，利用 Tauri 的 `emit("download-progress", payload)` 實時通知前端。

---

## 六、 GUI 介面設計與交互流程

### Wireframe 示意圖
```text
┌────────────────────────────────────────────────────────────────────────┐
│  🎬 BingeGet - 影集極速下載器                                 [—] [口] [X]│
├────────────────────────────────────────────────────────────────────────┤
│  影集網址: [ https://chinaq.fun/tv-cn/202614235/              ] [  解析  ] │
├────────────────────────────────────────────────────────────────────────┤
│  【影集與命名設定】                                                    │
│   自訂劇名: [ 繁花                   ]  季數: [ S01   ]  總集數: 36 集全 │
│                                                                        │
│  【渠道鎖定 (Channel Lock)】                                            │
│   鎖定播放渠道: [ ikzy (iKun資源 - 推薦，速度快/壓制穩定)           ▼ ] │
│   ℹ️ 系統已為全集鎖定此渠道，保證全集字體與畫質一致。                     │
│                                                                        │
│  【儲存與效能】                                                        │
│   儲存目錄: [ /mnt/data/media/TV/繁花/Season 01             ] [ 瀏覽 ] │
│   單集線程: [ 16 ──●────── 32 ]   同時下載: [ 2 集並發  ▼ ]           │
├────────────────────────────────────────────────────────────────────────┤
│  【集數勾選】 [全選] [清空] [前10集] [自訂範圍 1-20]                   │
│   [✓] 第 01 集    [✓] 第 02 集    [✓] 第 03 集    [✓] 第 04 集          │
│   [✓] 第 05 集    [✓] 第 06 集    [⚠️ 第 07 集 (缺ikzy,已指定hnzy)] ...│
├────────────────────────────────────────────────────────────────────────┤
│  【下載隊列】                                                          │
│   第 01 集: [========================] 100% 完成 (封裝為 mp4)            │
│   第 02 集: [===========>            ]  48%  15.2 MB/s (剩餘 1 分鐘)    │
│   第 03 集: [排隊等待中...]                                             │
├────────────────────────────────────────────────────────────────────────┤
│                                                  [ 開始下載 (共 36 集) ]│
└────────────────────────────────────────────────────────────────────────┘
```

---

## 七、 專案目錄結構

```text
BingeGet/
├── package.json                   # 前端配置 (Vue 3 / Vite / TailwindCSS / Lucide)
├── vite.config.ts
├── src/                           # 現代化 Web 前端
│   ├── App.vue                    # 主視窗佈局
│   ├── components/
│   │   ├── UrlBar.vue             # 網址輸入與解析按鈕
│   │   ├── SeriesConfig.vue       # 劇名、季數與儲存目錄設定
│   │   ├── ChannelLock.vue        # 渠道鎖定下拉選單
│   │   ├── EpisodePicker.vue      # 分集卡片勾選網格與缺集標註
│   │   └── DownloadQueue.vue      # 任務進度列表與即時速率
│   ├── stores/
│   │   └── downloader.ts          # Pinia 狀態管理
│   └── styles/
│       └── main.css               # Tailwind CSS 樣式
└── src-tauri/                     # Rust 原生後端
    ├── Cargo.toml                 # Rust 依賴 (tauri, tokio, reqwest, regex)
    ├── tauri.conf.json            # Tauri 配置
    └── src/
        ├── main.rs                # Tauri 入口與 Command 註冊
        ├── models.rs              # 數據結構定義
        ├── engine/
        │   ├── mod.rs
        │   └── queue.rs           # 隊列排程與 n-m3u8dl-re 進程封裝
        └── extractors/            # 模組化網站解析器
            ├── mod.rs             # Extractor Trait & Registry
            └── chinaq.rs          # ChinaQ 專用解析器
```

---

## 八、 實作時程與驗收步驟

* **階段一：後端核心與 ChinaQ 模組實作及驗證**
  * 實作 `SiteExtractor` 與 `ChinaQExtractor`，透過 Rust 測試案例驗證解析與抓取。
  * 實作 `DownloadQueue`，封裝 `/usr/bin/n-m3u8dl-re` 調度與進度監聽。
* **階段二：Tauri 2.0 與前端 GUI 介面刻劃**
  * 建立 Vue 3 + Tailwind CSS 現代深色介面。
  * 串接自訂劇名、季數、儲存路徑選擇（Tauri Dialog 插件）。
  * 整合渠道鎖定與分集網格勾選。
* **階段三：IPC 串接與端到端下載驗收**
  * 串聯前端事件與後端下載進程，驗證實時進度條與 mp4 封裝。
  * 進行完整影集下載測試。
