extern crate alloc;

use aidoku::{
    error::{Error, Result},
    prelude::*,
    std::{net::Request, String, Vec},
    DeepLink, Filter, FilterType, MangaPageResult,
};
use alloc::string::ToString;

use crate::parser;

const BASE_URL: &str = "https://www.webtoons.com";

pub fn search_manga(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut title = String::new();
    
    for filter in filters {
        if filter.kind == FilterType::Title {
            if let Ok(value) = filter.value.as_string() {
                title = value.read();
            }
        }
    }
    
    if title.is_empty() {
        // No search query, return empty
        return Ok(MangaPageResult {
            manga: Vec::new(),
            has_more: false,
        });
    }
    
    let mut url = String::from(BASE_URL);
    url.push_str("/en/search?keyword=");
    url.push_str(&urlencode(&title));
    url.push_str("&searchType=WEBTOON");
    
    if page > 1 {
        url.push_str("&page=");
        url.push_str(&page.to_string());
    }
    
    let html = Request::new(&url, aidoku::std::net::HttpMethod::Get).html()?;
    parser::parse_manga_listing(html, page)
}

pub fn parse_incoming_url(url: String) -> Result<DeepLink> {
    // Parse URLs like: https://www.webtoons.com/en/genre/title/list?title_no=123
    if url.contains("/list?title_no=") {
        let id = url.trim_start_matches("https://www.webtoons.com/")
            .trim_start_matches("http://www.webtoons.com/")
            .split('?')
            .next()
            .unwrap_or("")
            .to_string();
        
        if !id.is_empty() {
            return Ok(DeepLink {
                manga: Some(id),
                chapter: None,
            });
        }
    }
    
    // Parse episode URLs
    if url.contains("/viewer?") {
        let id = url.trim_start_matches("https://www.webtoons.com/")
            .trim_start_matches("http://www.webtoons.com/")
            .split('&')
            .next()
            .unwrap_or("")
            .to_string();
        
        if !id.is_empty() {
            return Ok(DeepLink {
                manga: None,
                chapter: Some(id),
            });
        }
    }
    
    Err(Error::new("Invalid URL"))
}

fn urlencode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push('%');
                // Manual hex conversion for no_std
                let hex_chars = b"0123456789ABCDEF";
                result.push(hex_chars[(byte >> 4) as usize] as char);
                result.push(hex_chars[(byte & 0x0F) as usize] as char);
            }
        }
    }
    result
}
