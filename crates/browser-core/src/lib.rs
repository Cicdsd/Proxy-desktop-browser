pub mod prelude;

// Re-export prelude utilities for easier access
pub use prelude::{
    OptionExt, ResultExt, unix_timestamp, unix_timestamp_ms,
    RetryConfig, retry_async, RateLimiter,
    CircuitBreaker, CircuitState,
    MetricsCollector, HistogramStats, MetricsSnapshot,
    validators, string_utils
};

pub use browser_profile::{BrowserProfile, BrowserProfileManager, ProfileSettings};
pub use screenshot::{ScreenshotManager, ScreenshotOptions, ScreenshotFormat, ScreenshotResult};

pub mod tab_manager;
pub mod config_manager;

pub mod tab_isolation;
pub mod fingerprint;
pub mod proxy;
pub mod http_client;
pub mod scraper_util;
pub mod security;
pub mod webview_manager;
pub mod browser_tab_manager;
pub mod free_ip_providers;
pub mod storage;
pub mod backup;
pub mod browser_controls;
pub mod local_proxy;
pub mod pac_server;
pub mod proxy_rotation;
pub mod proxy_validator;
pub mod chromium_engine;
pub mod ad_verification;

// V1000 Feature Modules - Phase 1: Foundation & Stability (v3-v10)
pub mod memory_profiler;
pub mod error_recovery;
pub mod performance_optimizer;

// V1000 Feature Modules - Phase 2: Feature Expansion (v10-v100)
pub mod network_intelligence;
pub mod privacy_fortress;

// V1000 Experimental Features (EXP-1001 to EXP-20005)
pub mod experimental;
pub mod efficiency;

// V1000 Additional Feature Modules
pub mod automation;
pub mod content_enhancement;
pub mod efficiency;
pub mod browser_profile;
pub mod screenshot;

// Database removed - using in-memory storage

// Configuration management exports
pub use config_manager::{
    ConfigManager, AppConfig,
    GeneralConfig, ProxyConfig, PrivacyConfig, PerformanceConfig,
    NetworkConfig as AppNetworkConfig, StorageConfig, LoggingConfig, FeatureFlags
};

pub use tab_manager::TabIPManager;
pub use tab_isolation::{TabProfile, NetworkConfig, TabStatus};
pub use fingerprint::BrowserFingerprint;
pub use proxy::{ProxyManager, ProxySettings, ProxyType, FreeProxy, ProxyTestResult};
pub use http_client::{HttpClient, PublicIpDetector, PublicIpInfo};
pub use scraper_util::ProxyScraper;
pub use security::{SecurityManager, BookmarkInput, ProxyInput};
pub use webview_manager::{WebviewManager, WebviewTab};
pub use browser_tab_manager::{BrowserTabManager, BrowserTab, CreateTabConfig, TabStats};
pub use free_ip_providers::{FreeIpProvider, FreeIpProviderManager, ProxyFilter};
pub use storage::{
    SessionManager, BrowserSession, SessionTab, SessionSettings, SessionProxyConfig,
    WindowState, ScrollPosition, TabHistoryEntry, SessionStatistics,
    StorageEngine,
    Cookie,
    HistoryEntry,
    Bookmark,
    StorageExport,
    ImportOptions,
    ExportOptions,
    ImportExportStats
};
pub use backup::{BackupManager, BackupData, BackupOptions, BackupInfo, AutoBackupSettings};
pub use browser_controls::{
    BrowserController, BrowserState, BrowserSettings, WebRtcPolicy, HistoryItem,
    DownloadManager, DownloadItem, DownloadState,
    ContextMenuManager, ContextMenuItem, ContextMenuItemType, ContextType, ContextInfo
};
pub use local_proxy::{
    LocalProxyServer, LocalProxyManager, ProxyConnection,
    WebSocketProxyHandler, WebSocketInterception,
    NetworkInterceptor, InterceptedRequest, ModificationRule, RequestModifications
};
pub use pac_server::{PacServer, PacManager};
pub use proxy_rotation::{
    ProxyRotationManager, ProxyRotationStrategy, ProxyMetrics, ProxySessionStats,
    SmartProxySelector, ProxyHealthMonitor, ProxyHealthStatus, BandwidthStats, GeoDiversityManager
};
pub use proxy_validator::{
    ProxyValidator, ProxyValidatorConfig, ValidationResult, ProxyHealthChecker,
    ProxyQuarantineManager, QuarantinedProxy, QuarantineStats,
    GeoVerifier, GeoVerificationConfig, GeoVerificationResult,
    EnhancedProxyHealthChecker
};
pub use chromium_engine::{
    ChromiumEngine,
    ChromiumEngineConfig,
    BrowserEngineManager,
    BrowserEngineType,
    ChromiumTab,
    EngineCapabilities
};
pub use ad_verification::{
    AdVerificationManager,
    AdVerificationConfig,
    ImpressionVerification,
    ImpressionData,
    VastVerification,
    VpaidVerification,
    VerificationSession,
    SessionStats,
    ViewabilityStatus,
    FraudSignal,
    AdFormat,
    VerificationStandard
};

// V1000 Phase 1 exports
pub use memory_profiler::{
    MemoryProfiler,
    MemorySnapshot,
    MemoryStats,
    MemoryThresholds,
    MemoryAlert,
    LeakReport,
    GcRecommendation
};
pub use error_recovery::{
    ErrorRecoveryManager,
    ErrorRecoveryConfig,
    ErrorCategory,
    RecoveryStrategy,
    RecoveryResult,
    ErrorStats,
    CrashPrediction
};
pub use performance_optimizer::{
    PerformanceOptimizer,
    PerformanceConfig,
    PerformanceReport,
    CoreWebVitals,
    CacheStats,
    CachePriority
};

// V1000 Phase 2 exports
pub use network_intelligence::{
    NetworkIntelligence,
    NetworkIntelligenceConfig,
    NetworkIntelligenceReport,
    TrafficReport,
    QosPriority
};
pub use privacy_fortress::{
    PrivacyFortress,
    PrivacyConfig,
    PrivacyReport,
    PrivacyGrade,
    TrackerStats,
    CookieIsolationLevel
};

// V1000 Experimental Feature exports
pub use experimental::{
    ExperimentalFeaturesManager, ExperimentalFeatureInfo,
    // Core Engine Experiments
    MultiEngineSystem, EngineType, ProcessIsolationConfig, SandboxLevel, WasiBrowser, WasiCapabilities,
    // Network Experiments
    MeshProxyNetwork, MeshNode, OnionRouter, DnsResolver, DnsStrategy,
    // Security Experiments
    ZkAuthSystem, SecureEnclaveManager, EnclaveType, MemorySafeSandbox, BehavioralAuth,
    // Privacy Experiments
    DifferentialPrivacy, DecoyTrafficGenerator, AntiCorrelationSystem,
    // AI/ML Experiments
    LocalLlm, VisualEngine, IntelligentFormFiller,
    // Rendering Experiments
    GpuRenderConfig, Spatial3DConfig, AdaptiveRenderer,
    // Cryptography Experiments
    PostQuantumCrypto, PqcAlgorithm,
    // Decentralized Experiments
    IpfsBrowser, BlockchainDns, DecentralizedIdentity,
    // Quantum Experiments
    QuantumRng,
};

// V1000 Automation System exports
pub use automation::{
    AutomationManager, AutomationStats, 
    VisualAutomationBuilder, Workflow, AutomationStep, StepType,
    NaturalLanguageAutomation, ActionRecorder, RecordedAction, ActionType,
    DistributedAutomation, AutomationNode, NodeStatus, DistributedTask, TaskStatus,
};

// V1000 Content Enhancement exports
pub use content_enhancement::{
    AdvancedLanguageDetector, LanguageProfile, LanguageDetectionResult, ScriptType,
    TextAnalyzer, TextStatistics,
    ContentEnhancementManager,
    ReaderMode, ReaderModeConfig, ReaderTheme, ExtractedArticle,
    MediaPlayer, MediaPlayerConfig, MediaInfo, MediaType, VideoQuality,
    ContentTransformer, TransformationType,
    AccessibilityManager, AccessibilityConfig, ColorBlindnessMode,
};

// Efficiency optimization exports (addressing issues #488-#987)
pub use efficiency::{
    MemoryPool,
    PoolStats,
    BufferManager,
    CpuOptimizer,
    LazyInit,
    PerformanceMonitor,
    PerformanceMetrics,
    LruCache,
    ConnectionPool,
    BatchProcessor,
    ResourceManager,
};
