use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub domain: String,
    pub name: String,
    pub value: String,
    pub path: String,
    pub expires: Option<i64>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub visit_count: i32,
    pub last_visit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub folder: Option<String>,
    pub created_at: i64,
}

/// Export data structure containing all storage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageExport {
    pub version: String,
    pub exported_at: i64,
    pub cookies: Vec<Cookie>,
    pub history: Vec<HistoryEntry>,
    pub bookmarks: Vec<Bookmark>,
    pub local_storage: HashMap<String, HashMap<String, String>>,
}

/// Import options for controlling what data to import
#[derive(Debug, Clone, Default)]
pub struct ImportOptions {
    /// Whether to merge with existing data or replace it
    pub merge: bool,
    /// Import cookies
    pub import_cookies: bool,
    /// Import history
    pub import_history: bool,
    /// Import bookmarks
    pub import_bookmarks: bool,
    /// Import local storage
    pub import_local_storage: bool,
}

impl ImportOptions {
    /// Create options to import all data types
    pub fn all() -> Self {
        Self {
            merge: true,
            import_cookies: true,
            import_history: true,
            import_bookmarks: true,
            import_local_storage: true,
        }
    }

    /// Create options to replace all data
    pub fn replace_all() -> Self {
        Self {
            merge: false,
            import_cookies: true,
            import_history: true,
            import_bookmarks: true,
            import_local_storage: true,
        }
    }
}

/// Export options for controlling what data to export
#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    /// Export cookies
    pub export_cookies: bool,
    /// Export history
    pub export_history: bool,
    /// Export bookmarks
    pub export_bookmarks: bool,
    /// Export local storage
    pub export_local_storage: bool,
}

impl ExportOptions {
    /// Create options to export all data types
    pub fn all() -> Self {
        Self {
            export_cookies: true,
            export_history: true,
            export_bookmarks: true,
            export_local_storage: true,
        }
    }
}

/// Import/Export statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportExportStats {
    pub cookies_count: usize,
    pub history_count: usize,
    pub bookmarks_count: usize,
    pub local_storage_origins: usize,
    pub local_storage_items: usize,
}

/// In-memory storage engine (no database dependency)
pub struct StorageEngine {
    data_dir: PathBuf,
    cookies: Arc<RwLock<HashMap<String, Cookie>>>, // key: domain+name+path
    history: Arc<RwLock<HashMap<String, HistoryEntry>>>, // key: url
    bookmarks: Arc<RwLock<HashMap<i64, Bookmark>>>, // key: id
    local_storage: Arc<RwLock<HashMap<String, HashMap<String, String>>>>, // key: origin -> (key -> value)
    next_history_id: Arc<RwLock<i64>>,
    next_bookmark_id: Arc<RwLock<i64>>,
}

impl StorageEngine {
    pub fn new(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        
        info!("Initialized in-memory storage engine");
        
        Ok(Self {
            data_dir: data_dir.to_path_buf(),
            cookies: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashMap::new())),
            bookmarks: Arc::new(RwLock::new(HashMap::new())),
            local_storage: Arc::new(RwLock::new(HashMap::new())),
            next_history_id: Arc::new(RwLock::new(1)),
            next_bookmark_id: Arc::new(RwLock::new(1)),
        })
    }

    // =========================================================================
    // EXPORT FUNCTIONS
    // =========================================================================

    /// Export all storage data to a StorageExport struct
    pub async fn export_all(&self) -> Result<StorageExport> {
        self.export_with_options(&ExportOptions::all()).await
    }

    /// Export storage data with specific options
    pub async fn export_with_options(&self, options: &ExportOptions) -> Result<StorageExport> {
        let now = chrono::Utc::now().timestamp();
        
        let cookies = if options.export_cookies {
            self.get_all_cookies().await?
        } else {
            Vec::new()
        };

        let history = if options.export_history {
            let h = self.history.read().await;
            h.values().cloned().collect()
        } else {
            Vec::new()
        };

        let bookmarks = if options.export_bookmarks {
            self.get_bookmarks().await?
        } else {
            Vec::new()
        };

        let local_storage = if options.export_local_storage {
            self.local_storage.read().await.clone()
        } else {
            HashMap::new()
        };

        info!(
            "Exported storage: {} cookies, {} history, {} bookmarks, {} origins",
            cookies.len(),
            history.len(),
            bookmarks.len(),
            local_storage.len()
        );

        Ok(StorageExport {
            version: "1.0.0".to_string(),
            exported_at: now,
            cookies,
            history,
            bookmarks,
            local_storage,
        })
    }

    /// Export storage data to a JSON file
    pub async fn export_to_file(&self, path: &Path) -> Result<ImportExportStats> {
        self.export_to_file_with_options(path, &ExportOptions::all()).await
    }

    /// Export storage data to a JSON file with specific options
    pub async fn export_to_file_with_options(
        &self,
        path: &Path,
        options: &ExportOptions,
    ) -> Result<ImportExportStats> {
        let export = self.export_with_options(options).await?;
        
        let stats = ImportExportStats {
            cookies_count: export.cookies.len(),
            history_count: export.history.len(),
            bookmarks_count: export.bookmarks.len(),
            local_storage_origins: export.local_storage.len(),
            local_storage_items: export.local_storage.values().map(|m| m.len()).sum(),
        };

        let json = serde_json::to_string_pretty(&export)
            .context("Failed to serialize storage data")?;
        
        tokio::fs::write(path, json)
            .await
            .context("Failed to write export file")?;

        info!("Exported storage to file: {:?}", path);
        Ok(stats)
    }

    /// Export storage data to a JSON string
    pub async fn export_to_json(&self) -> Result<String> {
        let export = self.export_all().await?;
        serde_json::to_string_pretty(&export).context("Failed to serialize storage data")
    }

    // =========================================================================
    // IMPORT FUNCTIONS
    // =========================================================================

    /// Import all storage data from a StorageExport struct
    pub async fn import_all(&self, data: StorageExport) -> Result<ImportExportStats> {
        self.import_with_options(data, &ImportOptions::all()).await
    }

    /// Import storage data with specific options
    /// Import cookies from export data
    async fn import_cookies_data(
        &self,
        cookies: &[Cookie],
        merge: bool,
    ) -> Result<usize> {
        if !merge {
            self.clear_cookies().await?;
        }
        for cookie in cookies {
            self.set_cookie(cookie.clone()).await?;
        }
        Ok(cookies.len())
    }

    /// Import history entries from export data
    async fn import_history_data(
        &self,
        history_entries: Vec<HistoryEntry>,
        merge: bool,
    ) -> Result<usize> {
        if !merge {
            self.clear_history().await?;
        }
        
        let mut history = self.history.write().await;
        let mut next_id = self.next_history_id.write().await;
        let count = history_entries.len();
        
        for mut entry in history_entries {
            if merge && history.contains_key(&entry.url) {
                self.merge_history_entry(&mut history, &entry);
            } else {
                entry.id = *next_id;
                *next_id += 1;
                history.insert(entry.url.clone(), entry);
            }
        }
        Ok(count)
    }

    /// Merge a history entry with an existing one
    fn merge_history_entry(
        &self,
        history: &mut HashMap<String, HistoryEntry>,
        entry: &HistoryEntry,
    ) {
        if let Some(existing) = history.get_mut(&entry.url) {
            existing.visit_count += entry.visit_count;
            if entry.last_visit > existing.last_visit {
                existing.last_visit = entry.last_visit;
                existing.title = entry.title.clone();
            }
        }
    }

    /// Import bookmarks from export data
    async fn import_bookmarks_data(
        &self,
        bookmarks_data: Vec<Bookmark>,
        merge: bool,
    ) -> Result<usize> {
        if !merge {
            self.bookmarks.write().await.clear();
        }
        
        let mut bookmarks = self.bookmarks.write().await;
        let mut next_id = self.next_bookmark_id.write().await;
        let count = bookmarks_data.len();
        
        for mut bookmark in bookmarks_data {
            if merge && bookmarks.values().any(|b| b.url == bookmark.url) {
                continue;
            }
            bookmark.id = *next_id;
            *next_id += 1;
            bookmarks.insert(bookmark.id, bookmark);
        }
        Ok(count)
    }

    /// Import local storage from export data
    async fn import_local_storage_data(
        &self,
        local_storage_data: HashMap<String, HashMap<String, String>>,
        merge: bool,
    ) -> Result<(usize, usize)> {
        if !merge {
            self.local_storage.write().await.clear();
        }
        
        let mut storage = self.local_storage.write().await;
        let mut items_count = 0;
        
        for (origin, items) in local_storage_data {
            items_count += items.len();
            if merge {
                let origin_storage = storage.entry(origin).or_insert_with(HashMap::new);
                origin_storage.extend(items);
            } else {
                storage.insert(origin, items);
            }
        }
        
        Ok((storage.len(), items_count))
    }

    /// Import storage data with specific options
    pub async fn import_with_options(
        &self,
        data: StorageExport,
        options: &ImportOptions,
    ) -> Result<ImportExportStats> {
        let mut stats = ImportExportStats::default();

        if options.import_cookies {
            stats.cookies_count = self.import_cookies_data(&data.cookies, options.merge).await?;
        }

        if options.import_history {
            stats.history_count = self.import_history_data(data.history, options.merge).await?;
        }

        if options.import_bookmarks {
            stats.bookmarks_count = self.import_bookmarks_data(data.bookmarks, options.merge).await?;
        }

        if options.import_local_storage {
            let (origins, items) = self.import_local_storage_data(data.local_storage, options.merge).await?;
            stats.local_storage_origins = origins;
            stats.local_storage_items = items;
        }

        info!(
            "Imported storage: {} cookies, {} history, {} bookmarks, {} local storage items",
            stats.cookies_count,
            stats.history_count,
            stats.bookmarks_count,
            stats.local_storage_items
        );

        Ok(stats)
    }

    /// Import storage data from a JSON file
    pub async fn import_from_file(&self, path: &Path) -> Result<ImportExportStats> {
        self.import_from_file_with_options(path, &ImportOptions::all()).await
    }

    /// Import storage data from a JSON file with specific options
    pub async fn import_from_file_with_options(
        &self,
        path: &Path,
        options: &ImportOptions,
    ) -> Result<ImportExportStats> {
        let json = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read import file")?;
        
        let data: StorageExport = serde_json::from_str(&json)
            .context("Failed to parse import file")?;

        info!(
            "Importing storage from file: {:?} (version: {}, exported: {})",
            path, data.version, data.exported_at
        );

        self.import_with_options(data, options).await
    }

    /// Import storage data from a JSON string
    pub async fn import_from_json(&self, json: &str) -> Result<ImportExportStats> {
        let data: StorageExport = serde_json::from_str(json)
            .context("Failed to parse JSON data")?;
        self.import_all(data).await
    }

    // =========================================================================
    // SELECTIVE EXPORT FUNCTIONS
    // =========================================================================

    /// Export only cookies to JSON
    pub async fn export_cookies_json(&self) -> Result<String> {
        let cookies = self.get_all_cookies().await?;
        serde_json::to_string_pretty(&cookies).context("Failed to serialize cookies")
    }

    /// Export only bookmarks to JSON
    pub async fn export_bookmarks_json(&self) -> Result<String> {
        let bookmarks = self.get_bookmarks().await?;
        serde_json::to_string_pretty(&bookmarks).context("Failed to serialize bookmarks")
    }

    /// Export only history to JSON
    pub async fn export_history_json(&self) -> Result<String> {
        let history = self.history.read().await;
        let entries: Vec<HistoryEntry> = history.values().cloned().collect();
        serde_json::to_string_pretty(&entries).context("Failed to serialize history")
    }

    // =========================================================================
    // SELECTIVE IMPORT FUNCTIONS
    // =========================================================================

    /// Import only cookies from JSON
    pub async fn import_cookies_json(&self, json: &str, merge: bool) -> Result<usize> {
        let cookies: Vec<Cookie> = serde_json::from_str(json)
            .context("Failed to parse cookies JSON")?;
        
        if !merge {
            self.clear_cookies().await?;
        }
        
        let count = cookies.len();
        for cookie in cookies {
            self.set_cookie(cookie).await?;
        }
        
        info!("Imported {} cookies", count);
        Ok(count)
    }

    /// Import only bookmarks from JSON
    pub async fn import_bookmarks_json(&self, json: &str, merge: bool) -> Result<usize> {
        let bookmarks: Vec<Bookmark> = serde_json::from_str(json)
            .context("Failed to parse bookmarks JSON")?;
        
        if !merge {
            self.bookmarks.write().await.clear();
        }
        
        let mut bm = self.bookmarks.write().await;
        let mut next_id = self.next_bookmark_id.write().await;
        let mut count = 0;
        
        for mut bookmark in bookmarks {
            if merge {
                let exists = bm.values().any(|b| b.url == bookmark.url);
                if exists {
                    continue;
                }
            }
            bookmark.id = *next_id;
            *next_id += 1;
            bm.insert(bookmark.id, bookmark);
            count += 1;
        }
        
        info!("Imported {} bookmarks", count);
        Ok(count)
    }

    /// Import only history from JSON
    pub async fn import_history_json(&self, json: &str, merge: bool) -> Result<usize> {
        let entries: Vec<HistoryEntry> = serde_json::from_str(json)
            .context("Failed to parse history JSON")?;
        
        if !merge {
            self.clear_history().await?;
        }
        
        let mut history = self.history.write().await;
        let mut next_id = self.next_history_id.write().await;
        let mut count = 0;
        
        for mut entry in entries {
            if merge && history.contains_key(&entry.url) {
                if let Some(existing) = history.get_mut(&entry.url) {
                    existing.visit_count += entry.visit_count;
                    if entry.last_visit > existing.last_visit {
                        existing.last_visit = entry.last_visit;
                        existing.title = entry.title;
                    }
                }
            } else {
                entry.id = *next_id;
                *next_id += 1;
                history.insert(entry.url.clone(), entry);
            }
            count += 1;
        }
        
        info!("Imported {} history entries", count);
        Ok(count)
    }

    // =========================================================================
    // COOKIE OPERATIONS
    // =========================================================================

    pub async fn set_cookie(&self, cookie: Cookie) -> Result<()> {
        let key = format!("{}|{}|{}", cookie.domain, cookie.name, cookie.path);
        self.cookies.write().await.insert(key, cookie);
        Ok(())
    }

    pub async fn get_cookies(&self, domain: &str) -> Result<Vec<Cookie>> {
        let cookies = self.cookies.read().await;
        let result: Vec<Cookie> = cookies
            .values()
            .filter(|c| c.domain.contains(domain) || domain.contains(&c.domain))
            .cloned()
            .collect();
        Ok(result)
    }

    pub async fn get_all_cookies(&self) -> Result<Vec<Cookie>> {
        let cookies = self.cookies.read().await;
        Ok(cookies.values().cloned().collect())
    }

    pub async fn delete_cookie(&self, domain: &str, name: &str, path: &str) -> Result<()> {
        let key = format!("{}|{}|{}", domain, name, path);
        self.cookies.write().await.remove(&key);
        Ok(())
    }

    pub async fn clear_cookies(&self) -> Result<()> {
        self.cookies.write().await.clear();
        Ok(())
    }

    // =========================================================================
    // HISTORY OPERATIONS
    // =========================================================================

    pub async fn add_history(&self, url: &str, title: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let mut history = self.history.write().await;
        
        if let Some(entry) = history.get_mut(url) {
            entry.visit_count += 1;
            entry.last_visit = now;
            if let Some(t) = title {
                entry.title = Some(t.to_string());
            }
        } else {
            let mut id_guard = self.next_history_id.write().await;
            let id = *id_guard;
            *id_guard += 1;
            
            history.insert(url.to_string(), HistoryEntry {
                id,
                url: url.to_string(),
                title: title.map(|t| t.to_string()),
                visit_count: 1,
                last_visit: now,
            });
        }
        Ok(())
    }

    pub async fn get_history(&self, limit: i64) -> Result<Vec<HistoryEntry>> {
        let history = self.history.read().await;
        let mut entries: Vec<HistoryEntry> = history.values().cloned().collect();
        entries.sort_by(|a, b| b.last_visit.cmp(&a.last_visit));
        entries.truncate(limit as usize);
        Ok(entries)
    }

    pub async fn search_history(&self, query: &str) -> Result<Vec<HistoryEntry>> {
        let history = self.history.read().await;
        let query_lower = query.to_lowercase();
        let mut entries: Vec<HistoryEntry> = history
            .values()
            .filter(|e| {
                e.url.to_lowercase().contains(&query_lower)
                    || e.title
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();
        entries.sort_by(|a, b| b.last_visit.cmp(&a.last_visit));
        entries.truncate(100);
        Ok(entries)
    }

    pub async fn clear_history(&self) -> Result<()> {
        self.history.write().await.clear();
        Ok(())
    }

    // =========================================================================
    // BOOKMARK OPERATIONS
    // =========================================================================

    pub async fn add_bookmark(&self, url: &str, title: &str, folder: Option<&str>) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();
        let mut id_guard = self.next_bookmark_id.write().await;
        let id = *id_guard;
        *id_guard += 1;
        
        let bookmark = Bookmark {
            id,
            url: url.to_string(),
            title: title.to_string(),
            folder: folder.map(|f| f.to_string()),
            created_at: now,
        };
        
        self.bookmarks.write().await.insert(id, bookmark);
        Ok(id)
    }

    pub async fn get_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let bookmarks = self.bookmarks.read().await;
        let mut entries: Vec<Bookmark> = bookmarks.values().cloned().collect();
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(entries)
    }

    pub async fn delete_bookmark(&self, id: i64) -> Result<()> {
        self.bookmarks.write().await.remove(&id);
        Ok(())
    }

    // =========================================================================
    // LOCAL STORAGE OPERATIONS
    // =========================================================================

    pub async fn set_local_storage(&self, origin: &str, key: &str, value: &str) -> Result<()> {
        let mut storage = self.local_storage.write().await;
        storage
            .entry(origin.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub async fn get_local_storage(&self, origin: &str, key: &str) -> Result<Option<String>> {
        let storage = self.local_storage.read().await;
        Ok(storage
            .get(origin)
            .and_then(|m| m.get(key))
            .cloned())
    }

    pub async fn get_all_local_storage(&self, origin: &str) -> Result<Vec<(String, String)>> {
        let storage = self.local_storage.read().await;
        Ok(storage
            .get(origin)
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default())
    }

    pub async fn clear_local_storage(&self, origin: &str) -> Result<()> {
        self.local_storage.write().await.remove(origin);
        Ok(())
    }

    pub async fn clear_all_local_storage(&self) -> Result<()> {
        self.local_storage.write().await.clear();
        Ok(())
    }

    // =========================================================================
    // UTILITY FUNCTIONS
    // =========================================================================

    pub fn db_path(&self) -> &Path {
        &self.data_dir
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> ImportExportStats {
        let cookies = self.cookies.read().await;
        let history = self.history.read().await;
        let bookmarks = self.bookmarks.read().await;
        let local_storage = self.local_storage.read().await;

        ImportExportStats {
            cookies_count: cookies.len(),
            history_count: history.len(),
            bookmarks_count: bookmarks.len(),
            local_storage_origins: local_storage.len(),
            local_storage_items: local_storage.values().map(|m| m.len()).sum(),
        }
    }

    /// Clear all storage data
    pub async fn clear_all(&self) -> Result<()> {
        self.clear_cookies().await?;
        self.clear_history().await?;
        self.bookmarks.write().await.clear();
        self.clear_all_local_storage().await?;
        
        // Reset IDs
        *self.next_history_id.write().await = 1;
        *self.next_bookmark_id.write().await = 1;
        
        info!("Cleared all storage data");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_storage() -> (StorageEngine, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageEngine::new(temp_dir.path()).unwrap();
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_export_import_all() {
        let (storage, _temp) = create_test_storage().await;

        // Add some data
        storage.set_cookie(Cookie {
            domain: "example.com".to_string(),
            name: "session".to_string(),
            value: "abc123".to_string(),
            path: "/".to_string(),
            expires: Some(9999999999),
            http_only: true,
            secure: true,
            same_site: "Lax".to_string(),
        }).await.unwrap();

        storage.add_history("https://example.com", Some("Example")).await.unwrap();
        storage.add_bookmark("https://example.com", "Example Site", None).await.unwrap();
        storage.set_local_storage("https://example.com", "key1", "value1").await.unwrap();

        // Export
        let export = storage.export_all().await.unwrap();
        assert_eq!(export.cookies.len(), 1);
        assert_eq!(export.history.len(), 1);
        assert_eq!(export.bookmarks.len(), 1);
        assert_eq!(export.local_storage.len(), 1);

        // Clear and import
        storage.clear_all().await.unwrap();
        let stats = storage.import_all(export).await.unwrap();
        
        assert_eq!(stats.cookies_count, 1);
        assert_eq!(stats.history_count, 1);
        assert_eq!(stats.bookmarks_count, 1);
        assert_eq!(stats.local_storage_items, 1);
    }

    #[tokio::test]
    async fn test_export_import_json() {
        let (storage, _temp) = create_test_storage().await;

        storage.add_bookmark("https://test.com", "Test", None).await.unwrap();
        
        let json = storage.export_to_json().await.unwrap();
        assert!(json.contains("test.com"));

        storage.clear_all().await.unwrap();
        
        let stats = storage.import_from_json(&json).await.unwrap();
        assert_eq!(stats.bookmarks_count, 1);
    }

    #[tokio::test]
    async fn test_selective_import() {
        let (storage, _temp) = create_test_storage().await;

        storage.set_cookie(Cookie {
            domain: "test.com".to_string(),
            name: "test".to_string(),
            value: "value".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
            same_site: "None".to_string(),
        }).await.unwrap();
        storage.add_bookmark("https://test.com", "Test", None).await.unwrap();

        let export = storage.export_all().await.unwrap();
        storage.clear_all().await.unwrap();

        // Import only bookmarks
        let options = ImportOptions {
            merge: false,
            import_cookies: false,
            import_history: false,
            import_bookmarks: true,
            import_local_storage: false,
        };
        
        storage.import_with_options(export, &options).await.unwrap();
        
        assert_eq!(storage.get_all_cookies().await.unwrap().len(), 0);
        assert_eq!(storage.get_bookmarks().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_merge_import() {
        let (storage, _temp) = create_test_storage().await;

        storage.add_bookmark("https://first.com", "First", None).await.unwrap();
        let export = storage.export_all().await.unwrap();

        storage.add_bookmark("https://second.com", "Second", None).await.unwrap();

        // Merge import
        let options = ImportOptions::all();
        storage.import_with_options(export, &options).await.unwrap();

        // Should have both bookmarks
        let bookmarks = storage.get_bookmarks().await.unwrap();
        assert_eq!(bookmarks.len(), 2);
    }
}

// =============================================================================
// Session Management System
// =============================================================================

/// Browser session data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSession {
    /// Unique session ID
    pub session_id: String,
    /// Session name (user-defined)
    pub name: String,
    /// Creation timestamp
    pub created_at: i64,
    /// Last accessed timestamp
    pub last_accessed: i64,
    /// Open tabs in this session
    pub tabs: Vec<SessionTab>,
    /// Window state
    pub window_state: WindowState,
    /// Session-specific settings
    pub settings: SessionSettings,
    /// Session tags for organization
    pub tags: Vec<String>,
    /// Whether this is the default session
    pub is_default: bool,
}

/// Tab information for session persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTab {
    /// Tab ID
    pub tab_id: String,
    /// Current URL
    pub url: String,
    /// Page title
    pub title: String,
    /// Favicon URL
    pub favicon_url: Option<String>,
    /// Tab position/index
    pub position: usize,
    /// Whether tab is pinned
    pub is_pinned: bool,
    /// Whether tab is muted
    pub is_muted: bool,
    /// Scroll position
    pub scroll_position: ScrollPosition,
    /// Navigation history for this tab
    pub history: Vec<TabHistoryEntry>,
    /// Current history index
    pub history_index: usize,
}

/// Scroll position for tab restoration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScrollPosition {
    pub x: i32,
    pub y: i32,
}

/// Tab navigation history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabHistoryEntry {
    pub url: String,
    pub title: String,
    pub timestamp: i64,
}

/// Window state for session restoration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_maximized: bool,
    pub is_fullscreen: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 1280,
            height: 720,
            is_maximized: false,
            is_fullscreen: false,
        }
    }
}

/// Session-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    /// Proxy configuration for this session
    pub proxy_config: Option<SessionProxyConfig>,
    /// User agent override
    pub user_agent: Option<String>,
    /// Whether to block ads in this session
    pub block_ads: bool,
    /// Whether to enable tracking protection
    pub tracking_protection: bool,
    /// Zoom level (1.0 = 100%)
    pub zoom_level: f64,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            proxy_config: None,
            user_agent: None,
            block_ads: true,
            tracking_protection: true,
            zoom_level: 1.0,
        }
    }
}

/// Proxy configuration for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProxyConfig {
    pub enabled: bool,
    pub proxy_type: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// Session manager for handling browser sessions
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, BrowserSession>>>,
    active_session_id: Arc<RwLock<Option<String>>>,
    storage_path: PathBuf,
    auto_save_enabled: bool,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            active_session_id: Arc::new(RwLock::new(None)),
            storage_path,
            auto_save_enabled: true,
        }
    }

    /// Create a new session
    pub async fn create_session(&self, name: &str, is_default: bool) -> Result<BrowserSession> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        let session = BrowserSession {
            session_id: session_id.clone(),
            name: name.to_string(),
            created_at: now,
            last_accessed: now,
            tabs: Vec::new(),
            window_state: WindowState::default(),
            settings: SessionSettings::default(),
            tags: Vec::new(),
            is_default,
        };

        let mut sessions = self.sessions.write().await;
        
        // If this is default, unmark other default sessions
        if is_default {
            for s in sessions.values_mut() {
                s.is_default = false;
            }
        }

        sessions.insert(session_id.clone(), session.clone());
        info!("Created new session: {} ({})", name, session_id);

        if self.auto_save_enabled {
            self.save_sessions_internal(&sessions).await?;
        }

        Ok(session)
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<BrowserSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Get the active session
    pub async fn get_active_session(&self) -> Option<BrowserSession> {
        let active_id = self.active_session_id.read().await;
        if let Some(id) = active_id.as_ref() {
            self.get_session(id).await
        } else {
            None
        }
    }

    /// Set the active session
    pub async fn set_active_session(&self, session_id: &str) -> Result<()> {
        let sessions = self.sessions.read().await;
        if !sessions.contains_key(session_id) {
            return Err(anyhow::anyhow!("Session not found"));
        }
        drop(sessions);

        let mut active_id = self.active_session_id.write().await;
        *active_id = Some(session_id.to_string());

        // Update last accessed
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_accessed = chrono::Utc::now().timestamp();
        }

        Ok(())
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<BrowserSession> {
        let sessions = self.sessions.read().await;
        let mut list: Vec<BrowserSession> = sessions.values().cloned().collect();
        list.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        list
    }

    /// Update a session
    pub async fn update_session(&self, session: BrowserSession) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if !sessions.contains_key(&session.session_id) {
            return Err(anyhow::anyhow!("Session not found"));
        }

        sessions.insert(session.session_id.clone(), session);

        if self.auto_save_enabled {
            self.save_sessions_internal(&sessions).await?;
        }

        Ok(())
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<bool> {
        let mut sessions = self.sessions.write().await;
        let removed = sessions.remove(session_id).is_some();

        if removed {
            // Clear active session if it was deleted
            let mut active_id = self.active_session_id.write().await;
            if active_id.as_ref() == Some(&session_id.to_string()) {
                *active_id = None;
            }

            info!("Deleted session: {}", session_id);

            if self.auto_save_enabled {
                self.save_sessions_internal(&sessions).await?;
            }
        }

        Ok(removed)
    }

    /// Add a tab to a session
    pub async fn add_tab_to_session(&self, session_id: &str, tab: SessionTab) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        session.tabs.push(tab);
        session.last_accessed = chrono::Utc::now().timestamp();

        if self.auto_save_enabled {
            self.save_sessions_internal(&sessions).await?;
        }

        Ok(())
    }

    /// Remove a tab from a session
    pub async fn remove_tab_from_session(&self, session_id: &str, tab_id: &str) -> Result<bool> {
        let mut sessions = self.sessions.write().await;
        
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        let initial_len = session.tabs.len();
        session.tabs.retain(|t| t.tab_id != tab_id);
        let removed = session.tabs.len() < initial_len;

        if removed {
            session.last_accessed = chrono::Utc::now().timestamp();

            if self.auto_save_enabled {
                self.save_sessions_internal(&sessions).await?;
            }
        }

        Ok(removed)
    }

    /// Update a tab in a session
    pub async fn update_tab_in_session(&self, session_id: &str, tab: SessionTab) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        if let Some(existing_tab) = session.tabs.iter_mut().find(|t| t.tab_id == tab.tab_id) {
            *existing_tab = tab;
            session.last_accessed = chrono::Utc::now().timestamp();

            if self.auto_save_enabled {
                self.save_sessions_internal(&sessions).await?;
            }
        }

        Ok(())
    }

    /// Save all sessions to disk
    pub async fn save_sessions(&self) -> Result<()> {
        let sessions = self.sessions.read().await;
        self.save_sessions_internal(&sessions).await
    }

    /// Internal save method
    async fn save_sessions_internal(&self, sessions: &HashMap<String, BrowserSession>) -> Result<()> {
        let sessions_file = self.storage_path.join("sessions.json");
        
        // Ensure directory exists
        if let Some(parent) = sessions_file.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let json = serde_json::to_string_pretty(sessions)?;
        tokio::fs::write(&sessions_file, json).await
            .context("Failed to write sessions file")?;

        Ok(())
    }

    /// Load sessions from disk
    pub async fn load_sessions(&self) -> Result<usize> {
        let sessions_file = self.storage_path.join("sessions.json");
        
        if !sessions_file.exists() {
            return Ok(0);
        }

        let json = tokio::fs::read_to_string(&sessions_file).await
            .context("Failed to read sessions file")?;

        let loaded: HashMap<String, BrowserSession> = serde_json::from_str(&json)
            .context("Failed to parse sessions file")?;

        let count = loaded.len();
        let mut sessions = self.sessions.write().await;
        *sessions = loaded;

        // Set active session to default or most recently accessed
        let active_id = sessions.values()
            .filter(|s| s.is_default)
            .next()
            .or_else(|| sessions.values().max_by_key(|s| s.last_accessed))
            .map(|s| s.session_id.clone());

        if let Some(id) = active_id {
            let mut active = self.active_session_id.write().await;
            *active = Some(id);
        }

        info!("Loaded {} sessions", count);
        Ok(count)
    }

    /// Export a session to JSON
    pub async fn export_session(&self, session_id: &str) -> Result<String> {
        let session = self.get_session(session_id).await
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        serde_json::to_string_pretty(&session)
            .context("Failed to serialize session")
    }

    /// Import a session from JSON
    pub async fn import_session(&self, json: &str) -> Result<String> {
        let mut session: BrowserSession = serde_json::from_str(json)
            .context("Failed to parse session JSON")?;

        // Generate new session ID to avoid conflicts
        session.session_id = uuid::Uuid::new_v4().to_string();
        session.last_accessed = chrono::Utc::now().timestamp();

        let session_id = session.session_id.clone();
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        if self.auto_save_enabled {
            self.save_sessions_internal(&sessions).await?;
        }

        Ok(session_id)
    }

    /// Get the default session or create one
    pub async fn get_or_create_default_session(&self) -> Result<BrowserSession> {
        let sessions = self.sessions.read().await;
        
        if let Some(default) = sessions.values().find(|s| s.is_default) {
            return Ok(default.clone());
        }

        drop(sessions);
        self.create_session("Default Session", true).await
    }

    /// Duplicate a session
    pub async fn duplicate_session(&self, session_id: &str, new_name: &str) -> Result<BrowserSession> {
        let original = self.get_session(session_id).await
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        let now = chrono::Utc::now().timestamp();
        let new_session = BrowserSession {
            session_id: uuid::Uuid::new_v4().to_string(),
            name: new_name.to_string(),
            created_at: now,
            last_accessed: now,
            tabs: original.tabs.clone(),
            window_state: original.window_state.clone(),
            settings: original.settings.clone(),
            tags: original.tags.clone(),
            is_default: false,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(new_session.session_id.clone(), new_session.clone());

        if self.auto_save_enabled {
            self.save_sessions_internal(&sessions).await?;
        }

        Ok(new_session)
    }

    /// Search sessions by name or tag
    pub async fn search_sessions(&self, query: &str) -> Vec<BrowserSession> {
        let sessions = self.sessions.read().await;
        let query_lower = query.to_lowercase();

        sessions.values()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower) ||
                s.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Get session statistics
    pub async fn get_statistics(&self) -> SessionStatistics {
        let sessions = self.sessions.read().await;
        
        let total_sessions = sessions.len();
        let total_tabs: usize = sessions.values().map(|s| s.tabs.len()).sum();
        let pinned_tabs: usize = sessions.values()
            .flat_map(|s| s.tabs.iter())
            .filter(|t| t.is_pinned)
            .count();

        SessionStatistics {
            total_sessions,
            total_tabs,
            pinned_tabs,
            active_session_id: self.active_session_id.read().await.clone(),
        }
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub total_sessions: usize,
    pub total_tabs: usize,
    pub pinned_tabs: usize,
    pub active_session_id: Option<String>,
}

#[cfg(test)]
mod session_tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_session_manager() -> (SessionManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path().to_path_buf());
        (manager, temp_dir)
    }

    #[tokio::test]
    async fn test_create_session() {
        let (manager, _temp) = create_test_session_manager().await;
        
        let session = manager.create_session("Test Session", false).await.unwrap();
        assert_eq!(session.name, "Test Session");
        assert!(!session.is_default);
    }

    #[tokio::test]
    async fn test_default_session() {
        let (manager, _temp) = create_test_session_manager().await;
        
        let session = manager.get_or_create_default_session().await.unwrap();
        assert!(session.is_default);
    }

    #[tokio::test]
    async fn test_add_tab() {
        let (manager, _temp) = create_test_session_manager().await;
        
        let session = manager.create_session("Test", false).await.unwrap();
        
        let tab = SessionTab {
            tab_id: "tab1".to_string(),
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            favicon_url: None,
            position: 0,
            is_pinned: false,
            is_muted: false,
            scroll_position: ScrollPosition::default(),
            history: vec![],
            history_index: 0,
        };
        
        manager.add_tab_to_session(&session.session_id, tab).await.unwrap();
        
        let updated = manager.get_session(&session.session_id).await.unwrap();
        assert_eq!(updated.tabs.len(), 1);
    }

    #[tokio::test]
    async fn test_session_persistence() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create and save
        {
            let manager = SessionManager::new(temp_dir.path().to_path_buf());
            manager.create_session("Persistent Session", true).await.unwrap();
        }
        
        // Load in new manager
        {
            let manager = SessionManager::new(temp_dir.path().to_path_buf());
            let count = manager.load_sessions().await.unwrap();
            assert_eq!(count, 1);
            
            let sessions = manager.list_sessions().await;
            assert_eq!(sessions[0].name, "Persistent Session");
        }
    }

    #[tokio::test]
    async fn test_duplicate_session() {
        let (manager, _temp) = create_test_session_manager().await;
        
        let original = manager.create_session("Original", false).await.unwrap();
        let duplicate = manager.duplicate_session(&original.session_id, "Copy").await.unwrap();
        
        assert_ne!(original.session_id, duplicate.session_id);
        assert_eq!(duplicate.name, "Copy");
    }
}
