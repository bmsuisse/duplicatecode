//! Optional semantic name similarity via an embeddings endpoint (Azure AI Foundry / Azure OpenAI /
//! any OpenAI-compatible API), cached in an append-only flat file so each distinct name is embedded
//! at most once per model.

use crate::naming::split_identifier;
use crate::units::Unit;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// `{endpoint}/openai/deployments/{deployment}/embeddings?api-version=…` (`*.openai.azure.com`)
    AzureOpenAi,
    /// `{endpoint}/models/embeddings?api-version=…` (`*.services.ai.azure.com`, Foundry model inference)
    Foundry,
    /// `{endpoint}/embeddings` with `model` in the body (OpenAI and compatible servers)
    OpenAi,
    /// `{endpoint}/v2/embed` (Cohere): `texts` + `input_type`, vectors under `embeddings.float`
    Cohere,
}

#[derive(Clone)]
pub struct EmbedConfig {
    pub endpoint: String,
    /// Deployment name (Azure OpenAI) or model name.
    pub deployment: String,
    pub api_key: Option<String>,
    pub api_version: Option<String>,
    /// Ask the service for shorter vectors (text-embedding-3 supports this); keeps the cache small.
    pub dims: Option<u32>,
    pub style: Style,
}

impl std::fmt::Debug for EmbedConfig {
    /// The API key is never printed.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbedConfig")
            .field("endpoint", &self.endpoint)
            .field("deployment", &self.deployment)
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .field("api_version", &self.api_version)
            .field("dims", &self.dims)
            .field("style", &self.style)
            .finish()
    }
}

/// The API key for `style`: `DUPLICATECODE_EMBED_API_KEY` first, then only the variables of the
/// provider the endpoint belongs to, so one provider's key is never sent to another's endpoint.
fn pick_key(style: Style, get: impl Fn(&str) -> Option<String>) -> Option<String> {
    let provider: &[&str] = match style {
        Style::AzureOpenAi => &["AZURE_OPENAI_API_KEY", "AZURE_AI_FOUNDRY_API_KEY"],
        Style::Foundry => &["AZURE_AI_FOUNDRY_API_KEY", "AZURE_OPENAI_API_KEY"],
        Style::OpenAi => &["OPENAI_API_KEY"],
        Style::Cohere => &["COHERE_API_KEY"],
    };
    ["DUPLICATECODE_EMBED_API_KEY"]
        .iter()
        .chain(provider)
        .find_map(|n| get(n))
}

fn env_first(names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|n| std::env::var(n).ok().filter(|v| !v.is_empty()))
}

impl EmbedConfig {
    /// Reads `AZURE_AI_FOUNDRY_ENDPOINT` (or `AZURE_OPENAI_ENDPOINT`, `DUPLICATECODE_EMBED_ENDPOINT`,
    /// `OPENAI_BASE_URL`), `…_API_KEY` (incl. `OPENAI_API_KEY`), `…_EMBEDDING_DEPLOYMENT` /
    /// `OPENAI_EMBEDDING_MODEL` and `…_API_VERSION`. With only `OPENAI_API_KEY` set, api.openai.com is
    /// used. Without an API key on an Azure endpoint, an Entra ID token from `az` is used.
    pub fn from_env(dims: Option<u32>) -> Option<EmbedConfig> {
        let endpoint = env_first(&[
            "AZURE_AI_FOUNDRY_ENDPOINT",
            "AZURE_OPENAI_ENDPOINT",
            "DUPLICATECODE_EMBED_ENDPOINT",
            "OPENAI_BASE_URL",
            "COHERE_BASE_URL",
        ])
        .or_else(|| env_first(&["OPENAI_API_KEY"]).map(|_| "https://api.openai.com/v1".to_string()))
        .or_else(|| env_first(&["COHERE_API_KEY"]).map(|_| "https://api.cohere.com".to_string()))?;
        let model = env_first(&[
            "AZURE_AI_FOUNDRY_EMBEDDING_DEPLOYMENT",
            "AZURE_OPENAI_EMBEDDING_DEPLOYMENT",
            "DUPLICATECODE_EMBED_MODEL",
            "OPENAI_EMBEDDING_MODEL",
            "COHERE_EMBEDDING_MODEL",
        ])
        .unwrap_or_else(|| {
            if endpoint.contains("cohere") {
                "embed-v4.0".into()
            } else {
                "text-embedding-3-small".into()
            }
        });
        let mut cfg = EmbedConfig::new(
            endpoint,
            model,
            None,
            env_first(&["AZURE_AI_FOUNDRY_API_VERSION", "AZURE_OPENAI_API_VERSION"]),
            dims,
        );
        cfg.api_key = pick_key(cfg.style, |n| env_first(&[n]));
        Some(cfg)
    }

    /// Host of the endpoint (no scheme, port or path), for messages and loopback checks.
    pub fn host(&self) -> String {
        let rest = self.endpoint.split("://").nth(1).unwrap_or(&self.endpoint);
        let authority = rest.split(['/', '?']).next().unwrap_or(rest);
        let authority = authority.rsplit('@').next().unwrap_or(authority);
        match authority.strip_prefix('[') {
            Some(v6) => v6.split(']').next().unwrap_or(v6).to_string(),
            None => authority.split(':').next().unwrap_or(authority).to_string(),
        }
    }

    /// The endpoint is this machine (`localhost`, `127.0.0.0/8`, `::1`).
    pub fn is_loopback(&self) -> bool {
        let h = self.host();
        h == "localhost" || h == "::1" || h.starts_with("127.")
    }

    /// Plain http to a host that is not this machine: key and source text travel in clear text.
    pub fn is_cleartext_remote(&self) -> bool {
        self.endpoint.starts_with("http://") && !self.is_loopback()
    }

    pub fn new(
        endpoint: String,
        deployment: String,
        api_key: Option<String>,
        api_version: Option<String>,
        dims: Option<u32>,
    ) -> Self {
        let e = endpoint.trim_end_matches('/').to_string();
        let style = if e.contains("cohere.com") || e.contains("cohere.ai") {
            Style::Cohere
        } else if e.contains(".openai.azure.com") {
            Style::AzureOpenAi
        } else if e.contains(".services.ai.azure.com")
            || e.contains(".inference.ai.azure.com")
            || e.contains(".models.ai.azure.com")
        {
            Style::Foundry
        } else {
            Style::OpenAi
        };
        EmbedConfig {
            endpoint: e,
            deployment,
            api_key,
            api_version,
            dims,
            style,
        }
    }

    /// Identifies the vector space: cached vectors are only reused for the same model/style/dims.
    pub fn model_id(&self) -> String {
        format!(
            "{:?}:{}:{}",
            self.style,
            self.deployment,
            self.dims.unwrap_or(0)
        )
    }

    fn url(&self) -> String {
        match self.style {
            Style::AzureOpenAi => format!(
                "{}/openai/deployments/{}/embeddings?api-version={}",
                self.endpoint,
                self.deployment,
                self.api_version.as_deref().unwrap_or("2024-10-21")
            ),
            Style::Foundry => format!(
                "{}/models/embeddings?api-version={}",
                self.endpoint,
                self.api_version.as_deref().unwrap_or("2024-05-01-preview")
            ),
            Style::OpenAi => format!("{}/embeddings", self.endpoint),
            Style::Cohere => format!("{}/v2/embed", self.endpoint),
        }
    }

    fn body(&self, texts: &[String], query: bool) -> serde_json::Value {
        if self.style == Style::Cohere {
            let mut b = serde_json::json!({
                "model": self.deployment,
                "texts": texts,
                "input_type": if query { "search_query" } else { "search_document" },
                "embedding_types": ["float"],
            });
            // only embed-v4.0 accepts a reduced output size
            if let Some(d) = self.dims.filter(|_| self.deployment.contains("v4")) {
                b["output_dimension"] = d.into();
            }
            return b;
        }
        let mut b = serde_json::json!({ "input": texts });
        if self.style != Style::AzureOpenAi {
            b["model"] = self.deployment.clone().into();
        }
        if let Some(d) = self.dims {
            b["dimensions"] = d.into();
        }
        b
    }

    /// The authentication header, if any. Without a key, only the Azure styles fall back to an
    /// Entra token; an OpenAI-compatible or Cohere endpoint without a key gets no credential at all
    /// (a keyless local server must never receive an Azure token).
    fn auth(&self) -> Result<Option<(String, String)>, String> {
        if let Some(k) = &self.api_key {
            return Ok(Some(match self.style {
                // OpenAI and compatible servers expect a bearer token; Azure uses `api-key`.
                Style::OpenAi | Style::Cohere => ("Authorization".into(), format!("Bearer {k}")),
                _ => ("api-key".into(), k.clone()),
            }));
        }
        if matches!(self.style, Style::OpenAi | Style::Cohere) {
            return Ok(None);
        }
        if let Some(t) = env_first(&["AZURE_AI_FOUNDRY_TOKEN"]) {
            return Ok(Some(("Authorization".into(), format!("Bearer {t}"))));
        }
        let out = std::process::Command::new("az")
            .args([
                "account",
                "get-access-token",
                "--resource",
                "https://cognitiveservices.azure.com",
                "--query",
                "accessToken",
                "-o",
                "tsv",
            ])
            .output()
            .map_err(|e| format!("no API key set and `az` not usable: {e}"))?;
        let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !out.status.success() || t.is_empty() {
            return Err("no API key set and `az account get-access-token` failed (run `az login` or set AZURE_AI_FOUNDRY_API_KEY)".into());
        }
        Ok(Some(("Authorization".into(), format!("Bearer {t}"))))
    }

    /// Embed one batch of documents. Retries on throttling / transient server errors.
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        self.embed_batch_as(texts, false)
    }

    /// `query`: the texts are search queries, not documents (matters for Cohere's `input_type`).
    fn embed_batch_as(&self, texts: &[String], query: bool) -> Result<Vec<Vec<f32>>, String> {
        let auth = self.auth()?;
        let url = self.url();
        let body = self.body(texts, query);
        // no redirects: a redirect would re-send headers such as `api-key` to another host
        let agent = ureq::AgentBuilder::new().redirects(0).build();
        let mut last = String::new();
        for attempt in 0..4 {
            let mut req = agent.post(&url);
            if let Some((k, v)) = &auth {
                req = req.set(k, v);
            }
            match req.send_json(body.clone()) {
                Ok(resp) => {
                    let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
                    let rows = if self.style == Style::Cohere {
                        parse_cohere(&v)?
                    } else {
                        parse_openai(&v)?
                    };
                    return check_count(rows, texts.len());
                }
                Err(ureq::Error::Status(code, resp)) if code == 429 || code >= 500 => {
                    let wait = resp
                        .header("retry-after")
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(1 << attempt);
                    last = format!("HTTP {code}");
                    std::thread::sleep(std::time::Duration::from_secs(wait.min(20)));
                }
                Err(ureq::Error::Status(code, resp)) => {
                    return Err(format!(
                        "HTTP {code}: {}",
                        resp.into_string()
                            .unwrap_or_default()
                            .chars()
                            .take(300)
                            .collect::<String>()
                    ));
                }
                Err(e) => {
                    last = e.to_string();
                    std::thread::sleep(std::time::Duration::from_secs(1 << attempt));
                }
            }
        }
        Err(format!("embedding request failed: {last}"))
    }
}

/// One embedding row: a non-empty array of numbers. A missing, empty or non-numeric row is an error:
/// it would otherwise be cached as a valid vector and served as a hit forever.
fn row_to_vec(row: &serde_json::Value) -> Result<Vec<f32>, String> {
    let a = row
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or("embedding row is missing, empty or not an array")?;
    a.iter()
        .map(|x| {
            x.as_f64()
                .map(|f| f as f32)
                .ok_or_else(|| "embedding contains a non-numeric value".to_string())
        })
        .collect()
}

/// OpenAI-style: `{"data": [{"index": i, "embedding": [...]}, ...]}`
fn parse_openai(v: &serde_json::Value) -> Result<Vec<Vec<f32>>, String> {
    let mut rows: Vec<(usize, Vec<f32>)> = v["data"]
        .as_array()
        .ok_or("response has no `data` array")?
        .iter()
        .map(|d| {
            Ok((
                d["index"].as_u64().unwrap_or(0) as usize,
                row_to_vec(&d["embedding"])?,
            ))
        })
        .collect::<Result<_, String>>()?;
    rows.sort_by_key(|r| r.0);
    Ok(rows.into_iter().map(|r| r.1).collect())
}

fn check_count(rows: Vec<Vec<f32>>, expected: usize) -> Result<Vec<Vec<f32>>, String> {
    if rows.len() == expected {
        Ok(rows)
    } else {
        Err(format!(
            "expected {expected} embeddings, got {}",
            rows.len()
        ))
    }
}

/// Cohere v2: `{"embeddings": {"float": [[...], ...]}}`
fn parse_cohere(v: &serde_json::Value) -> Result<Vec<Vec<f32>>, String> {
    v["embeddings"]["float"]
        .as_array()
        .ok_or("response has no `embeddings.float` array")?
        .iter()
        .map(row_to_vec)
        .collect()
}

/// Append-only flat-file cache: repeated records of
/// `u16 model_len | model | u16 text_len | text | u16 dims | dims * f32 (little endian)`.
/// Later records win; unreadable tails are ignored (the file stays usable after a crash).
pub struct EmbeddingCache {
    path: PathBuf,
    map: HashMap<(String, String), Arc<[f32]>>,
    pending: Vec<(String, String)>,
    /// Bytes of the file that parsed cleanly; anything after is cut off before the next append so
    /// that new records are never written behind an unreadable tail.
    valid_len: u64,
}

impl EmbeddingCache {
    pub fn default_path() -> PathBuf {
        let base = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
            .unwrap_or_else(|| PathBuf::from(".cache"));
        base.join("duplicatecode").join("embeddings.bin")
    }

    pub fn load(path: &Path) -> EmbeddingCache {
        let mut map = HashMap::new();
        let mut valid_len = 0u64;
        if let Ok(bytes) = std::fs::read(path) {
            let mut i = 0usize;
            let rd = |i: &mut usize, n: usize| -> Option<&[u8]> {
                let s = bytes.get(*i..*i + n)?;
                *i += n;
                Some(s)
            };
            'outer: loop {
                macro_rules! take {
                    ($n:expr) => {
                        match rd(&mut i, $n) {
                            Some(s) => s,
                            None => break 'outer,
                        }
                    };
                }
                let ml = u16::from_le_bytes(take!(2).try_into().unwrap()) as usize;
                let model = String::from_utf8_lossy(take!(ml)).to_string();
                let tl = u16::from_le_bytes(take!(2).try_into().unwrap()) as usize;
                let text = String::from_utf8_lossy(take!(tl)).to_string();
                let d = u16::from_le_bytes(take!(2).try_into().unwrap()) as usize;
                let raw = take!(d * 4);
                let v: Vec<f32> = raw
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                    .collect();
                valid_len = i as u64;
                if !v.is_empty() {
                    map.insert((model, text), Arc::from(v));
                }
            }
        }
        EmbeddingCache {
            path: path.to_path_buf(),
            map,
            pending: Vec::new(),
            valid_len,
        }
    }

    pub fn get(&self, model: &str, text: &str) -> Option<Arc<[f32]>> {
        self.map
            .get(&(model.to_string(), text.to_string()))
            .cloned()
    }

    pub fn insert(&mut self, model: &str, text: &str, v: Vec<f32>) -> Arc<[f32]> {
        let key = (model.to_string(), text.to_string());
        let a: Arc<[f32]> = Arc::from(v);
        self.map.insert(key.clone(), a.clone());
        self.pending.push(key);
        a
    }

    /// Append everything inserted since the last flush. A tail that did not parse on load is cut
    /// off first; entries whose key or vector does not fit the u16 length fields stay in memory only.
    pub fn flush(&mut self) -> std::io::Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        if let Some(dir) = self.path.parent() {
            create_private_dir(dir)?;
        }
        let mut f = open_private_append(&self.path)?;
        if f.metadata()?.len() > self.valid_len {
            f.set_len(self.valid_len)?;
        }
        let mut buf = Vec::new();
        for key in self.pending.drain(..) {
            let v = &self.map[&key];
            let (Ok(ml), Ok(tl), Ok(d)) = (
                u16::try_from(key.0.len()),
                u16::try_from(key.1.len()),
                u16::try_from(v.len()),
            ) else {
                continue;
            };
            buf.extend(ml.to_le_bytes());
            buf.extend(key.0.as_bytes());
            buf.extend(tl.to_le_bytes());
            buf.extend(key.1.as_bytes());
            buf.extend(d.to_le_bytes());
            for x in v.iter() {
                buf.extend(x.to_le_bytes());
            }
        }
        f.write_all(&buf)?;
        self.valid_len += buf.len() as u64;
        Ok(())
    }
}

/// The cache holds identifier names and vectors of private code: owner-only on unix.
fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(dir)
    }
}

fn open_private_append(path: &Path) -> std::io::Result<std::fs::File> {
    let mut o = std::fs::OpenOptions::new();
    o.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    o.open(path)
}

#[derive(Debug, Default)]
pub struct EmbedStats {
    pub distinct_names: usize,
    pub from_cache: usize,
    pub fetched: usize,
}

fn normalize(mut v: Vec<f32>) -> Vec<f32> {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 {
        v.iter_mut().for_each(|x| *x /= n);
    }
    v
}

/// The text embedded for a unit name: its subwords, so `getInitials` and `get_initials` coincide.
pub fn name_phrase(name: &str) -> String {
    split_identifier(name).join(" ")
}

/// Embed every `(cache key, text)` in `wanted` that the cache does not hold yet, `chunk` texts per
/// request, flushing after each request so progress survives a failure later on.
fn fetch_missing(
    cfg: &EmbedConfig,
    cache: &mut EmbeddingCache,
    model: &str,
    wanted: &BTreeMap<String, String>,
    chunk: usize,
) -> Result<EmbedStats, String> {
    let missing: Vec<(&String, &String)> = wanted
        .iter()
        .filter(|(k, _)| cache.get(model, k).is_none())
        .collect();
    let mut stats = EmbedStats {
        distinct_names: wanted.len(),
        from_cache: wanted.len() - missing.len(),
        fetched: 0,
    };
    for batch in missing.chunks(chunk) {
        let texts: Vec<String> = batch.iter().map(|(_, t)| (*t).clone()).collect();
        let vecs = cfg.embed_batch(&texts)?;
        let dim = vecs.first().map_or(0, Vec::len);
        if vecs.iter().any(|v| v.is_empty() || v.len() != dim) {
            return Err("the endpoint returned vectors of different or zero length".into());
        }
        for ((k, _), v) in batch.iter().zip(vecs) {
            cache.insert(model, k, normalize(v));
        }
        stats.fetched += batch.len();
        cache.flush().map_err(|e| e.to_string())?;
    }
    Ok(stats)
}

/// Attach a unit-length embedding of every unit's name (cache first, then the endpoint in batches).
pub fn embed_unit_names(
    units: &mut [Unit],
    cfg: &EmbedConfig,
    cache: &mut EmbeddingCache,
) -> Result<EmbedStats, String> {
    let model = cfg.model_id();
    let phrases: Vec<String> = units
        .iter()
        .map(|u| name_phrase(&u.name).chars().take(256).collect())
        .collect();
    let wanted = phrases
        .iter()
        .filter(|p| !p.is_empty())
        .map(|p| (p.clone(), p.clone()))
        .collect();
    let stats = fetch_missing(cfg, cache, &model, &wanted, 64)?;
    for (u, p) in units.iter_mut().zip(&phrases) {
        u.name_vec = cache.get(&model, p);
    }
    Ok(stats)
}

/// Unit-length embedding of one free-text query (for searching units by description).
pub fn embed_query(cfg: &EmbedConfig, text: &str) -> Result<Vec<f32>, String> {
    let mut rows = cfg.embed_batch_as(&[text.to_string()], true)?;
    rows.pop()
        .map(normalize)
        .ok_or_else(|| "empty embedding response".into())
}

/// Cache key of a unit text: two FNV-1a 64-bit hashes with different offsets (128 bits, stable
/// across Rust releases, unlike `DefaultHasher`) plus the length.
fn code_key(text: &str) -> String {
    let fnv = |seed: u64| {
        text.bytes()
            .fold(seed, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
    };
    format!(
        "code:{:016x}{:016x}:{}",
        fnv(0xcbf2_9ce4_8422_2325),
        fnv(0x8422_2325_cbf2_9ce4),
        text.len()
    )
}

/// Attach a unit-length embedding of every unit's source text (cache first, then the endpoint in
/// small batches; texts are cut at `max_chars`). Identical texts are embedded once.
pub fn embed_unit_code(
    units: &mut [Unit],
    cfg: &EmbedConfig,
    cache: &mut EmbeddingCache,
    max_chars: usize,
) -> Result<EmbedStats, String> {
    let model = format!("{}:code", cfg.model_id());
    let keys: Vec<Option<String>> = units
        .iter()
        .map(|u| {
            let text = &u.text[..u
                .text
                .char_indices()
                .nth(max_chars)
                .map_or(u.text.len(), |c| c.0)];
            (!text.trim().is_empty()).then(|| code_key(text))
        })
        .collect();
    let wanted = units
        .iter()
        .zip(&keys)
        .filter_map(|(u, k)| {
            let k = k.as_ref()?;
            let end = u
                .text
                .char_indices()
                .nth(max_chars)
                .map_or(u.text.len(), |c| c.0);
            Some((k.clone(), u.text[..end].to_string()))
        })
        .collect();
    let stats = fetch_missing(cfg, cache, &model, &wanted, 16)?;
    for (u, k) in units.iter_mut().zip(&keys) {
        u.vec = k.as_ref().and_then(|k| cache.get(&model, k));
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Minimal fake embeddings server: 3-dim vectors derived from the text; records requests.
    fn fake_server(requests: Arc<AtomicUsize>, seen: Arc<std::sync::Mutex<Vec<String>>>) -> String {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        std::thread::spawn(move || {
            for s in l.incoming() {
                let mut s = s.unwrap();
                let mut buf = vec![0u8; 65536];
                let mut n = 0;
                let (head_end, clen) = loop {
                    n += s.read(&mut buf[n..]).unwrap();
                    let t = String::from_utf8_lossy(&buf[..n]).to_string();
                    if let Some(p) = t.find("\r\n\r\n") {
                        let cl = t
                            .lines()
                            .find_map(|l| {
                                l.to_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|v| v.trim().parse::<usize>().unwrap())
                            })
                            .unwrap_or(0);
                        if n >= p + 4 + cl {
                            break (p + 4, cl);
                        }
                    }
                };
                let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
                let body: serde_json::Value =
                    serde_json::from_slice(&buf[head_end..head_end + clen]).unwrap();
                requests.fetch_add(1, Ordering::SeqCst);
                seen.lock()
                    .unwrap()
                    .push(format!("{}\n{}", head.lines().next().unwrap(), body));
                let data: Vec<_> = body["input"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let t = t.as_str().unwrap();
                        let sum: u32 = t.bytes().map(|b| b as u32).sum();
                        serde_json::json!({"index": i, "embedding": [t.len() as f32, (sum % 97) as f32, 1.0]})
                    })
                    .collect();
                let resp = serde_json::json!({ "data": data }).to_string();
                write!(s, "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}", resp.len(), resp).unwrap();
            }
        });
        format!("http://{addr}")
    }

    #[test]
    fn embeds_names_and_caches_in_flat_file() {
        let (reqs, seen) = (
            Arc::new(AtomicUsize::new(0)),
            Arc::new(std::sync::Mutex::new(vec![])),
        );
        let endpoint = fake_server(reqs.clone(), seen.clone());
        let cfg = EmbedConfig::new(
            endpoint,
            "emb-small".into(),
            Some("k".into()),
            None,
            Some(3),
        );
        assert_eq!(cfg.style, Style::OpenAi);
        let path = std::env::temp_dir().join(format!("dc-embed-test-{}.bin", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mk = || {
            ["fetchUserById", "fetch_user_by_id", "getInitials"]
                .iter()
                .map(|n| {
                    crate::extract_units(
                        "a.py",
                        crate::Lang::Python,
                        &format!("def {n}(x):\n    return x + 1\n"),
                    )
                    .remove(0)
                })
                .collect::<Vec<_>>()
        };
        let mut units = mk();
        let mut cache = EmbeddingCache::load(&path);
        let st = embed_unit_names(&mut units, &cfg, &mut cache).unwrap();
        assert_eq!((st.distinct_names, st.fetched, st.from_cache), (2, 2, 0)); // camel/snake coincide
        assert_eq!(reqs.load(Ordering::SeqCst), 1);
        assert!(units.iter().all(|u| u.name_vec.is_some()));
        let n: f32 = units[0]
            .name_vec
            .as_ref()
            .unwrap()
            .iter()
            .map(|x| x * x)
            .sum();
        assert!((n - 1.0).abs() < 1e-5, "vectors are unit length");
        assert!(
            seen.lock().unwrap()[0].contains("\"dimensions\":3")
                && seen.lock().unwrap()[0].contains("POST /embeddings")
        );

        // a fresh process: everything comes from the flat file, no request
        let mut units2 = mk();
        let mut cache2 = EmbeddingCache::load(&path);
        let st2 = embed_unit_names(&mut units2, &cfg, &mut cache2).unwrap();
        assert_eq!((st2.fetched, st2.from_cache), (0, 2));
        assert_eq!(reqs.load(Ordering::SeqCst), 1);
        assert_eq!(units[2].name_vec, units2[2].name_vec);
        // a different model does not reuse the vectors
        let cfg2 = EmbedConfig::new(
            cfg.endpoint.clone(),
            "other".into(),
            Some("k".into()),
            None,
            Some(3),
        );
        let st3 = embed_unit_names(&mut mk(), &cfg2, &mut EmbeddingCache::load(&path)).unwrap();
        assert_eq!(st3.fetched, 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn embeds_unit_code_once_per_distinct_text() {
        let (reqs, seen) = (
            Arc::new(AtomicUsize::new(0)),
            Arc::new(std::sync::Mutex::new(vec![])),
        );
        let cfg = EmbedConfig::new(
            fake_server(reqs.clone(), seen),
            "m".into(),
            Some("k".into()),
            None,
            Some(3),
        );
        let path = std::env::temp_dir().join(format!("dc-embed-code-{}.bin", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut units: Vec<Unit> = ["a", "b", "a"]
            .iter()
            .enumerate()
            .flat_map(|(i, n)| {
                crate::extract_units(
                    &format!("{i}.py"),
                    crate::Lang::Python,
                    &format!(
                        "def f(x):\n    return x + {}\n",
                        if *n == "a" { 1 } else { 2 }
                    ),
                )
            })
            .collect();
        let mut cache = EmbeddingCache::load(&path);
        let st = embed_unit_code(&mut units, &cfg, &mut cache, 3000).unwrap();
        assert_eq!((st.distinct_names, st.fetched), (2, 2));
        assert!(units.iter().all(|u| u.vec.is_some()));
        assert_eq!(units[0].vec, units[2].vec);
        assert_ne!(units[0].vec, units[1].vec);
        let st2 =
            embed_unit_code(&mut units, &cfg, &mut EmbeddingCache::load(&path), 3000).unwrap();
        assert_eq!((st2.fetched, st2.from_cache), (0, 2));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn detects_provider_style_and_urls() {
        let az = EmbedConfig::new(
            "https://r.openai.azure.com/".into(),
            "dep".into(),
            Some("k".into()),
            None,
            None,
        );
        assert_eq!(az.style, Style::AzureOpenAi);
        assert_eq!(
            az.url(),
            "https://r.openai.azure.com/openai/deployments/dep/embeddings?api-version=2024-10-21"
        );
        assert!(az.body(&["a".into()], false).get("model").is_none());
        let f = EmbedConfig::new(
            "https://r.services.ai.azure.com".into(),
            "text-embedding-3-small".into(),
            Some("k".into()),
            None,
            Some(256),
        );
        assert_eq!(f.style, Style::Foundry);
        assert_eq!(
            f.url(),
            "https://r.services.ai.azure.com/models/embeddings?api-version=2024-05-01-preview"
        );
        assert_eq!(
            f.body(&["a".into()], false)["model"],
            "text-embedding-3-small"
        );
        assert_eq!(f.body(&["a".into()], false)["dimensions"], 256);
    }

    #[test]
    fn api_key_header_depends_on_style() {
        let mk = |e: &str| EmbedConfig::new(e.into(), "m".into(), Some("sk-1".into()), None, None);
        assert_eq!(
            mk("https://api.openai.com/v1").auth().unwrap(),
            Some(("Authorization".into(), "Bearer sk-1".into()))
        );
        assert_eq!(
            mk("https://r.openai.azure.com").auth().unwrap(),
            Some(("api-key".into(), "sk-1".into()))
        );
    }

    #[test]
    fn cohere_style_url_body_auth_and_response() {
        let c = EmbedConfig::new(
            "https://api.cohere.com/".into(),
            "embed-v4.0".into(),
            Some("co-key".into()),
            None,
            Some(256),
        );
        assert_eq!(c.style, Style::Cohere);
        assert_eq!(c.url(), "https://api.cohere.com/v2/embed");
        assert_eq!(
            c.auth().unwrap(),
            Some(("Authorization".into(), "Bearer co-key".into()))
        );
        let b = c.body(&["x".into(), "y".into()], false);
        assert_eq!(b["texts"].as_array().unwrap().len(), 2);
        assert_eq!(b["input_type"], "search_document");
        assert_eq!(b["output_dimension"], 256);
        // v3 models do not accept output_dimension
        let v3 = EmbedConfig::new(
            "https://api.cohere.com".into(),
            "embed-english-v3.0".into(),
            None,
            None,
            Some(256),
        );
        assert!(v3
            .body(&["x".into()], false)
            .get("output_dimension")
            .is_none());
        let rows =
            parse_cohere(&serde_json::json!({"embeddings": {"float": [[1.0, 2.0], [3.0, 4.0]]}}))
                .unwrap();
        assert_eq!(rows, vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        assert!(parse_cohere(&serde_json::json!({"embeddings": {}})).is_err());
    }

    #[test]
    fn keyless_openai_style_endpoints_get_no_credential() {
        // never fall back to an Azure token for a local / third-party / Cohere endpoint
        for ep in ["http://127.0.0.1:8080/v1", "https://api.cohere.com"] {
            let c = EmbedConfig::new(ep.into(), "m".into(), None, None, None);
            assert_eq!(c.auth().unwrap(), None, "{ep}");
        }
    }

    #[test]
    fn api_key_comes_from_the_provider_of_the_endpoint() {
        let env = |pairs: &'static [(&str, &str)]| {
            move |n: &str| pairs.iter().find(|p| p.0 == n).map(|p| p.1.to_string())
        };
        let all: &[(&str, &str)] = &[
            ("OPENAI_API_KEY", "sk-openai"),
            ("COHERE_API_KEY", "co-cohere"),
            ("AZURE_OPENAI_API_KEY", "az-openai"),
            ("AZURE_AI_FOUNDRY_API_KEY", "az-foundry"),
        ];
        assert_eq!(
            pick_key(Style::Cohere, env(all)).as_deref(),
            Some("co-cohere")
        );
        assert_eq!(
            pick_key(Style::OpenAi, env(all)).as_deref(),
            Some("sk-openai")
        );
        assert_eq!(
            pick_key(Style::AzureOpenAi, env(all)).as_deref(),
            Some("az-openai")
        );
        assert_eq!(
            pick_key(Style::Foundry, env(all)).as_deref(),
            Some("az-foundry")
        );
        // an Azure-only environment gives a Cohere or OpenAI endpoint no key
        let azure_only: &[(&str, &str)] = &[("AZURE_OPENAI_API_KEY", "az-openai")];
        assert_eq!(pick_key(Style::OpenAi, env(azure_only)), None);
        assert_eq!(pick_key(Style::Cohere, env(azure_only)), None);
        // the explicit override wins for every style
        let both: &[(&str, &str)] = &[
            ("DUPLICATECODE_EMBED_API_KEY", "explicit"),
            ("OPENAI_API_KEY", "sk-openai"),
        ];
        assert_eq!(
            pick_key(Style::Cohere, env(both)).as_deref(),
            Some("explicit")
        );
    }

    #[test]
    fn host_and_loopback_detection() {
        let h = |e: &str| EmbedConfig::new(e.into(), "m".into(), None, None, None);
        assert_eq!(h("http://127.0.0.1:8099/v1").host(), "127.0.0.1");
        assert!(h("http://127.0.0.1:8099/v1").is_loopback());
        assert!(h("http://localhost/v1").is_loopback());
        assert!(h("http://[::1]:9/v1").is_loopback());
        assert!(!h("https://x.openai.azure.com").is_loopback());
        assert_eq!(h("https://user@api.cohere.com/v2").host(), "api.cohere.com");
        assert!(h("http://10.0.0.5/v1").is_cleartext_remote());
        assert!(!h("http://127.0.0.1/v1").is_cleartext_remote());
        assert!(!h("https://api.openai.com/v1").is_cleartext_remote());
    }

    #[test]
    fn empty_or_non_numeric_embeddings_are_rejected_not_cached() {
        let bad = serde_json::json!({"data": [{"index": 0}]});
        assert!(parse_openai(&bad).is_err());
        let empty = serde_json::json!({"data": [{"index": 0, "embedding": []}]});
        assert!(parse_openai(&empty).is_err());
        let text = serde_json::json!({"data": [{"index": 0, "embedding": "AAAA"}]});
        assert!(parse_openai(&text).is_err());
        let mixed = serde_json::json!({"data": [{"index": 0, "embedding": [1.0, "x"]}]});
        assert!(parse_openai(&mixed).is_err());
        assert!(parse_cohere(&serde_json::json!({"embeddings": {"float": [[]]}})).is_err());
        let ok = serde_json::json!({"data": [{"index": 1, "embedding": [3.0]}, {"index": 0, "embedding": [1.0, 2.0]}]});
        assert_eq!(parse_openai(&ok).unwrap(), vec![vec![1.0, 2.0], vec![3.0]]);
    }

    #[test]
    fn a_corrupt_tail_is_cut_off_so_new_records_stay_readable() {
        let path = std::env::temp_dir().join(format!("dc-embed-tail-{}.bin", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut c = EmbeddingCache::load(&path);
        c.insert("m", "first", vec![1.0, 0.0]);
        c.flush().unwrap();
        // simulate a crash that left half a record behind
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        f.write_all(&[9, 0, 1, 2, 3]).unwrap();
        drop(f);
        let mut c = EmbeddingCache::load(&path);
        assert!(c.get("m", "first").is_some());
        c.insert("m", "second", vec![0.0, 1.0]);
        c.flush().unwrap();
        let c = EmbeddingCache::load(&path);
        assert!(c.get("m", "first").is_some(), "old record survives");
        assert!(
            c.get("m", "second").is_some(),
            "record appended after the bad tail is readable"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn oversized_entries_stay_in_memory_and_do_not_desync_the_file() {
        let path = std::env::temp_dir().join(format!("dc-embed-big-{}.bin", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut c = EmbeddingCache::load(&path);
        c.insert("m", &"x".repeat(70_000), vec![1.0]);
        c.insert("m", "small", vec![2.0]);
        c.flush().unwrap();
        let c = EmbeddingCache::load(&path);
        assert!(c.get("m", "small").is_some());
        assert!(c.get("m", &"x".repeat(70_000)).is_none());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn debug_output_hides_the_api_key() {
        let c = EmbedConfig::new(
            "https://api.openai.com/v1".into(),
            "m".into(),
            Some("sk-secret".into()),
            None,
            None,
        );
        assert!(!format!("{c:?}").contains("sk-secret"));
    }

    #[test]
    fn cohere_queries_use_search_query_and_documents_search_document() {
        let c = EmbedConfig::new(
            "https://api.cohere.com".into(),
            "embed-v4.0".into(),
            None,
            None,
            None,
        );
        assert_eq!(c.body(&["x".into()], true)["input_type"], "search_query");
        assert_eq!(
            c.body(&["x".into()], false)["input_type"],
            "search_document"
        );
    }

    #[test]
    fn truncated_cache_tail_is_ignored() {
        let path = std::env::temp_dir().join(format!("dc-embed-trunc-{}.bin", std::process::id()));
        let mut c = EmbeddingCache::load(&path);
        c.insert("m", "hello world", vec![1.0, 0.0]);
        c.flush().unwrap();
        let mut b = std::fs::read(&path).unwrap();
        b.extend([9, 0, 1]); // garbage tail
        std::fs::write(&path, b).unwrap();
        assert!(EmbeddingCache::load(&path)
            .get("m", "hello world")
            .is_some());
        let _ = std::fs::remove_file(&path);
    }
}
