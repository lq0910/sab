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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub home_mode: HomeMode,
    pub search_engine: SearchEngine,
    pub show_recent: bool,
    pub startup_url: String,
    #[serde(default)]
    pub theme: Theme,
}
impl Default for Settings {
    fn default() -> Self {
        // 功能：默认主页显示最近访问区，空列表时展示品牌提示；时间：2026-11-08；作者：lq。
        Self { home_mode: HomeMode::Minimal, search_engine: SearchEngine::Bing, show_recent: true, startup_url: String::new(), theme: Theme::Obsidian }
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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: String, pub url: String, pub title: String,
    pub folder_id: String, pub tags: Vec<String>, pub pinned: bool, pub updated_at: i64,
}

#[derive(Serialize)]
pub struct BookmarkPage { pub items: Vec<Bookmark>, pub total: u32 }

#[derive(Serialize)]
pub struct Bootstrap { pub settings: Settings, pub folders: Vec<Folder> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentVisit { pub url: String, pub title: String, pub visited_at: i64 }
