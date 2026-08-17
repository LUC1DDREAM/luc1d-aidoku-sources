extern crate alloc;

use aidoku::{
    error::{Error, Result},
    prelude::*,
    std::{String, Vec, html::Node},
    Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
};

pub fn parse_manga_listing(html: Node, _page: i32) -> Result<MangaPageResult> {
    let items = html.select("ul.card_lst li").array();
    let mut manga_list = Vec::new();
    
    for item in items {
        let node = item.as_node()?;
        
        let link = node.select("a").attr("href").read();
        let id = String::from(link.trim_start_matches("https://www.webtoons.com/")
            .trim_start_matches("http://www.webtoons.com/"));
        
        let title = node.select(".subj").text().read();
        let cover = node.select("img").attr("src").read();
        let author = node.select(".author").text().read();
        
        manga_list.push(Manga {
            id: id.clone(),
            cover,
            title,
            author,
            artist: String::new(),
            description: String::new(),
            url: {
                let mut url = String::from("https://www.webtoons.com/");
                url.push_str(&id);
                url
            },
            categories: Vec::new(),
            status: MangaStatus::Unknown,
            nsfw: MangaContentRating::Safe,
            viewer: MangaViewer::Scroll,
        });
    }
    
    Ok(MangaPageResult {
        manga: manga_list,
        has_more: false,
    })
}

pub fn parse_manga_details(html: Node, id: String) -> Result<Manga> {
    let title = html.select("h1.subj").text().read();
    let cover = html.select(".detail_header .thumb img").attr("src").read();
    let author = html.select(".detail_header .author").text().read();
    let description = html.select(".summary").text().read();
    
    let genre_text = html.select(".genre").text().read();
    let categories = genre_text.split(',')
        .map(|s| String::from(s.trim()))
        .filter(|s| !s.is_empty())
        .collect();
    
    let status = if html.select(".day_info .completed").text().read().contains("COMPLETED") {
        MangaStatus::Completed
    } else {
        MangaStatus::Ongoing
    };
    
    Ok(Manga {
        id: id.clone(),
        cover,
        title,
        author,
        artist: String::new(),
        description,
        url: {
            let mut url = String::from("https://www.webtoons.com/");
            url.push_str(&id);
            url
        },
        categories,
        status,
        nsfw: MangaContentRating::Safe,
        viewer: MangaViewer::Scroll,
    })
}

pub fn parse_chapter_list(html: Node) -> Result<Vec<Chapter>> {
    let items = html.select("#_listUl li").array();
    let mut chapters = Vec::new();
    
    for (index, item) in items.enumerate() {
        let node = item.as_node()?;
        
        let link = node.select("a").attr("href").read();
        let id = String::from(link.trim_start_matches("https://www.webtoons.com/")
            .trim_start_matches("http://www.webtoons.com/"));
        
        let title = node.select(".subj span").text().read();
        
        let chapter_num = (items.len() - index) as f32;
        
        let date_str = node.select(".date").text().read();
        let date_updated = parse_relative_date(&date_str);
        
        // Enhanced: Detect Fast Pass / Locked episodes
        let is_locked = node.select(".ico_locked").html().read().len() > 0;
        let is_fastpass = node.select(".ico_fastpass").html().read().len() > 0;
        
        let mut scanlator = String::new();
        if is_fastpass {
            scanlator.push_str("🔒 Fast Pass");
        } else if is_locked {
            scanlator.push_str("🔒 Locked");
        } else {
            scanlator.push_str("Free");
        }
        
        // Check for unlock date
        if let Ok(unlock_text) = node.select(".date._unlockDate").text().ok() {
            let unlock_str = unlock_text.read();
            if !unlock_str.is_empty() {
                scanlator.push_str(" • Unlocks ");
                scanlator.push_str(&unlock_str);
            }
        }
        
        chapters.push(Chapter {
            id,
            title,
            volume: -1.0,
            chapter: chapter_num,
            date_updated,
            scanlator,
            url: String::new(),
            lang: String::from("en"),
        });
    }
    
    Ok(chapters)
}

pub fn parse_page_list(html: Node) -> Result<Vec<Page>> {
    let viewer = html.select("#_imageList").array();
    let mut pages = Vec::new();
    
    for (index, item) in viewer.enumerate() {
        let node = item.as_node()?;
        let url = node.select("img").attr("data-url")
            .or_else(|_| node.select("img").attr("src"))
            .read();
        
        if !url.is_empty() {
            pages.push(Page {
                index: index as i32,
                url,
                base64: String::new(),
                text: String::new(),
            });
        }
    }
    
    Ok(pages)
}

fn parse_relative_date(date_str: &str) -> f64 {
    let now = aidoku::std::defaults::current_date();
    
    if date_str.contains("hour") {
        let hours: f64 = date_str.split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);
        return now - (hours * 3600.0);
    } else if date_str.contains("day") {
        let days: f64 = date_str.split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);
        return now - (days * 86400.0);
    } else if date_str.contains("week") {
        let weeks: f64 = date_str.split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);
        return now - (weeks * 604800.0);
    }
    
    // Try to parse as absolute date (MMM DD, YYYY)
    // Fallback to current date if parsing fails
    now
}
