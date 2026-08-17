extern crate alloc;

use aidoku::{
    error::{Error, Result},
    prelude::*,
    std::{String, Vec},
    DeepLink, Filter, FilterType,
};

// Manual i32 to string conversion for no_std
pub fn i32_to_string(mut n: i32) -> String {
    if n == 0 {
        return String::from("0");
    }
    
    let negative = n < 0;
    if negative {
        n = -n;
    }
    
    let mut buf = Vec::new();
    while n > 0 {
        buf.push((b'0' + (n % 10) as u8) as char);
        n /= 10;
    }
    
    if negative {
        buf.push('-');
    }
    
    buf.reverse();
    buf.into_iter().collect()
}

pub fn get_filter_string(filters: Vec<Filter>) -> String {
    let mut query = String::new();
    let mut tags_included = Vec::new();
    let mut tags_excluded = Vec::new();
    
    for filter in filters {
        match filter.kind {
            FilterType::Title => {
                if let Ok(value) = filter.value.as_string() {
                    let title = value.read();
                    if !title.is_empty() {
                        query.push_str("&title=");
                        query.push_str(&urlencode(&title));
                    }
                }
            }
            FilterType::Author => {
                if let Ok(value) = filter.value.as_string() {
                    let author = value.read();
                    if !author.is_empty() {
                        query.push_str("&authors=");
                        query.push_str(&urlencode(&author));
                    }
                }
            }
            FilterType::Genre => {
                if let Ok(value) = filter.value.as_int() {
                    match value {
                        0 => {} // None
                        1 => tags_included.push(filter.name.clone()),
                        2 => tags_excluded.push(filter.name.clone()),
                        _ => {}
                    }
                }
            }
            FilterType::Select => {
                if filter.name == "Status" {
                    if let Ok(value) = filter.value.as_int() {
                        let status = match value {
                            0 => "ongoing",
                            1 => "completed",
                            2 => "hiatus",
                            3 => "cancelled",
                            _ => "",
                        };
                        if !status.is_empty() {
                            query.push_str("&status[]=");
                            query.push_str(status);
                        }
                    }
                } else if filter.name == "Demographic" {
                    if let Ok(value) = filter.value.as_int() {
                        let demo = match value {
                            0 => "shounen",
                            1 => "shoujo",
                            2 => "seinen",
                            3 => "josei",
                            _ => "",
                        };
                        if !demo.is_empty() {
                            query.push_str("&publicationDemographic[]=");
                            query.push_str(demo);
                        }
                    }
                } else if filter.name == "Content Rating" {
                    if let Ok(value) = filter.value.as_int() {
                        match value {
                            0 => {
                                query.push_str("&contentRating[]=safe");
                                query.push_str("&contentRating[]=suggestive");
                            }
                            1 => {
                                query.push_str("&contentRating[]=erotica");
                            }
                            2 => {
                                query.push_str("&contentRating[]=pornographic");
                            }
                            _ => {}
                        }
                    }
                }
            }
            FilterType::Sort => {
                if let Ok(obj) = filter.value.as_object() {
                    if let Ok(index) = obj.get("index").as_int() {
                        let ascending = obj.get("ascending").as_bool().unwrap_or(false);
                        let order_dir = if ascending { "asc" } else { "desc" };
                        
                        let order_by = match index {
                            0 => "latestUploadedChapter",
                            1 => "title",
                            2 => "rating",
                            3 => "followedCount",
                            4 => "createdAt",
                            5 => "year",
                            _ => "latestUploadedChapter",
                        };
                        
                        query.push_str("&order[");
                        query.push_str(order_by);
                        query.push_str("]=");
                        query.push_str(order_dir);
                    }
                }
            }
            _ => {}
        }
    }
    
    for tag in tags_included {
        query.push_str("&includedTags[]=");
        query.push_str(&get_tag_id(&tag));
    }
    
    for tag in tags_excluded {
        query.push_str("&excludedTags[]=");
        query.push_str(&get_tag_id(&tag));
    }
    
    query
}

pub fn parse_incoming_url(url: String) -> Result<DeepLink> {
    // Parse URLs like: https://mangadex.org/title/{id}
    if url.contains("/title/") {
        let parts: Vec<&str> = url.split("/title/").collect();
        if parts.len() > 1 {
            let id = parts[1].split('/').next().unwrap_or("");
            if !id.is_empty() {
                return Ok(DeepLink {
                    manga: Some(String::from(id)),
                    chapter: None,
                });
            }
        }
    }
    
    // Parse chapter URLs: https://mangadex.org/chapter/{id}
    if url.contains("/chapter/") {
        let parts: Vec<&str> = url.split("/chapter/").collect();
        if parts.len() > 1 {
            let id = parts[1].split('/').next().unwrap_or("");
            if !id.is_empty() {
                return Ok(DeepLink {
                    manga: None,
                    chapter: Some(String::from(id)),
                });
            }
        }
    }
    
    Err(Error::new("Invalid URL"))
}

fn get_tag_id(tag_name: &str) -> &str {
    match tag_name {
        "Action" => "391b0423-d847-456f-aff0-8b0cfc03066b",
        "Adventure" => "87cc87cd-a395-47af-b27a-93258283bbc6",
        "Comedy" => "4d32cc48-9f00-4cca-9b5a-a839f0764984",
        "Drama" => "b9af3a63-f058-46de-a9a0-e0c13906197a",
        "Fantasy" => "cdc58593-87dd-415e-bbc0-2ec27bf404cc",
        "Horror" => "cdad7e68-1419-41dd-bdce-27753074a640",
        "Mystery" => "ee968100-4191-4968-93d3-f82d72be7e46",
        "Romance" => "423e2eae-a7a2-4a8b-ac03-a8351462d71d",
        "Sci-Fi" => "256c8bd9-4904-4360-bf4f-508a76d67183",
        "Slice of Life" => "e5301a23-ebd9-49dd-a0cb-2add944c7fe9",
        "Sports" => "69964a64-2f90-4d33-beeb-f3ed2875eb4c",
        "Supernatural" => "eabc5b4c-6aff-42f3-b657-3e90cbd00b75",
        "Thriller" => "07251805-a27e-4d59-b488-f0bfbec15168",
        _ => "",
    }
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
