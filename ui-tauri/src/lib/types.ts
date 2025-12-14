export type Tab = {
  tab_id: string;
  ip: string;
  country_code: string;
  country_name: string;
  city: string;
  timezone: string;
  isp: string;
};


export type VirtualIPResponse = {
  ip: string;
  country_code: string;
  country_name: string;
  city: string;
  region: string;
  timezone: string;
  language: string;
  currency: string;
  isp: string;
};

export type ValidationResponse = {
  ip: string;
  ip_matches: boolean;
  webrtc_secure: boolean;
  dns_secure: boolean;
  overall_pass: boolean;
};

export type User = {
  id: string;
  username: string;
  email?: string;
  role?: string;
  enterpriseId?: string;
};

export type Country = {
  code: string;
  name: string;
  flag: string;
  timezone: string;
  language: string;
  currency: string;
  is_top: boolean;
};

export type ProxySettings = {
  proxy_type: 'direct' | 'http' | 'https' | 'socks4' | 'socks5';
  host: string | null;
  port: number | null;
  username: string | null;
  password: string | null;
  dns_servers: string[];
  bypass_list: string[];
  enabled?: boolean;
  type?: 'http' | 'https' | 'socks4' | 'socks5' | 'direct';
};

export type ProxyStatus = {
  isConnected: boolean;
  currentIp: string;
  currentCountry: string;
  latency: number;
  bytesDownloaded: number;
  bytesUploaded: number;
  proxyServer: string;
  lastChecked: string;
};

export type FreeProxy = {
  ip: string;
  port: number;
  protocol: 'http' | 'https' | 'socks4' | 'socks5' | 'direct';
  country: string;
  country_code: string;
  anonymity: string;
  speed: number;
  uptime: number;
  last_checked: string;
  provider: string;
  is_working: boolean;
};

export type ProxyTestResult = {
  proxy: FreeProxy;
  is_working: boolean;
  latency_ms: number | null;
  detected_ip: string | null;
  error: string | null;
};

export type PublicIpInfo = {
  ip: string;
  country: string | null;
  country_code: string | null;
  city: string | null;
  region: string | null;
  isp: string | null;
  timezone: string | null;
};

export type BackupOptions = {
  include_proxy_settings: boolean;
  include_browser_config: boolean;
  include_cookies: boolean;
  include_history: boolean;
  include_bookmarks: boolean;
  include_local_storage: boolean;
  password: string | null;
};

export type BackupInfo = {
  id: string;
  filename: string;
  path: string;
  created_at: string;
  size_bytes: number;
  is_encrypted: boolean;
};

export type BrowserState = {
  tab_id: string;
  current_url: string;
  title: string;
  can_go_back: boolean;
  can_go_forward: boolean;
  is_loading: boolean;
};

export type BrowserSettings = {
  user_agent: string;
  language: string;
  timezone: string;
  webrtc_policy: 'default' | 'disable_non_proxied_udp' | 'disabled';
  dns_over_https: boolean;
  block_trackers: boolean;
  block_ads: boolean;
  javascript_enabled: boolean;
  cookies_enabled: boolean;
  engine_type: 'system' | 'integrated_chromium';
  stealth_mode: boolean;
  headless_mode: boolean;
};

export type HistoryEntry = {
  id: number;
  url: string;
  title: string | null;
  visit_count: number;
  last_visit: number;
};

export type Bookmark = {
  id: number;
  url: string;
  title: string;
  folder: string | null;
  created_at: number;
};

export type WebviewTab = {
  tab_id: string;
  window_label: string;
  url: string;
  title: string;
  is_loading: boolean;
  can_go_back: boolean;
  can_go_forward: boolean;
  created_at: number;
  proxy_url?: string;
  country_code?: string;
  ip_address?: string;
};
