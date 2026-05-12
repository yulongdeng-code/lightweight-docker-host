use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedOutput {
    pub original: String,
    pub compressed: String,
    pub savings_percent: f32,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
}

#[derive(Debug, Error)]
pub enum CompressError {
    #[error("HTML parsing failed: {0}")]
    HtmlParseError(String),
    #[error("Invalid URL: {0}")]
    UrlError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMParams {
    pub temperature: f32,
    pub max_tokens: usize,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub tokens_used: usize,
}

#[derive(Debug, Error)]
pub enum LLMError {
    #[error("Compression error: {0}")]
    Compress(#[from] CompressError),
    #[error("LLM call failed: {0}")]
    CallFailed(String),
}

pub trait TokenCompressor: Send + Sync {
    fn compress(&self, input: &str) -> Result<CompressedOutput, CompressError>;
    fn html_to_markdown(&self, html: &str) -> Result<String, CompressError>;
    fn shorten_urls(&self, text: &str) -> Result<String, CompressError>;
    fn remove_non_ascii(&self, text: &str) -> String;
    fn calculate_savings(&self, original: &str, compressed: &str) -> f32;
}

pub struct TokenJuiceCompressor;

impl TokenJuiceCompressor {
    pub fn new() -> Self {
        Self
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count() * 4 / 3
    }

    fn remove_whitespace(&self, text: &str) -> String {
        let re = Regex::new(r"\s+").unwrap();
        re.replace_all(text, " ").trim().to_string()
    }

    fn remove_comments(&self, text: &str) -> String {
        let re = Regex::new(r"//.*$|/\*[\s\S]*?\*/").unwrap();
        re.replace_all(text, "").to_string()
    }

    fn truncate_long_text(&self, text: &str, max_len: usize) -> String {
        if text.len() > max_len {
            format!("{}... [truncated]", &text[0..max_len])
        } else {
            text.to_string()
        }
    }
}

impl TokenCompressor for TokenJuiceCompressor {
    fn compress(&self, input: &str) -> Result<CompressedOutput, CompressError> {
        let mut compressed = input.to_string();

        compressed = self.remove_non_ascii(&compressed);
        compressed = self.shorten_urls(&compressed)?;
        compressed = self.remove_comments(&compressed);
        compressed = self.remove_whitespace(&compressed);
        compressed = self.truncate_long_text(&compressed, 10000);

        let original_tokens = self.estimate_tokens(input);
        let compressed_tokens = self.estimate_tokens(&compressed);
        let savings_percent = self.calculate_savings(input, &compressed);

        Ok(CompressedOutput {
            original: input.to_string(),
            compressed,
            savings_percent,
            original_tokens,
            compressed_tokens,
        })
    }

    fn html_to_markdown(&self, html: &str) -> Result<String, CompressError> {
        Ok(html2md::parse_html(html))
    }

    fn shorten_urls(&self, text: &str) -> Result<String, CompressError> {
        let re = Regex::new(r"https?://[^\s]+").unwrap();
        let mut result = text.to_string();
        for cap in re.captures_iter(text) {
            if let Some(url_str) = cap.get(0) {
                if let Ok(url) = Url::parse(url_str.as_str()) {
                    let short = format!("[{}]", url.domain().unwrap_or("url"));
                    result = result.replace(url_str.as_str(), &short);
                }
            }
        }
        Ok(result)
    }

    fn remove_non_ascii(&self, text: &str) -> String {
        text.chars()
            .filter(|c| c.is_ascii() || c.is_whitespace())
            .collect()
    }

    fn calculate_savings(&self, original: &str, compressed: &str) -> f32 {
        let orig_tokens = self.estimate_tokens(original) as f32;
        let comp_tokens = self.estimate_tokens(compressed) as f32;
        if orig_tokens > 0.0 {
            ((orig_tokens - comp_tokens) / orig_tokens) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for TokenJuiceCompressor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LLMInterceptor {
    compressor: Box<dyn TokenCompressor>,
}

impl LLMInterceptor {
    pub fn new() -> Self {
        Self {
            compressor: Box::new(TokenJuiceCompressor::new()),
        }
    }

    pub async fn call_llm(&self, prompt: &str, _params: LLMParams) -> Result<LLMResponse, LLMError> {
        let compressed = self.compressor.compress(prompt)?;
        Ok(LLMResponse {
            content: format!("Compressed input saved {}% tokens", compressed.savings_percent),
            tokens_used: compressed.compressed_tokens,
        })
    }
}

impl Default for LLMInterceptor {
    fn default() -> Self {
        Self::new()
    }
}
