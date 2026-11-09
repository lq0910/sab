// 功能：定义 Rust 服务与本地 UI 的序列化契约；时间：2026-11-08；作者：lq。
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum HomeMode { #[default] Minimal, Bookmarks }

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchEngine { #[default] Bing, Baidu, Google, Duckduckgo }

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Obsidian,
    Blue,
    Violet,
    Emerald,
    Amber,
}

// 功能：跨 IPC 保存自动、浅色、深色界面模式；时间：2026-11-08；作者：lq。
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode { #[default] Auto, Light, Dark }

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub home_mode: HomeMode,
    pub search_engine: SearchEngine,
    pub show_recent: bool,
    pub startup_url: String,
    #[serde(default)]
    pub theme: Theme,
    // 功能：兼容旧版偏好并保存新的界面明暗模式；时间：2026-11-08；作者：lq。
    #[serde(default)]
    pub color_mode: ColorMode,
}
impl Default for Settings {
    fn default() -> Self {
        // 功能：默认主页显示最近访问区，空列表时展示品牌提示；时间：2026-11-08；作者：lq。
        Self { home_mode: HomeMode::Minimal, search_engine: SearchEngine::Bing, show_recent: true, startup_url: String::new(), theme: Theme::Obsidian, color_mode: ColorMode::Auto }
    }
}

#[derive(Serialize)]
pub struct Folder { pub id: String, pub name: String, pub count: u32 }

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkDraft {
    pub url: String,
    pub title: String,
    pub folder_id: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    // 功能：在浏览器导入时保留原始目录层级供分类规则参考；时间：2026-11-08；作者：lq。
    #[serde(default)]
    pub import_folder: Option<String>,
}

// 功能：汇报浏览器书签导入的新增与跳过数量；时间：2026-11-08；作者：lq。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkImportResult {
    pub imported: u32,
    pub skipped: u32,
    // 功能：汇报通过重复导入恢复的浏览器原目录线索数量；时间：2026-11-08；作者：lq。
    pub folder_metadata_updated: u32,
}

// 功能：描述自动分类预览中的分类分组、候选数量和示例书签；时间：2026-11-08；作者：lq。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationGroup { pub folder_id: String, pub folder_name: String, pub count: u32, pub samples: Vec<String> }

// 功能：向界面返回待整理数量及规则可判断的分类建议；时间：2026-11-08；作者：lq。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationPreview {
    pub groups: Vec<ClassificationGroup>, pub suggested: u32, pub remaining: u32,
    // 功能：提供证据不足或类别冲突的样例，方便用户手动纠正；时间：2026-11-09；作者：lq。
    pub remaining_samples: Vec<String>,
}

// 功能：返回批量分类执行或撤销后的处理数量；时间：2026-11-08；作者：lq。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationBatchResult { pub batch_id: String, pub changed: u32, pub remaining: u32 }

// 功能：返回撤销自动分类时成功还原的收藏数量；时间：2026-11-08；作者：lq。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationUndoResult { pub restored: u32 }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: String, pub url: String, pub title: String,
    pub folder_id: String, pub tags: Vec<String>, pub pinned: bool, pub updated_at: i64,
    // 功能：向界面提供自动分类的可解释原因；时间：2026-11-08；作者：lq。
    pub rule_reason: Option<String>,
}

#[derive(Serialize)]
pub struct BookmarkPage { pub items: Vec<Bookmark>, pub total: u32 }

#[derive(Serialize)]
pub struct Bootstrap { pub settings: Settings, pub folders: Vec<Folder> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentVisit { pub url: String, pub title: String, pub visited_at: i64 }

// 功能：向前端返回浏览历史的分页数据；时间：2026-11-08；作者：lq。
#[derive(Serialize)]
pub struct HistoryPage { pub items: Vec<RecentVisit>, pub total: u32 }
