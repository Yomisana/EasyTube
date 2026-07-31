use url::Url;

pub struct FilterPipeline;

impl FilterPipeline {
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
        let host = url.host_str().unwrap_or("");
        allowlist
            .iter()
            .any(|allowed| host == allowed.as_str() || host.ends_with(&format!(".{}", allowed)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_multiline() {
        let input = "  https://youtube.com/watch?v=123\nsome text  ";
        assert_eq!(
            FilterPipeline::clean(input),
            Some("https://youtube.com/watch?v=123")
        );
    }

    #[test]
    fn test_extract_url() {
        let text = "check https://youtube.com/watch?v=123 out";
        let url = FilterPipeline::extract_url(text);
        assert!(url.is_some());
        assert_eq!(url.unwrap().as_str(), "https://youtube.com/watch?v=123");
    }

    #[test]
    fn test_strip_tracking() {
        let mut url = Url::parse("https://example.com/watch?v=1&utm_source=fb&ref=share").unwrap();
        let blacklist = vec!["utm_source".into(), "ref".into(), "fbclid".into()];
        FilterPipeline::strip_tracking(&mut url, &blacklist);
        assert_eq!(url.as_str(), "https://example.com/watch?v=1");
    }

    #[test]
    fn test_is_allowed() {
        let url = Url::parse("https://www.youtube.com/watch?v=123").unwrap();
        let allowlist = vec!["youtube.com".into(), "vimeo.com".into()];
        assert!(FilterPipeline::is_allowed(&url, &allowlist));
    }
}
