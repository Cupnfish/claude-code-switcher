//! Provider account usage queries for the apply TUI.

use crate::templates::TemplateType;
use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use reqwest::Url;
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Duration;

const HTTP_TIMEOUT: Duration = Duration::from_secs(12);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageReport {
    pub provider: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsageStatus {
    Unsupported,
    NoKey,
    Loading {
        stale: Option<UsageReport>,
        spinner: char,
    },
    Ready(UsageReport),
    Error {
        message: String,
        stale: Option<UsageReport>,
    },
}

#[derive(Debug, Clone)]
pub struct UsageRequest {
    pub template_type: TemplateType,
    pub api_key: String,
    pub anthropic_base_url: Option<String>,
}

pub fn supports_usage(template_type: &TemplateType) -> bool {
    matches!(template_type, TemplateType::DeepSeek | TemplateType::Zai)
}

pub fn query_usage(request: &UsageRequest) -> Result<UsageReport> {
    if request.api_key.trim().is_empty() {
        return Err(anyhow!("API key is empty"));
    }

    let client = Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .context("failed to build HTTP client")?;

    match request.template_type {
        TemplateType::DeepSeek => query_deepseek_balance(&client, &request.api_key),
        TemplateType::Zai => {
            let base_url = request
                .anthropic_base_url
                .as_deref()
                .ok_or_else(|| anyhow!("ZAI base URL is missing"))?;
            query_zai_usage(&client, &request.api_key, base_url)
        }
        _ => Err(anyhow!("usage query is not supported for this provider")),
    }
}

fn query_deepseek_balance(client: &Client, api_key: &str) -> Result<UsageReport> {
    let value: Value = client
        .get("https://api.deepseek.com/user/balance")
        .header(reqwest::header::ACCEPT, "application/json")
        .bearer_auth(api_key)
        .send()
        .context("DeepSeek balance request failed")?
        .error_for_status()
        .context("DeepSeek balance request returned an error")?
        .json()
        .context("failed to parse DeepSeek balance response")?;

    Ok(parse_deepseek_balance(&value))
}

fn query_zai_usage(
    client: &Client,
    api_key: &str,
    anthropic_base_url: &str,
) -> Result<UsageReport> {
    let base_domain = base_domain(anthropic_base_url)?;
    let quota_limit = query_zai_endpoint(
        client,
        api_key,
        &format!("{base_domain}/api/monitor/usage/quota/limit"),
    )
    .context("ZAI quota request failed")?;

    Ok(parse_zai_usage(
        &quota_limit,
        platform_label(anthropic_base_url),
    ))
}

fn query_zai_endpoint(client: &Client, api_key: &str, url: &str) -> Result<Value> {
    let parsed_url = Url::parse(url).with_context(|| format!("invalid URL: {url}"))?;
    client
        .get(parsed_url)
        .header(reqwest::header::AUTHORIZATION, api_key)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("Accept-Language", "en-US,en")
        .send()
        .with_context(|| format!("request failed for {url}"))?
        .error_for_status()
        .with_context(|| format!("request returned an error for {url}"))?
        .json()
        .with_context(|| format!("failed to parse JSON for {url}"))
}

fn base_domain(base_url: &str) -> Result<String> {
    let parsed = Url::parse(base_url).with_context(|| format!("invalid base URL: {base_url}"))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow!("base URL has no host: {base_url}"))?;
    let port = parsed.port().map(|p| format!(":{p}")).unwrap_or_default();
    Ok(format!("{}://{}{}", parsed.scheme(), host, port))
}

fn platform_label(base_url: &str) -> &'static str {
    if base_url.contains("api.z.ai") {
        "ZAI"
    } else {
        "ZHIPU"
    }
}

fn parse_deepseek_balance(value: &Value) -> UsageReport {
    let available = value
        .get("is_available")
        .and_then(Value::as_bool)
        .map(|v| if v { "available" } else { "unavailable" });

    let balances = value
        .get("balance_infos")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(format_deepseek_balance_info)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut parts = Vec::new();
    if let Some(available) = available {
        parts.push(available.to_string());
    }
    parts.extend(balances.into_iter().take(2));

    usage_report("DeepSeek", parts)
}

fn format_deepseek_balance_info(value: &Value) -> Option<String> {
    let currency = value
        .get("currency")
        .and_then(Value::as_str)
        .unwrap_or("balance");
    let total = primitive_to_string(value.get("total_balance")?)?;
    let granted = value
        .get("granted_balance")
        .and_then(primitive_to_string)
        .map(|v| format!("grant {v}"));
    let topped_up = value
        .get("topped_up_balance")
        .and_then(primitive_to_string)
        .map(|v| format!("top-up {v}"));

    let mut suffix = Vec::new();
    if let Some(granted) = granted {
        suffix.push(granted);
    }
    if let Some(topped_up) = topped_up {
        suffix.push(topped_up);
    }

    if suffix.is_empty() {
        Some(format!("{currency} {total}"))
    } else {
        Some(format!("{currency} {total} ({})", suffix.join(", ")))
    }
}

fn parse_zai_usage(quota_limit: &Value, provider: &str) -> UsageReport {
    usage_report(
        provider,
        summarize_zai_quota(data_or_self(quota_limit))
            .into_iter()
            .collect(),
    )
}

fn data_or_self(value: &Value) -> &Value {
    value.get("data").unwrap_or(value)
}

fn summarize_zai_quota(data: &Value) -> Option<String> {
    let limits = data.get("limits")?.as_array()?;
    let mut parts = Vec::new();

    for item in limits {
        if let Some(part) = summarize_zai_limit(item) {
            parts.push(part);
        }

        if let Some(details) = item.get("usageDetails").and_then(Value::as_array) {
            for detail in details {
                if let Some(part) = summarize_zai_limit(detail) {
                    parts.push(part);
                }
            }
        }
    }

    dedup_keep_order(&mut parts);
    if parts.is_empty() {
        None
    } else {
        Some(parts.into_iter().take(4).collect::<Vec<_>>().join(" | "))
    }
}

fn summarize_zai_limit(item: &Value) -> Option<String> {
    let label = zai_limit_label(item)?;
    let percentage = value_from_keys(
        item,
        &[
            "percentage",
            "percent",
            "usagePercentage",
            "usagePercent",
            "usedPercentage",
            "usedPercent",
        ],
    )
    .and_then(format_percentage);
    let current = value_from_keys(
        item,
        &[
            "currentValue",
            "currentUsage",
            "usedValue",
            "used",
            "usageValue",
            "consumed",
        ],
    )
    .and_then(primitive_to_string);
    let total = value_from_keys(
        item,
        &[
            "usage",
            "total",
            "limit",
            "totalValue",
            "maxValue",
            "quota",
            "quotaValue",
        ],
    )
    .and_then(primitive_to_string);

    let mut text = match (current, total, percentage) {
        (Some(current), Some(total), Some(percentage)) => {
            format!("{label} {current}/{total} {percentage}")
        }
        (Some(current), Some(total), None) => format!("{label} {current}/{total}"),
        (_, _, Some(percentage)) => format!("{label} {percentage}"),
        (Some(current), None, None) => format!("{label} {current}"),
        _ => return None,
    };

    if let Some(reset) = zai_reset_time(item) {
        text.push('@');
        text.push_str(&reset);
    }

    Some(text)
}

fn zai_limit_label(item: &Value) -> Option<String> {
    let raw = value_from_keys(
        item,
        &["type", "name", "label", "period", "cycle", "window"],
    )
    .and_then(Value::as_str)
    .unwrap_or_default();
    let normalized = raw.to_ascii_uppercase();

    if normalized == "TOKENS_LIMIT"
        || normalized.contains("5H")
        || normalized.contains("5_HOUR")
        || normalized.contains("FIVE_HOUR")
        || (normalized.contains("TOKEN") && normalized.contains("HOUR"))
    {
        return Some("5h".to_string());
    }
    if normalized == "TIME_LIMIT" {
        return Some("MCP".to_string());
    }
    if normalized.contains("WEEK") {
        return Some("wk".to_string());
    }
    if normalized.contains("MONTH") {
        return Some("mo".to_string());
    }
    if normalized.contains("DAY") {
        return Some("day".to_string());
    }
    if normalized.contains("MCP") {
        return Some("MCP".to_string());
    }

    None
}

fn value_from_keys<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| value.get(*key))
}

fn zai_reset_time(item: &Value) -> Option<String> {
    value_from_keys(
        item,
        &[
            "nextResetTime",
            "next_reset_time",
            "nextRefreshTime",
            "next_refresh_time",
            "refreshTime",
            "resetTime",
            "resetAt",
            "expireTime",
            "expiresAt",
            "endTime",
            "validUntil",
        ],
    )
    .and_then(format_reset_time)
}

fn format_reset_time(value: &Value) -> Option<String> {
    match value {
        Value::Number(number) => {
            let timestamp = number.as_i64()?;
            let dt = if timestamp > 1_000_000_000_000 {
                Local.timestamp_millis_opt(timestamp).single()?
            } else {
                Local.timestamp_opt(timestamp, 0).single()?
            };
            Some(format_local_time(dt))
        }
        Value::String(raw) => format_reset_time_string(raw),
        _ => None,
    }
}

fn format_reset_time_string(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(format_local_time(dt.with_timezone(&Local)));
    }
    for fmt in [
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
    ] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(raw, fmt) {
            let local = Local
                .from_local_datetime(&naive)
                .single()
                .or_else(|| Local.from_local_datetime(&naive).earliest())?;
            return Some(format_local_time(local));
        }
    }
    Some(raw.chars().take(16).collect())
}

fn format_local_time(dt: DateTime<Local>) -> String {
    let now = Local::now();
    if dt.date_naive() == now.date_naive() {
        dt.format("%H:%M").to_string()
    } else {
        dt.format("%m-%d %H:%M").to_string()
    }
}

fn format_percentage(value: &Value) -> Option<String> {
    primitive_to_string(value).map(|mut raw| {
        if !raw.ends_with('%') {
            raw.push('%');
        }
        raw
    })
}

fn primitive_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(v) => Some(v.to_string()),
        _ => None,
    }
}

fn dedup_keep_order(values: &mut Vec<String>) {
    let mut unique = Vec::new();
    for value in values.drain(..) {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    *values = unique;
}

fn usage_report(provider: &str, parts: Vec<String>) -> UsageReport {
    let parts = parts
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let summary = if parts.is_empty() {
        "no usage data".to_string()
    } else {
        parts.join(" | ")
    };
    UsageReport {
        provider: provider.to_string(),
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_deepseek_balance() {
        let report = parse_deepseek_balance(&json!({
            "is_available": true,
            "balance_infos": [{
                "currency": "CNY",
                "total_balance": "12.34",
                "granted_balance": "2.00",
                "topped_up_balance": "10.34"
            }]
        }));

        assert_eq!(report.provider, "DeepSeek");
        assert!(report.summary.contains("available"));
        assert!(report.summary.contains("CNY 12.34"));
        assert!(report.summary.contains("grant 2.00"));
    }

    #[test]
    fn parses_zai_quota_and_usage() {
        let report = parse_zai_usage(
            &json!({
                "data": {
                    "limits": [
                        {"type": "TOKENS_LIMIT", "percentage": 42, "nextResetTime": "2026-06-18 18:00:00"},
                        {"type": "WEEKLY_LIMIT", "percentage": 15, "currentValue": 150, "usage": 1000, "nextResetTime": "2026-06-22 00:00:00"},
                        {"type": "TIME_LIMIT", "percentage": 7, "currentValue": 2, "usage": 30}
                    ]
                }
            }),
            "ZHIPU",
        );

        assert_eq!(report.provider, "ZHIPU");
        assert!(report.summary.contains("5h 42%@"));
        assert!(report.summary.contains("wk 150/1000 15%@"));
        assert!(report.summary.contains("MCP 2/30 7%"));
        assert!(!report.summary.contains("24h"));
    }

    #[test]
    fn zai_fallback_does_not_render_raw_json() {
        let report = parse_zai_usage(
            &json!({"data": {"limits": [{"type": "UNKNOWN_LIMIT", "extra": {"raw": true}}]}}),
            "ZHIPU",
        );

        assert_eq!(report.summary, "no usage data");
        assert!(!report.summary.contains('{'));
        assert!(!report.summary.contains('}'));
    }

    #[test]
    fn derives_base_domain_from_anthropic_url() {
        assert_eq!(
            base_domain("https://open.bigmodel.cn/api/anthropic").unwrap(),
            "https://open.bigmodel.cn"
        );
        assert_eq!(
            base_domain("https://localhost:8443/api/anthropic").unwrap(),
            "https://localhost:8443"
        );
    }
}
