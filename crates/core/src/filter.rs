use std::collections::HashMap;
use std::time::Instant;

use url::Url;

pub struct FilterPipeline {
    recent_urls: HashMap<String, Instant>,
    cooldown_secs: u64,
}

impl FilterPipeline {
    pub fn new(cooldown_secs: u64) -> Self {
        Self {
            recent_urls: HashMap::new(),
            cooldown_secs,
        }
    }

    pub fn clean(text: &str) -> Option<&str> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return None;
        }
        trimmed.lines().next()
    }

    pub fn extract_url(text: &str) -> Option<Url> {
        let re = regex::Regex::new(r"https?://[\w\-._~:/?#\[\]@!$&'()*+,;=]+").ok()?;
        let cap = re.find(text)?;
        Url::parse(cap.as_str()).ok()
    }

    pub fn strip_tracking(url: &mut Url, blacklist: &[String]) {
        let params: Vec<String> = url
            .query_pairs()
            .filter(|(k, _)| {
                let key_lower = k.to_lowercase();
                !blacklist.iter().any(|b| key_lower == b.to_lowercase())
            })
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let query = params.join("&");
        url.set_query(if query.is_empty() { None } else { Some(&query) });
    }

    pub fn is_allowed(url: &Url, allowlist: &[String]) -> bool {
        if allowlist.is_empty() {
            return true;
        }
        let host = url.host_str().unwrap_or("");
        allowlist
            .iter()
            .any(|allowed| host == allowed.as_str() || host.ends_with(&format!(".{}", allowed)))
    }

    pub fn is_allowed_domain(url: &Url, allowlist: &[String], domain_blocklist: &[String]) -> bool {
        let host = url.host_str().unwrap_or("");

        if domain_blocklist
            .iter()
            .any(|b| host == b.as_str() || host.ends_with(&format!(".{}", b)))
        {
            return false;
        }

        Self::is_allowed(url, allowlist)
    }

    pub fn is_duplicate(&mut self, url: &Url) -> bool {
        let key = format!(
            "{}:{}{}",
            url.host_str().unwrap_or(""),
            url.path(),
            url.query().map(|q| format!("?{}", q)).unwrap_or_default()
        );
        self.cleanup_expired();
        if let Some(last_seen) = self.recent_urls.get(&key) {
            if last_seen.elapsed().as_secs() < self.cooldown_secs {
                return true;
            }
        }
        self.recent_urls.insert(key, Instant::now());
        false
    }

    fn cleanup_expired(&mut self) {
        let cooldown = self.cooldown_secs;
        self.recent_urls
            .retain(|_, t| t.elapsed().as_secs() < cooldown);
    }

    pub fn normalize_url(url: &Url) -> String {
        let mut s = url.to_string();
        if let Some(idx) = s.find('#') {
            s.truncate(idx);
        }
        if s.ends_with('/') {
            s.pop();
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterResult {
    Valid { url: String, cleaned_url: String },
    Ignored(String),
}

pub fn run_filter_pipeline(
    raw_text: &str,
    pipeline: &mut FilterPipeline,
    allowlist: &[String],
    domain_blocklist: &[String],
    tracking_blacklist: &[String],
    history_urls: &[String],
) -> FilterResult {
    let text = match FilterPipeline::clean(raw_text) {
        Some(t) => t,
        None => return FilterResult::Ignored("empty_or_whitespace".into()),
    };

    let mut url = match FilterPipeline::extract_url(text) {
        Some(u) => u,
        None => return FilterResult::Ignored("no_url_found".into()),
    };

    FilterPipeline::strip_tracking(&mut url, tracking_blacklist);

    if !FilterPipeline::is_allowed_domain(&url, allowlist, domain_blocklist) {
        return FilterResult::Ignored("domain_not_allowed".into());
    }

    let normalized = FilterPipeline::normalize_url(&url);

    if pipeline.is_duplicate(&url) {
        return FilterResult::Ignored("duplicate_url".into());
    }

    if history_urls
        .iter()
        .any(|h| normalize_url_str(h) == normalized)
    {
        return FilterResult::Ignored("already_downloaded".into());
    }

    FilterResult::Valid {
        url: url.to_string(),
        cleaned_url: normalized,
    }
}

fn normalize_url_str(url: &str) -> String {
    if let Ok(u) = Url::parse(url) {
        FilterPipeline::normalize_url(&u)
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_multiline() {
        assert_eq!(
            FilterPipeline::clean("  https://youtube.com/watch?v=123\nother"),
            Some("https://youtube.com/watch?v=123")
        );
    }

    #[test]
    fn test_clean_empty() {
        assert_eq!(FilterPipeline::clean("   "), None);
    }

    #[test]
    fn test_extract_url() {
        let url = FilterPipeline::extract_url("check https://youtube.com/watch?v=123 out").unwrap();
        assert_eq!(url.as_str(), "https://youtube.com/watch?v=123");
    }

    #[test]
    fn test_extract_no_url() {
        assert!(FilterPipeline::extract_url("hello world").is_none());
    }

    #[test]
    fn test_strip_tracking() {
        let mut url = Url::parse("https://example.com/watch?v=1&utm_source=fb&ref=share").unwrap();
        let blacklist = vec!["utm_source".into(), "ref".into(), "fbclid".into()];
        FilterPipeline::strip_tracking(&mut url, &blacklist);
        assert_eq!(url.as_str(), "https://example.com/watch?v=1");
    }

    #[test]
    fn test_is_allowed_match() {
        let url = Url::parse("https://www.youtube.com/watch?v=123").unwrap();
        assert!(FilterPipeline::is_allowed(&url, &["youtube.com".into()]));
    }

    #[test]
    fn test_is_allowed_subdomain() {
        let url = Url::parse("https://www.youtube.com/watch?v=123").unwrap();
        assert!(FilterPipeline::is_allowed(&url, &["youtube.com".into()]));
    }

    #[test]
    fn test_is_allowed_denied() {
        let url = Url::parse("https://evil.com/watch").unwrap();
        assert!(!FilterPipeline::is_allowed(&url, &["youtube.com".into()]));
    }

    #[test]
    fn test_is_allowed_empty_allowlist() {
        let url = Url::parse("https://anything.com/watch").unwrap();
        assert!(FilterPipeline::is_allowed(&url, &[]));
    }

    #[test]
    fn test_domain_blocklist() {
        let url = Url::parse("https://blocked.com/test").unwrap();
        assert!(!FilterPipeline::is_allowed_domain(
            &url,
            &[],
            &["blocked.com".into()]
        ));
    }

    #[test]
    fn test_is_duplicate() {
        let mut pipeline = FilterPipeline::new(10);
        let url = Url::parse("https://youtube.com/watch?v=1").unwrap();
        assert!(!pipeline.is_duplicate(&url));
        assert!(pipeline.is_duplicate(&url));
    }

    #[test]
    fn test_is_duplicate_different_url() {
        let mut pipeline = FilterPipeline::new(10);
        let a = Url::parse("https://youtube.com/watch?v=1").unwrap();
        let b = Url::parse("https://youtube.com/watch?v=2").unwrap();
        assert!(!pipeline.is_duplicate(&a));
        assert!(!pipeline.is_duplicate(&b));
    }

    #[test]
    fn test_normalize_removes_fragment() {
        let url = Url::parse("https://example.com/page#section").unwrap();
        assert_eq!(
            FilterPipeline::normalize_url(&url),
            "https://example.com/page"
        );
    }

    #[test]
    fn test_pipeline_valid() {
        let mut pipeline = FilterPipeline::new(10);
        let result = run_filter_pipeline(
            "https://youtube.com/watch?v=abc&utm_source=fb",
            &mut pipeline,
            &["youtube.com".into()],
            &[],
            &["utm_source".into()],
            &[],
        );
        match result {
            FilterResult::Valid { url, cleaned_url } => {
                assert!(cleaned_url.contains("youtube.com/watch?v=abc"));
                assert!(!cleaned_url.contains("utm_source"));
                assert!(!url.contains("utm_source"));
            }
            _ => panic!("expected valid"),
        }
    }

    #[test]
    fn test_pipeline_blocked_domain() {
        let mut pipeline = FilterPipeline::new(10);
        let result = run_filter_pipeline(
            "https://evil.com/watch?v=abc",
            &mut pipeline,
            &["youtube.com".into()],
            &["evil.com".into()],
            &[],
            &[],
        );
        assert_eq!(result, FilterResult::Ignored("domain_not_allowed".into()));
    }

    #[test]
    fn test_pipeline_already_downloaded() {
        let mut pipeline = FilterPipeline::new(10);
        let result = run_filter_pipeline(
            "https://youtube.com/watch?v=abc",
            &mut pipeline,
            &[],
            &[],
            &[],
            &["https://youtube.com/watch?v=abc".into()],
        );
        assert_eq!(result, FilterResult::Ignored("already_downloaded".into()));
    }

    #[test]
    fn test_pipeline_no_url() {
        let mut pipeline = FilterPipeline::new(10);
        let result = run_filter_pipeline("hello world", &mut pipeline, &[], &[], &[], &[]);
        assert_eq!(result, FilterResult::Ignored("no_url_found".into()));
    }
}
