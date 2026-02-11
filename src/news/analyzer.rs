use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use anyhow::Result;
use log::{info, warn};
use chrono::{DateTime, Utc};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct NewsAnalyzer {
    api_key: String,
    cache: HashMap<String, Vec<NewsArticle>>,
    last_fetch: Option<DateTime<Utc>>,
}

impl NewsAnalyzer {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            cache: HashMap::new(),
            last_fetch: None,
        }
    }
    
    pub async fn fetch_news(&mut self, source: &str, query: &str) -> Result<Vec<NewsArticle>> {
        // Check cache first
        let cache_key = format!("{}:{}", source, query);
        if let Some(cached) = self.cache.get(&cache_key) {
            if let Some(last_fetch) = self.last_fetch {
                if Utc::now().signed_duration_since(last_fetch).num_minutes() < 10 {
                    info!("📰 Using cached news for: {}", query);
                    return Ok(cached.clone());
                }
            }
        }
        
        info!("🌐 Fetching news from: {} (query: {})", source, query);

        let articles = if self.api_key == "test_news_key" || self.api_key.is_empty() {
            self.fetch_simulated_news(query)
        } else {
            match self.fetch_news_api(source).await {
                Ok(real_articles) if !real_articles.is_empty() => real_articles,
                Ok(_) => self.fetch_simulated_news(query),
                Err(err) => {
                    warn!("⚠️  News API fetch failed: {}. Falling back to simulated news.", err);
                    self.fetch_simulated_news(query)
                }
            }
        };
        
        // Cache the results
        self.cache.insert(cache_key, articles.clone());
        self.last_fetch = Some(Utc::now());
        
        info!("✅ Fetched {} news articles", articles.len());
        
        Ok(articles)
    }

    fn fetch_simulated_news(&self, query: &str) -> Vec<NewsArticle> {
        let mut articles = Vec::new();
        let article_count = 5 + rand::thread_rng().gen_range(0..10);

        for i in 0..article_count {
            let article = self.generate_simulated_article(query, i);
            articles.push(article);
        }

        articles
    }

    async fn fetch_news_api(&self, source: &str) -> Result<Vec<NewsArticle>> {
        let client = Client::new();
        let mut url = source.to_string();
        if !url.contains("apiKey=") {
            let joiner = if url.contains('?') { "&" } else { "?" };
            url = format!("{}{}apiKey={}", url, joiner, self.api_key);
        }

        let response = client.get(&url).send().await?;
        let body: Value = response.json().await?;

        let articles = body
            .get("articles")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let parsed = articles
            .into_iter()
            .enumerate()
            .map(|(idx, article)| {
                let title = article.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
                let description = article.get("description").and_then(|v| v.as_str()).unwrap_or("");
                let content = article.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let url = article.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let published_at = article
                    .get("publishedAt")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                    .unwrap_or_else(Utc::now);
                let source = article
                    .get("source")
                    .and_then(|v| v.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                NewsArticle {
                    id: format!("newsapi_{}_{}", source, idx),
                    title: title.to_string(),
                    description: description.to_string(),
                    content: content.to_string(),
                    url: url.to_string(),
                    published_at,
                    source: source.to_string(),
                    sentiment: 0.0,
                    keywords: vec!["news".to_string()],
                    impact_score: 0.5,
                }
            })
            .collect();

        Ok(parsed)
    }
    
    fn generate_simulated_article(&self, query: &str, id: usize) -> NewsArticle {
        let mut rng = rand::thread_rng();
        
        let titles = [
            format!("Market reacts to new {} developments", query),
            format!("Experts predict {} trends for next quarter", query),
            format!("Breaking: Major announcement affects {}", query),
            format!("{} sector shows strong growth", query),
            format!("Investors cautious about {} volatility", query),
            format!("New regulations impact {} market", query),
            format!("Technology disrupts traditional {}", query),
            format!("Global events influence {} prices", query),
            format!("Analysis: {} outlook for 2024", query),
            format!("Trading volume spikes in {} markets", query),
        ];
        
        let sources = [
            "Bloomberg",
            "Reuters",
            "Wall Street Journal",
            "Financial Times",
            "CNBC",
            "MarketWatch",
            "Yahoo Finance",
            "Seeking Alpha",
            "Investing.com",
            "CoinDesk",
        ];
        
        let descriptions = [
            "Significant developments are influencing market sentiment.",
            "Analysts are revising their forecasts based on new data.",
            "Traders are positioning themselves for upcoming volatility.",
            "Institutional investors are showing increased interest.",
            "Retail participation is at record levels in this sector.",
            "Regulatory changes are creating both risks and opportunities.",
            "Technological innovation is disrupting traditional models.",
            "Global macroeconomic factors are playing a key role.",
            "Market makers are adjusting their strategies.",
            "Liquidity conditions are affecting price discovery.",
        ];
        
        NewsArticle {
            id: format!("article_{}_{}", query, id),
            title: titles[id % titles.len()].clone(),
            description: descriptions[id % descriptions.len()].to_string(),
            content: format!("Detailed analysis of {} market conditions. Multiple factors are contributing to current price action. Experts suggest careful consideration of risk management strategies.", query),
            url: format!("https://news.example.com/{}/{}", query, id),
            published_at: Utc::now() - chrono::Duration::hours(rng.gen_range(0..72)),
            source: sources[id % sources.len()].to_string(),
            sentiment: rng.gen_range(-1.0..1.0),
            keywords: vec![query.to_string(), "market".to_string(), "trading".to_string()],
            impact_score: rng.gen_range(0.0..1.0),
        }
    }
    
    pub async fn analyze_sentiment(&self, text: &str) -> Result<f64> {
        // In production, this would call Claude API or use NLP library
        // For simulation, generate sentiment based on keywords
        
        let positive_keywords = [
            "bullish", "growth", "profit", "gain", "increase", "positive",
            "optimistic", "strong", "recovery", "rally", "surge", "boom"
        ];
        
        let negative_keywords = [
            "bearish", "loss", "decline", "drop", "negative", "pessimistic",
            "weak", "crash", "collapse", "risk", "volatile", "uncertain"
        ];
        
        let text_lower = text.to_lowercase();
        
        let mut score = 0.0;
        
        for keyword in positive_keywords {
            if text_lower.contains(keyword) {
                score += 0.2;
            }
        }
        
        for keyword in negative_keywords {
            if text_lower.contains(keyword) {
                score -= 0.2;
            }
        }
        
        // Add some randomness
        let random_factor: f64 = rand::thread_rng().gen_range(-0.3..0.3);
        score += random_factor;
        
        // Clamp between -1 and 1
        let sentiment = score.max(-1.0).min(1.0);
        
        Ok(sentiment)
    }
    
    pub fn extract_keywords(&self, text: &str) -> Vec<String> {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to",
            "for", "of", "with", "by", "is", "are", "was", "were", "be",
            "been", "being", "have", "has", "had", "do", "does", "did"
        ];
        
        text.split_whitespace()
            .filter(|word| !stop_words.contains(&word.to_lowercase().as_str()))
            .filter(|word| word.len() > 3)
            .map(|word| word.to_string())
            .collect()
    }
    
    pub fn calculate_impact_score(&self, article: &NewsArticle) -> f64 {
        // Calculate impact based on sentiment, source credibility, and recency
        let source_score = match article.source.as_str() {
            "Bloomberg" | "Reuters" | "Wall Street Journal" => 0.9,
            "Financial Times" | "CNBC" => 0.8,
            _ => 0.6,
        };
        
        let recency_hours = Utc::now().signed_duration_since(article.published_at).num_hours() as f64;
        let recency_score = if recency_hours < 1.0 {
            1.0
        } else if recency_hours < 24.0 {
            0.8
        } else if recency_hours < 72.0 {
            0.5
        } else {
            0.2
        };
        
        let sentiment_magnitude = article.sentiment.abs();
        
        (source_score * 0.4 + recency_score * 0.3 + sentiment_magnitude * 0.3) * article.impact_score
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewsArticle {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub url: String,
    pub published_at: DateTime<Utc>,
    pub source: String,
    pub sentiment: f64,
    pub keywords: Vec<String>,
    pub impact_score: f64,
}
