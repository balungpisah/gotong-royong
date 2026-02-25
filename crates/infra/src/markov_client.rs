use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use reqwest::StatusCode;
use serde_json::Value;
use tokio::sync::{Mutex, Notify, RwLock};
use tokio::time::sleep;
use uuid::Uuid;

use crate::config::AppConfig;

const PLATFORM_TOKEN_HEADER: &str = "X-Platform-Token";
const PLATFORM_ID_PREFIX: &str = "gotong_royong:";
const SCOPE_QUERY_KEY: &str = "view_scope";
const SCOPE_QUERY_PLATFORM_VALUE: &str = "platform";
const SCOPE_QUERY_PLATFORM_ID_KEY: &str = "platform_id";
const MARKOV_CACHE_MAX_ENTRIES: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStatus {
    Hit,
    Miss,
    Stale,
}

impl CacheStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            CacheStatus::Hit => "hit",
            CacheStatus::Miss => "miss",
            CacheStatus::Stale => "stale",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheMetadata {
    pub status: CacheStatus,
    pub stale: bool,
    pub age_ms: u64,
    pub cached_at_epoch_ms: i64,
}

#[derive(Debug, Clone)]
pub struct CachedJson {
    pub value: Value,
    pub meta: CacheMetadata,
}

#[derive(Debug, Clone)]
pub struct MarkovProfileSnapshot {
    pub identity: String,
    pub markov_user_id: Option<String>,
    pub reputation: CachedJson,
    pub tier: Option<CachedJson>,
    pub activity: Option<CachedJson>,
    pub cv_hidup: Option<CachedJson>,
}

#[derive(Debug, thiserror::Error)]
pub enum MarkovClientError {
    #[error("markov read client configuration error: {0}")]
    Configuration(String),
    #[error("markov read circuit is open")]
    CircuitOpen,
    #[error("markov bad request: {0}")]
    BadRequest(String),
    #[error("markov unauthorized: {0}")]
    Unauthorized(String),
    #[error("markov forbidden: {0}")]
    Forbidden(String),
    #[error("markov not found: {0}")]
    NotFound(String),
    #[error("markov upstream error: {0}")]
    Upstream(String),
    #[error("markov transport error: {0}")]
    Transport(String),
    #[error("markov response decode error: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Clone, Copy)]
enum CacheClass {
    Profile,
    Gameplay,
}

impl CacheClass {
    fn ttl(self, client: &MarkovReadClient) -> Duration {
        match self {
            CacheClass::Profile => client.profile_ttl,
            CacheClass::Gameplay => client.gameplay_ttl,
        }
    }

    fn stale_window(self, client: &MarkovReadClient) -> Duration {
        match self {
            CacheClass::Profile => client.profile_stale_window,
            CacheClass::Gameplay => client.gameplay_stale_window,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum TokenPolicy {
    Required,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: Value,
    cached_at_instant: Instant,
    cached_at_epoch_ms: i64,
    fresh_until: Instant,
    stale_until: Instant,
}

impl CacheEntry {
    fn new(value: Value, now: Instant, ttl: Duration, stale_window: Duration) -> Self {
        let safe_stale_window = stale_window.max(ttl);
        Self {
            value,
            cached_at_instant: now,
            cached_at_epoch_ms: now_epoch_ms(),
            fresh_until: now + ttl,
            stale_until: now + safe_stale_window,
        }
    }

    fn into_cached_json(&self, status: CacheStatus, now: Instant) -> CachedJson {
        let age_ms = now
            .checked_duration_since(self.cached_at_instant)
            .unwrap_or_default()
            .as_millis() as u64;
        CachedJson {
            value: self.value.clone(),
            meta: CacheMetadata {
                status,
                stale: matches!(status, CacheStatus::Stale),
                age_ms,
                cached_at_epoch_ms: self.cached_at_epoch_ms,
            },
        }
    }
}

#[derive(Debug, Default)]
struct CircuitState {
    consecutive_failures: u32,
    open_until: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct MarkovReadClient {
    http: reqwest::Client,
    base_url: String,
    platform_token: Option<String>,
    retry_max_attempts: u32,
    retry_backoff_base: Duration,
    retry_backoff_max: Duration,
    circuit_fail_threshold: u32,
    circuit_open_duration: Duration,
    profile_ttl: Duration,
    profile_stale_window: Duration,
    gameplay_ttl: Duration,
    gameplay_stale_window: Duration,
    read_scope_query_params: Vec<(String, String)>,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    revalidating_keys: Arc<Mutex<HashSet<String>>>,
    inflight_fetches: Arc<Mutex<HashMap<String, Arc<Notify>>>>,
    cache_max_entries: usize,
    circuit: Arc<Mutex<CircuitState>>,
}

impl MarkovReadClient {
    pub fn from_config(config: &AppConfig) -> Self {
        let timeout = Duration::from_millis(config.markov_read_timeout_ms.max(1));
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let token = config.markov_read_platform_token.trim().to_string();
        let read_scope_query_params = if config.markov_read_explicit_scope_query_enabled {
            vec![
                (
                    SCOPE_QUERY_KEY.to_string(),
                    SCOPE_QUERY_PLATFORM_VALUE.to_string(),
                ),
                (
                    SCOPE_QUERY_PLATFORM_ID_KEY.to_string(),
                    config.markov_read_platform_id.trim().to_string(),
                ),
            ]
        } else {
            Vec::new()
        };
        Self {
            http,
            base_url: config
                .markov_read_base_url
                .trim_end_matches('/')
                .to_string(),
            platform_token: if token.is_empty() { None } else { Some(token) },
            retry_max_attempts: config.markov_read_retry_max_attempts.max(1),
            retry_backoff_base: Duration::from_millis(config.markov_read_retry_backoff_base_ms),
            retry_backoff_max: Duration::from_millis(config.markov_read_retry_backoff_max_ms),
            circuit_fail_threshold: config.markov_read_circuit_fail_threshold.max(1),
            circuit_open_duration: Duration::from_millis(config.markov_read_circuit_open_ms.max(1)),
            profile_ttl: Duration::from_millis(config.markov_cache_profile_ttl_ms.max(1)),
            profile_stale_window: Duration::from_millis(
                config
                    .markov_cache_profile_stale_while_revalidate_ms
                    .max(config.markov_cache_profile_ttl_ms.max(1)),
            ),
            gameplay_ttl: Duration::from_millis(config.markov_cache_gameplay_ttl_ms.max(1)),
            gameplay_stale_window: Duration::from_millis(
                config
                    .markov_cache_gameplay_stale_while_revalidate_ms
                    .max(config.markov_cache_gameplay_ttl_ms.max(1)),
            ),
            read_scope_query_params,
            cache: Arc::new(RwLock::new(HashMap::new())),
            revalidating_keys: Arc::new(Mutex::new(HashSet::new())),
            inflight_fetches: Arc::new(Mutex::new(HashMap::new())),
            cache_max_entries: MARKOV_CACHE_MAX_ENTRIES,
            circuit: Arc::new(Mutex::new(CircuitState::default())),
        }
    }

    pub fn platform_identity(user_id: &str) -> String {
        format!("{PLATFORM_ID_PREFIX}{user_id}")
    }

    fn normalize_user_selector(platform_user_id: &str) -> String {
        let trimmed = platform_user_id.trim();
        if trimmed.contains(':') {
            return trimmed.to_string();
        }
        if Uuid::parse_str(trimmed).is_ok() {
            return trimmed.to_string();
        }
        Self::platform_identity(trimmed)
    }

    fn with_scope_query(&self, mut query_params: Vec<(String, String)>) -> Vec<(String, String)> {
        if self.read_scope_query_params.is_empty() {
            return query_params;
        }
        query_params.extend(self.read_scope_query_params.iter().cloned());
        query_params
    }

    pub async fn get_user_reputation(
        &self,
        gotong_user_id: &str,
    ) -> Result<CachedJson, MarkovClientError> {
        let user_id = gotong_user_id.trim();
        if user_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "user_id must not be empty".to_string(),
            ));
        }
        let identity = Self::platform_identity(user_id);
        self.get_user_reputation_by_identity(&identity).await
    }

    pub async fn get_user_reputation_by_identity(
        &self,
        identity: &str,
    ) -> Result<CachedJson, MarkovClientError> {
        let path = format!("users/{identity}/reputation");
        self.fetch_cached_json(
            path,
            self.with_scope_query(Vec::new()),
            CacheClass::Profile,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_user_tier(
        &self,
        markov_user_id: &str,
    ) -> Result<CachedJson, MarkovClientError> {
        let path = format!("users/{markov_user_id}/tier");
        self.fetch_cached_json(
            path,
            self.with_scope_query(Vec::new()),
            CacheClass::Profile,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_user_activity(
        &self,
        markov_user_id: &str,
    ) -> Result<CachedJson, MarkovClientError> {
        let path = format!("users/{markov_user_id}/activity");
        self.fetch_cached_json(
            path,
            self.with_scope_query(Vec::new()),
            CacheClass::Profile,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_cv_hidup(
        &self,
        markov_user_id: &str,
    ) -> Result<CachedJson, MarkovClientError> {
        let path = format!("cv-hidup/{markov_user_id}");
        self.fetch_cached_json(
            path,
            self.with_scope_query(Vec::new()),
            CacheClass::Profile,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_cv_hidup_qr(
        &self,
        platform_user_id: &str,
    ) -> Result<CachedJson, MarkovClientError> {
        let platform_user_id = platform_user_id.trim();
        if platform_user_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "platform_user_id must not be empty".to_string(),
            ));
        }
        let identity = Self::normalize_user_selector(platform_user_id);
        let path = format!("cv-hidup/{identity}/qr");
        self.fetch_cached_json(
            path,
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn post_cv_hidup_export(
        &self,
        platform_user_id: &str,
        payload: Value,
    ) -> Result<CachedJson, MarkovClientError> {
        let platform_user_id = platform_user_id.trim();
        if platform_user_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "platform_user_id must not be empty".to_string(),
            ));
        }
        let identity = Self::normalize_user_selector(platform_user_id);
        let path = format!("cv-hidup/{identity}/export");
        let value = self
            .post_from_origin(&path, payload, TokenPolicy::Required)
            .await?;
        Ok(uncached_json(value))
    }

    pub async fn get_cv_hidup_verify(
        &self,
        export_id: &str,
    ) -> Result<CachedJson, MarkovClientError> {
        let export_id = export_id.trim();
        if export_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "export_id must not be empty".to_string(),
            ));
        }
        let path = format!("cv-hidup/verify/{export_id}");
        self.fetch_cached_json(
            path,
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn search_skills(
        &self,
        query: &str,
        lang: Option<&str>,
        fuzzy: Option<bool>,
        limit: Option<u32>,
    ) -> Result<CachedJson, MarkovClientError> {
        let mut params = vec![("q".to_string(), query.trim().to_string())];
        if let Some(lang) = lang {
            let lang = lang.trim();
            if !lang.is_empty() {
                params.push(("lang".to_string(), lang.to_string()));
            }
        }
        if let Some(fuzzy) = fuzzy {
            params.push(("fuzzy".to_string(), fuzzy.to_string()));
        }
        if let Some(limit) = limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        self.fetch_cached_json(
            "skills/search".to_string(),
            params,
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_skill_node(&self, node_id: &str) -> Result<CachedJson, MarkovClientError> {
        let node_id = node_id.trim();
        if node_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "node_id must not be empty".to_string(),
            ));
        }
        let path = format!("skills/nodes/{node_id}");
        self.fetch_cached_json(
            path,
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_skill_node_labels(
        &self,
        node_id: &str,
    ) -> Result<CachedJson, MarkovClientError> {
        let node_id = node_id.trim();
        if node_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "node_id must not be empty".to_string(),
            ));
        }
        let path = format!("skills/nodes/{node_id}/labels");
        self.fetch_cached_json(
            path,
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_skill_node_relations(
        &self,
        node_id: &str,
    ) -> Result<CachedJson, MarkovClientError> {
        let node_id = node_id.trim();
        if node_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "node_id must not be empty".to_string(),
            ));
        }
        let path = format!("skills/nodes/{node_id}/relations");
        self.fetch_cached_json(
            path,
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_skill_parent(&self, skill_id: &str) -> Result<CachedJson, MarkovClientError> {
        let skill_id = skill_id.trim();
        if skill_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "skill_id must not be empty".to_string(),
            ));
        }
        let path = format!("skills/{skill_id}/parent");
        self.fetch_cached_json(
            path,
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_por_requirements(
        &self,
        task_type: &str,
    ) -> Result<CachedJson, MarkovClientError> {
        let task_type = task_type.trim();
        if task_type.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "task_type must not be empty".to_string(),
            ));
        }
        let path = format!("por/requirements/{task_type}");
        self.fetch_cached_json(
            path,
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_por_status(&self, evidence_id: &str) -> Result<CachedJson, MarkovClientError> {
        let evidence_id = evidence_id.trim();
        if evidence_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "evidence_id must not be empty".to_string(),
            ));
        }
        let path = format!("por/status/{evidence_id}");
        self.fetch_cached_json(
            path,
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_por_triad_requirements(
        &self,
        track: &str,
        transition: &str,
    ) -> Result<CachedJson, MarkovClientError> {
        let track = track.trim();
        let transition = transition.trim();
        if track.is_empty() || transition.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "track and transition must not be empty".to_string(),
            ));
        }
        let path = format!("por/triad-requirements/{track}/{transition}");
        self.fetch_cached_json(
            path,
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_reputation_leaderboard(
        &self,
        limit: Option<u32>,
        tier: Option<&str>,
        rank_by: Option<&str>,
    ) -> Result<CachedJson, MarkovClientError> {
        let mut params = Vec::new();
        if let Some(limit) = limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(tier) = tier {
            let tier = tier.trim();
            if !tier.is_empty() {
                params.push(("tier".to_string(), tier.to_string()));
            }
        }
        if let Some(rank_by) = rank_by {
            let rank_by = rank_by.trim();
            if !rank_by.is_empty() {
                params.push(("rank_by".to_string(), rank_by.to_string()));
            }
        }
        let params = self.with_scope_query(params);
        self.fetch_cached_json(
            "reputation/leaderboard".to_string(),
            params,
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_reputation_distribution(&self) -> Result<CachedJson, MarkovClientError> {
        self.fetch_cached_json(
            "reputation/distribution".to_string(),
            self.with_scope_query(Vec::new()),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_gdf_weather(&self) -> Result<CachedJson, MarkovClientError> {
        self.fetch_cached_json(
            "slash/gdf".to_string(),
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_vouch_budget(&self, user_id: &str) -> Result<CachedJson, MarkovClientError> {
        let user_id = user_id.trim();
        if user_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "user_id must not be empty".to_string(),
            ));
        }
        let identity = Self::normalize_user_selector(user_id);
        let path = format!("users/{identity}/vouch-budget");
        self.fetch_cached_json(
            path,
            self.with_scope_query(Vec::new()),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_decay_warnings(&self, user_id: &str) -> Result<CachedJson, MarkovClientError> {
        let user_id = user_id.trim();
        if user_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "user_id must not be empty".to_string(),
            ));
        }
        let identity = Self::normalize_user_selector(user_id);
        let path = format!("users/{identity}/decay/warnings");
        self.fetch_cached_json(
            path,
            self.with_scope_query(Vec::new()),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_community_pulse(&self) -> Result<CachedJson, MarkovClientError> {
        self.fetch_cached_json(
            "community/pulse/overview".to_string(),
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_community_pulse_insights(&self) -> Result<CachedJson, MarkovClientError> {
        self.fetch_cached_json(
            "community/pulse/insights".to_string(),
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_community_pulse_trends(
        &self,
        period: Option<&str>,
    ) -> Result<CachedJson, MarkovClientError> {
        let mut params = Vec::new();
        if let Some(period) = period {
            let period = period.trim();
            if !period.is_empty() {
                params.push(("period".to_string(), period.to_string()));
            }
        }
        self.fetch_cached_json(
            "community/pulse/trends".to_string(),
            params,
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_hero_status(&self, user_id: &str) -> Result<CachedJson, MarkovClientError> {
        let user_id = user_id.trim();
        if user_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "user_id must not be empty".to_string(),
            ));
        }
        let path = format!("hero/{user_id}");
        self.fetch_cached_json(
            path,
            Vec::new(),
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn get_hero_leaderboard(
        &self,
        limit: Option<u32>,
    ) -> Result<CachedJson, MarkovClientError> {
        let mut params = Vec::new();
        if let Some(limit) = limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        self.fetch_cached_json(
            "hero/leaderboard".to_string(),
            params,
            CacheClass::Gameplay,
            TokenPolicy::Required,
        )
        .await
    }

    pub async fn user_profile_snapshot(
        &self,
        gotong_user_id: &str,
    ) -> Result<MarkovProfileSnapshot, MarkovClientError> {
        let user_id = gotong_user_id.trim();
        if user_id.is_empty() {
            return Err(MarkovClientError::BadRequest(
                "user_id must not be empty".to_string(),
            ));
        }

        let identity = Self::platform_identity(user_id);
        let reputation = self.get_user_reputation_by_identity(&identity).await?;
        let markov_user_id = reputation
            .value
            .get("user_id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);

        let (tier, activity, cv_hidup) = if let Some(markov_user_id) = markov_user_id.clone() {
            let (tier_result, activity_result, cv_result) = tokio::join!(
                self.get_user_tier(&markov_user_id),
                self.get_user_activity(&markov_user_id),
                self.get_cv_hidup(&markov_user_id)
            );
            (tier_result.ok(), activity_result.ok(), cv_result.ok())
        } else {
            (None, None, None)
        };

        Ok(MarkovProfileSnapshot {
            identity,
            markov_user_id,
            reputation,
            tier,
            activity,
            cv_hidup,
        })
    }

    async fn fetch_cached_json(
        &self,
        path: String,
        query_params: Vec<(String, String)>,
        cache_class: CacheClass,
        token_policy: TokenPolicy,
    ) -> Result<CachedJson, MarkovClientError> {
        let normalized_query = normalize_query(query_params);
        let cache_key = cache_key(&path, &normalized_query);
        let now = Instant::now();
        if let Some(entry) = self.cache.read().await.get(&cache_key).cloned() {
            if now <= entry.fresh_until {
                return Ok(entry.into_cached_json(CacheStatus::Hit, now));
            }
            if now <= entry.stale_until {
                self.spawn_revalidate(
                    cache_key.clone(),
                    path.clone(),
                    normalized_query.clone(),
                    cache_class,
                    token_policy,
                )
                .await;
                return Ok(entry.into_cached_json(CacheStatus::Stale, now));
            }
        }

        loop {
            let waiter = {
                let mut inflight = self.inflight_fetches.lock().await;
                if let Some(notify) = inflight.get(&cache_key) {
                    Some(notify.clone())
                } else {
                    inflight.insert(cache_key.clone(), Arc::new(Notify::new()));
                    None
                }
            };

            if let Some(waiter) = waiter {
                waiter.notified().await;
                let now = Instant::now();
                if let Some(entry) = self.cache.read().await.get(&cache_key).cloned() {
                    if now <= entry.fresh_until {
                        return Ok(entry.into_cached_json(CacheStatus::Hit, now));
                    }
                    if now <= entry.stale_until {
                        return Ok(entry.into_cached_json(CacheStatus::Stale, now));
                    }
                }
                continue;
            }

            let fetched = self
                .fetch_from_origin(&path, &normalized_query, token_policy)
                .await;

            let notify = {
                let mut inflight = self.inflight_fetches.lock().await;
                inflight
                    .remove(&cache_key)
                    .unwrap_or_else(|| Arc::new(Notify::new()))
            };

            match fetched {
                Ok(value) => {
                    let now = Instant::now();
                    let entry = CacheEntry::new(
                        value,
                        now,
                        cache_class.ttl(self),
                        cache_class.stale_window(self),
                    );
                    let result = entry.into_cached_json(CacheStatus::Miss, now);
                    let mut cache = self.cache.write().await;
                    cache.insert(cache_key.clone(), entry);
                    self.prune_cache_locked(&mut cache, now);
                    notify.notify_waiters();
                    return Ok(result);
                }
                Err(err) => {
                    notify.notify_waiters();
                    return Err(err);
                }
            }
        }
    }

    async fn spawn_revalidate(
        &self,
        cache_key: String,
        path: String,
        query_params: Vec<(String, String)>,
        cache_class: CacheClass,
        token_policy: TokenPolicy,
    ) {
        let mut revalidating = self.revalidating_keys.lock().await;
        if revalidating.contains(&cache_key) {
            return;
        }
        revalidating.insert(cache_key.clone());
        drop(revalidating);

        let client = self.clone();
        tokio::spawn(async move {
            let refreshed = client
                .fetch_from_origin(&path, &query_params, token_policy)
                .await;
            if let Ok(value) = refreshed {
                let now = Instant::now();
                let entry = CacheEntry::new(
                    value,
                    now,
                    cache_class.ttl(&client),
                    cache_class.stale_window(&client),
                );
                let mut cache = client.cache.write().await;
                cache.insert(cache_key.clone(), entry);
                client.prune_cache_locked(&mut cache, now);
            } else if let Err(err) = refreshed {
                tracing::warn!(error = %err, path = %path, "markov cache revalidation failed");
            }

            let mut revalidating = client.revalidating_keys.lock().await;
            revalidating.remove(&cache_key);
        });
    }

    async fn fetch_from_origin(
        &self,
        path: &str,
        query_params: &[(String, String)],
        token_policy: TokenPolicy,
    ) -> Result<Value, MarkovClientError> {
        let attempts = self.retry_max_attempts.max(1);
        let url = endpoint_url(&self.base_url, path);

        for attempt in 0..attempts {
            self.ensure_circuit_closed().await?;

            let mut request = self.http.get(&url).header("accept", "application/json");
            if !query_params.is_empty() {
                request = request.query(query_params);
            }
            match (&self.platform_token, token_policy) {
                (Some(token), TokenPolicy::Required) => {
                    request = request.header(PLATFORM_TOKEN_HEADER, token);
                }
                (None, TokenPolicy::Required) => {
                    return Err(MarkovClientError::Configuration(
                        "markov platform token is required but not configured".to_string(),
                    ));
                }
            }

            let response = match request.send().await {
                Ok(response) => response,
                Err(err) => {
                    if attempt + 1 < attempts {
                        sleep(backoff_for_attempt(
                            self.retry_backoff_base,
                            self.retry_backoff_max,
                            attempt,
                        ))
                        .await;
                        continue;
                    }
                    self.record_transient_failure().await;
                    return Err(MarkovClientError::Transport(err.to_string()));
                }
            };

            let status = response.status();
            if status.is_success() {
                let body = response
                    .json::<Value>()
                    .await
                    .map_err(|err| MarkovClientError::InvalidResponse(err.to_string()))?;
                self.record_success().await;
                return Ok(body);
            }

            let message = response.text().await.unwrap_or_default();
            match status {
                StatusCode::BAD_REQUEST => {
                    self.record_success().await;
                    return Err(MarkovClientError::BadRequest(message));
                }
                StatusCode::UNAUTHORIZED => {
                    self.record_success().await;
                    return Err(MarkovClientError::Unauthorized(message));
                }
                StatusCode::FORBIDDEN => {
                    self.record_success().await;
                    return Err(MarkovClientError::Forbidden(message));
                }
                StatusCode::NOT_FOUND => {
                    self.record_success().await;
                    return Err(MarkovClientError::NotFound(message));
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    if attempt + 1 < attempts {
                        sleep(backoff_for_attempt(
                            self.retry_backoff_base,
                            self.retry_backoff_max,
                            attempt,
                        ))
                        .await;
                        continue;
                    }
                    self.record_transient_failure().await;
                    return Err(MarkovClientError::Upstream(format!(
                        "status {}: {}",
                        status.as_u16(),
                        message
                    )));
                }
                _ if status.is_server_error() => {
                    if attempt + 1 < attempts {
                        sleep(backoff_for_attempt(
                            self.retry_backoff_base,
                            self.retry_backoff_max,
                            attempt,
                        ))
                        .await;
                        continue;
                    }
                    self.record_transient_failure().await;
                    return Err(MarkovClientError::Upstream(format!(
                        "status {}: {}",
                        status.as_u16(),
                        message
                    )));
                }
                _ => {
                    self.record_success().await;
                    return Err(MarkovClientError::Upstream(format!(
                        "status {}: {}",
                        status.as_u16(),
                        message
                    )));
                }
            }
        }

        Err(MarkovClientError::Upstream(
            "retry loop exited unexpectedly".to_string(),
        ))
    }

    async fn post_from_origin(
        &self,
        path: &str,
        payload: Value,
        token_policy: TokenPolicy,
    ) -> Result<Value, MarkovClientError> {
        let attempts = self.retry_max_attempts.max(1);
        let url = endpoint_url(&self.base_url, path);

        for attempt in 0..attempts {
            self.ensure_circuit_closed().await?;

            let mut request = self
                .http
                .post(&url)
                .header("accept", "application/json")
                .json(&payload);

            match (&self.platform_token, token_policy) {
                (Some(token), TokenPolicy::Required) => {
                    request = request.header(PLATFORM_TOKEN_HEADER, token);
                }
                (None, TokenPolicy::Required) => {
                    return Err(MarkovClientError::Configuration(
                        "markov platform token is required but not configured".to_string(),
                    ));
                }
            }

            let response = match request.send().await {
                Ok(response) => response,
                Err(err) => {
                    if attempt + 1 < attempts {
                        sleep(backoff_for_attempt(
                            self.retry_backoff_base,
                            self.retry_backoff_max,
                            attempt,
                        ))
                        .await;
                        continue;
                    }
                    self.record_transient_failure().await;
                    return Err(MarkovClientError::Transport(err.to_string()));
                }
            };

            let status = response.status();
            if status.is_success() {
                let body = response
                    .json::<Value>()
                    .await
                    .map_err(|err| MarkovClientError::InvalidResponse(err.to_string()))?;
                self.record_success().await;
                return Ok(body);
            }

            let message = response.text().await.unwrap_or_default();
            match status {
                StatusCode::BAD_REQUEST => {
                    self.record_success().await;
                    return Err(MarkovClientError::BadRequest(message));
                }
                StatusCode::UNAUTHORIZED => {
                    self.record_success().await;
                    return Err(MarkovClientError::Unauthorized(message));
                }
                StatusCode::FORBIDDEN => {
                    self.record_success().await;
                    return Err(MarkovClientError::Forbidden(message));
                }
                StatusCode::NOT_FOUND => {
                    self.record_success().await;
                    return Err(MarkovClientError::NotFound(message));
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    if attempt + 1 < attempts {
                        sleep(backoff_for_attempt(
                            self.retry_backoff_base,
                            self.retry_backoff_max,
                            attempt,
                        ))
                        .await;
                        continue;
                    }
                    self.record_transient_failure().await;
                    return Err(MarkovClientError::Upstream(format!(
                        "status {}: {}",
                        status.as_u16(),
                        message
                    )));
                }
                _ if status.is_server_error() => {
                    if attempt + 1 < attempts {
                        sleep(backoff_for_attempt(
                            self.retry_backoff_base,
                            self.retry_backoff_max,
                            attempt,
                        ))
                        .await;
                        continue;
                    }
                    self.record_transient_failure().await;
                    return Err(MarkovClientError::Upstream(format!(
                        "status {}: {}",
                        status.as_u16(),
                        message
                    )));
                }
                _ => {
                    self.record_success().await;
                    return Err(MarkovClientError::Upstream(format!(
                        "status {}: {}",
                        status.as_u16(),
                        message
                    )));
                }
            }
        }

        Err(MarkovClientError::Upstream(
            "retry loop exited unexpectedly".to_string(),
        ))
    }

    async fn ensure_circuit_closed(&self) -> Result<(), MarkovClientError> {
        let now = Instant::now();
        let mut state = self.circuit.lock().await;
        if let Some(open_until) = state.open_until {
            if now < open_until {
                return Err(MarkovClientError::CircuitOpen);
            }
            state.open_until = None;
            state.consecutive_failures = 0;
        }
        Ok(())
    }

    async fn record_success(&self) {
        let mut state = self.circuit.lock().await;
        state.consecutive_failures = 0;
        state.open_until = None;
    }

    async fn record_transient_failure(&self) {
        let now = Instant::now();
        let mut state = self.circuit.lock().await;
        if let Some(open_until) = state.open_until
            && now < open_until
        {
            return;
        }
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        if state.consecutive_failures >= self.circuit_fail_threshold {
            state.open_until = Some(now + self.circuit_open_duration);
            state.consecutive_failures = 0;
            tracing::warn!(
                open_for_ms = self.circuit_open_duration.as_millis() as u64,
                "markov read circuit opened after repeated failures"
            );
        }
    }

    fn prune_cache_locked(&self, cache: &mut HashMap<String, CacheEntry>, now: Instant) {
        cache.retain(|_, entry| now <= entry.stale_until);
        if cache.len() <= self.cache_max_entries {
            return;
        }

        let mut keys_by_age = cache
            .iter()
            .map(|(key, entry)| (key.clone(), entry.cached_at_instant))
            .collect::<Vec<_>>();
        keys_by_age.sort_by_key(|(_, cached_at)| *cached_at);

        let evict_count = cache.len().saturating_sub(self.cache_max_entries);
        for (key, _) in keys_by_age.into_iter().take(evict_count) {
            cache.remove(&key);
        }
    }
}

fn uncached_json(value: Value) -> CachedJson {
    CachedJson {
        value,
        meta: CacheMetadata {
            status: CacheStatus::Miss,
            stale: false,
            age_ms: 0,
            cached_at_epoch_ms: now_epoch_ms(),
        },
    }
}

fn normalize_query(params: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut out = params
        .into_iter()
        .filter_map(|(key, value)| {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if key.is_empty() || value.is_empty() {
                None
            } else {
                Some((key, value))
            }
        })
        .collect::<Vec<_>>();
    out.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    out
}

fn cache_key(path: &str, query_params: &[(String, String)]) -> String {
    if query_params.is_empty() {
        return path.to_string();
    }
    let query_string = query_params
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    format!("{path}?{query_string}")
}

fn endpoint_url(base_url: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn backoff_for_attempt(base: Duration, max: Duration, attempt: u32) -> Duration {
    if base.is_zero() {
        return Duration::from_millis(1);
    }
    let multiplier = 1u64 << attempt.min(8);
    let base_ms = base.as_millis() as u64;
    let max_ms = max.as_millis() as u64;
    let delay_ms = base_ms.saturating_mul(multiplier).max(1);
    if max_ms == 0 {
        Duration::from_millis(delay_ms)
    } else {
        Duration::from_millis(delay_ms.min(max_ms))
    }
}

fn now_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
