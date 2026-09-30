//! Optional semantic name similarity via an embeddings endpoint (Azure AI Foundry / Azure OpenAI /
//! any OpenAI-compatible API), cached in an append-only flat file so each distinct name is embedded
//! at most once per model.

use crate::naming::split_identifier;
use crate::units::Unit;
use std::collections::HashMap;
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
}

#[derive(Clone, Debug)]
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

fn env_first(names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|n| std::env::var(n).ok().filter(|v| !v.is_empty()))
}

impl EmbedConfig {
    /// Reads `AZURE_AI_FOUNDRY_ENDPOINT` (or `AZURE_OPENAI_ENDPOINT`, `DUPLICATECODE_EMBED_ENDPOINT`),
    /// `…_API_KEY`, `…_EMBEDDING_DEPLOYMENT` and `…_API_VERSION`. Without an API key an Entra ID token
    /// from `az account get-access-token` is used.
    pub fn from_env(dims: Option<u32>) -> Option<EmbedConfig> {
        let endpoint = env_first(&[
            "AZURE_AI_FOUNDRY_ENDPOINT",
            "AZURE_OPENAI_ENDPOINT",
            "DUPLICATECODE_EMBED_ENDPOINT",
        ])?;
        Some(EmbedConfig::new(
            endpoint,
            env_first(&[
                "AZURE_AI_FOUNDRY_EMBEDDING_DEPLOYMENT",
                "AZURE_OPENAI_EMBEDDING_DEPLOYMENT",
                "DUPLICATECODE_EMBED_MODEL",
            ])
            .unwrap_or_else(|| "text-embedding-3-small".into()),
            env_first(&[
                "AZURE_AI_FOUNDRY_API_KEY",
                "AZURE_OPENAI_API_KEY",
                "DUPLICATECODE_EMBED_API_KEY",
            ]),
            env_first(&["AZURE_AI_FOUNDRY_API_VERSION", "AZURE_OPENAI_API_VERSION"]),
            dims,
        ))
    }

    pub fn new(
        endpoint: String,
        deployment: String,
        api_key: Option<String>,
        api_version: Option<String>,
        dims: Option<u32>,
    ) -> Self {
        let e = endpoint.trim_end_matches('/').to_string();
        let style = if e.contains(".openai.azure.com") {
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
        }
    }

    fn body(&self, texts: &[String]) -> serde_json::Value {
        let mut b = serde_json::json!({ "input": texts });
        if self.style != Style::AzureOpenAi {
            b["model"] = self.deployment.clone().into();
        }
        if let Some(d) = self.dims {
            b["dimensions"] = d.into();
        }
        b
    }

    fn auth(&self) -> Result<(String, String), String> {
        if let Some(k) = &self.api_key {
            return Ok(("api-key".into(), k.clone()));
        }
        if let Some(t) = env_first(&["AZURE_AI_FOUNDRY_TOKEN"]) {
            return Ok(("Authorization".into(), format!("Bearer {t}")));
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
        Ok(("Authorization".into(), format!("Bearer {t}")))
    }

    /// Embed one batch. Retries on throttling / transient server errors.
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        let (hk, hv) = self.auth()?;
        let url = self.url();
        let body = self.body(texts);
        let mut last = String::new();
        for attempt in 0..4 {
            match ureq::post(&url).set(&hk, &hv).send_json(body.clone()) {
                Ok(resp) => {
                    let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
                    let mut rows: Vec<(usize, Vec<f32>)> = v["data"]
                        .as_array()
                        .ok_or("response has no `data` array")?
                        .iter()
                        .map(|d| {
                            let idx = d["index"].as_u64().unwrap_or(0) as usize;
                            let e = d["embedding"]
                                .as_array()
                                .map(|a| {
                                    a.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect()
                                })
                                .unwrap_or_default();
                            (idx, e)
                        })
                        .collect();
                    rows.sort_by_key(|r| r.0);
                    if rows.len() != texts.len() {
                        return Err(format!(
                            "expected {} embeddings, got {}",
                            texts.len(),
                            rows.len()
                        ));
                    }
                    return Ok(rows.into_iter().map(|r| r.1).collect());
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

/// Append-only flat-file cache: repeated records of
/// `u16 model_len | model | u16 text_len | text | u16 dims | dims * f32 (little endian)`.
/// Later records win; unreadable tails are ignored (the file stays usable after a crash).
pub struct EmbeddingCache {
    path: PathBuf,
    map: HashMap<(String, String), Arc<[f32]>>,
    pending: Vec<(String, String)>,
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
                map.insert((model, text), Arc::from(v));
            }
        }
        EmbeddingCache {
            path: path.to_path_buf(),
            map,
            pending: Vec::new(),
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

    /// Append everything inserted since the last flush.
    pub fn flush(&mut self) -> std::io::Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let mut buf = Vec::new();
        for key in self.pending.drain(..) {
            let v = &self.map[&key];
            for s in [&key.0, &key.1] {
                buf.extend((s.len() as u16).to_le_bytes());
                buf.extend(s.as_bytes());
            }
            buf.extend((v.len() as u16).to_le_bytes());
            for x in v.iter() {
                buf.extend(x.to_le_bytes());
            }
        }
        f.write_all(&buf)
    }
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

/// Attach a unit-length embedding of every unit's name (cache first, then the endpoint in batches).
pub fn embed_unit_names(
    units: &mut [Unit],
    cfg: &EmbedConfig,
    cache: &mut EmbeddingCache,
) -> Result<EmbedStats, String> {
    let model = cfg.model_id();
    let mut phrases: Vec<String> = units
        .iter()
        .map(|u| name_phrase(&u.name))
        .filter(|p| !p.is_empty())
        .collect();
    phrases.sort();
    phrases.dedup();
    let mut stats = EmbedStats {
        distinct_names: phrases.len(),
        ..Default::default()
    };
    let missing: Vec<String> = phrases
        .iter()
        .filter(|p| cache.get(&model, p).is_none())
        .cloned()
        .collect();
    stats.from_cache = phrases.len() - missing.len();
    for chunk in missing.chunks(64) {
        let vecs = cfg.embed_batch(chunk)?;
        for (t, v) in chunk.iter().zip(vecs) {
            cache.insert(&model, t, normalize(v));
        }
        stats.fetched += chunk.len();
        cache.flush().map_err(|e| e.to_string())?; // keep progress if a later batch fails
    }
    for u in units.iter_mut() {
        u.name_vec = cache.get(&model, &name_phrase(&u.name));
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write as _};
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
        assert!(az.body(&["a".into()]).get("model").is_none());
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
        assert_eq!(f.body(&["a".into()])["model"], "text-embedding-3-small");
        assert_eq!(f.body(&["a".into()])["dimensions"], 256);
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
