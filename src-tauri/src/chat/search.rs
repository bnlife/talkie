use crate::{models, store, AppState};

/// Find a running MCP search instance and call it with the user's query.
/// Returns (text_for_llm, structured_results).
pub async fn perform_search(state: &AppState, query: &str, search_engine: Option<&str>) -> Result<(String, Vec<models::SearchResult>), String> {
    log::info!("RS::CMD::search | start | query={} engine={:?}", query, search_engine);

    // Find a running MCP instance that provides search
    let instances = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        store::list_mcp_instances(&db).map_err(|e| e.to_string())?
    };

    log::debug!("RS::CMD::search | installed={}", instances.len());
    for inst in &instances {
        log::debug!("RS::CMD::search | instance | id={} server_id={} enabled={}", inst.id, inst.server_id, inst.enabled);
    }

    // Look for an enabled instance whose server is a search server
    // If search_engine is specified, match by server_id; otherwise use first match
    let search_instance = if let Some(engine) = search_engine {
        instances.iter().find(|i| {
            i.enabled && i.server_id == engine
        }).or_else(|| {
            // Fallback: try contains match
            instances.iter().find(|i| {
                i.enabled && (i.server_id == "brave-search" || i.server_id == "duckduckgo"
                    || i.server_id == "bocha-search" || i.server_id == "local:bocha-search"
                    || i.server_id.contains("search"))
            })
        })
    } else {
        instances.iter().find(|i| {
            i.enabled && (i.server_id == "brave-search" || i.server_id == "duckduckgo"
                || i.server_id == "bocha-search" || i.server_id == "local:bocha-search"
                || i.server_id.contains("search"))
        })
    };

    let instance = match search_instance {
        Some(i) => {
            log::info!("RS::CMD::search | found | id={} server_id={}", i.id, i.server_id);
            i
        }
        None => {
            log::warn!("RS::CMD::search | no search instance");
            return Err("没有启用的搜索 MCP 实例".to_string());
        }
    };

    // If instance is not running, try to auto-start it
    if !state.mcp_pool.is_running(&instance.id).await {
        log::info!("RS::CMD::search | auto-starting | id={}", instance.id);
        state.mcp_pool.start(instance).await.map_err(|e| {
            log::error!("RS::CMD::search | auto-start failed | id={} err={}", instance.id, e);
            format!("搜索引擎启动失败: {}", e)
        })?;
    }

    // Call the search tool
    let tool_name = if instance.server_id.contains("bocha") {
        "bocha_search"
    } else if instance.server_id.contains("tavily") {
        "tavily-search"
    } else {
        "search" // Generic fallback
    };

    let args = if instance.server_id.contains("tavily") {
        serde_json::json!({
            "query": query,
            "max_results": 5,
        })
    } else {
        serde_json::json!({
            "query": query,
            "count": 5,
            "freshness": "noLimit",
            "summary": true,
        })
    };

    let result = state.mcp_pool.call_tool(&instance.id, tool_name, args).await?;
    log::debug!("RS::CMD::search | raw | {}", serde_json::to_string(&result).unwrap_or_default());

    // Parse structured results and format text for LLM
    let search_results = parse_search_results(&result);
    let text = format_search_results(&result);
    for (i, sr) in search_results.iter().enumerate() {
        log::info!("RS::CMD::search | parsed[{}] | title={} url={} snippet={:?}", i, sr.title, sr.url, sr.snippet);
    }
    log::info!("RS::CMD::search | done | results={} text_len={}", search_results.len(), text.len());

    Ok((text, search_results))
}

/// Format MCP search tool results into a readable context string with numbered citations.
pub fn format_search_results(result: &serde_json::Value) -> String {
    // Try to extract text content from MCP tool result
    if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
        let mut texts = Vec::new();
        for item in content {
            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                texts.push(text.to_string());
            }
        }
        if !texts.is_empty() {
            return texts.join("\n");
        }
    }

    // Fallback: use the raw JSON
    serde_json::to_string_pretty(result).unwrap_or_default()
}

/// Parse MCP search tool results into structured `SearchResult` items.
///
/// Tries to extract `[title](url)` links from the text content.
/// Returns an empty vec if no structured results can be parsed.
pub fn parse_search_results(result: &serde_json::Value) -> Vec<models::SearchResult> {
    let mut results = Vec::new();

    let text = match result.get("content").and_then(|c| c.as_array()) {
        Some(items) => {
            items.iter()
                .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("\n")
        }
        None => return results,
    };

    // Pattern: "N. [title](url)" followed by optional snippet line (non-URL, non-numbered)
    let re = regex::Regex::new(r"(?m)^\d+\.\s+\[([^\]]+)\]\(([^)]+)\)(?:\n[ \t]+([^\n\d][^\n]*))?")
        .expect("invalid regex");

    for cap in re.captures_iter(&text) {
        let title = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let url = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
    let snippet = cap.get(3).and_then(|m| {
        let s = m.as_str().trim();
        // Filter out "Published:" lines from snippet
        if s.starts_with("Published:") { None } else { Some(s.to_string()) }
    });

        if !title.is_empty() && !url.is_empty() {
            results.push(models::SearchResult { title, url, snippet });
        }
    }

    log::info!("RS::CMD::parse | count={}", results.len());
    results
}
