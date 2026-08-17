use aidoku::{
    FilterValue,
    alloc::{String, Vec, string::ToString},
};

pub fn urlencode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push('%');
                let hex_chars = b"0123456789ABCDEF";
                result.push(hex_chars[(byte >> 4) as usize] as char);
                result.push(hex_chars[(byte & 0x0F) as usize] as char);
            }
        }
    }
    result
}

pub fn get_filter_string(filters: Vec<FilterValue>) -> String {
    let mut query = String::new();
    
    for filter in filters {
        match filter {
            FilterValue::Title { value } => {
                query.push_str("&title=");
                query.push_str(&urlencode(&value));
            }
            FilterValue::Author { value } => {
                query.push_str("&authors=");
                query.push_str(&urlencode(&value));
            }
            FilterValue::Genre { included, excluded, .. } => {
                for tag in included {
                    query.push_str("&includedTags[]=");
                    query.push_str(&urlencode(&tag));
                }
                for tag in excluded {
                    query.push_str("&excludedTags[]=");
                    query.push_str(&urlencode(&tag));
                }
            }
            FilterValue::Sort { index, ascending, .. } => {
                let order = match index {
                    0 => "latestUploadedChapter",
                    1 => "title",
                    2 => "year",
                    3 => "createdAt",
                    4 => "followedCount",
                    5 => "relevance",
                    6 => "rating",
                    _ => "latestUploadedChapter",
                };
                query.push_str("&order[");
                query.push_str(order);
                query.push_str("]=");
                query.push_str(if ascending { "asc" } else { "desc" });
            }
            FilterValue::Select { id, value } => {
                query.push_str("&");
                query.push_str(&id);
                query.push_str("=");
                query.push_str(&urlencode(&value));
            }
            _ => {}
        }
    }
    
    query
}
