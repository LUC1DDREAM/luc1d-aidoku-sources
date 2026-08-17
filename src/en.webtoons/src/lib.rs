#![no_std]

use aidoku::{
    prelude::*,
    Chapter, ContentRating, Filter, FilterValue, Listing, Manga, MangaPageResult, MangaStatus, 
    Page, PageContent, Result, Source, Viewer,
    alloc::{String, Vec, string::ToString, format},
    error::AidokuError,
    imports::{
        net::Request,
        std::parse_date,
    },
};

const BASE_URL: &str = "https://www.webtoons.com";

struct Webtoons;

impl Source for Webtoons {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let Some(title) = query else {
            return Ok(MangaPageResult {
                entries: Vec::new(),
                has_next_page: false,
            });
        };
        
        if title.is_empty() {
            return Ok(MangaPageResult {
                entries: Vec::new(),
                has_next_page: false,
            });
        }
        
        let mut url = format!("{BASE_URL}/en/search?keyword={}&searchType=WEBTOON", urlencode(&title));
        
        if page > 1 {
            url.push_str(&format!("&page={page}"));
        }
        
        let html = Request::get(&url)?.html()?;
        self.parse_manga_listing(html, page)
    }

    fn get_listing_manga_list(
        &self,
        listing: Listing,
        page: i32,
    ) -> Result<MangaPageResult> {
        let genre = match listing.name.as_str() {
            "Popular" => "popular",
            "Romance" => "romance",
            "Fantasy" => "fantasy",
            "Comedy" => "comedy",
            "Action" => "action",
            "Drama" => "drama",
            "Thriller" => "thriller",
            "Supernatural" => "supernatural",
            _ => "popular",
        };
        
        let url = format!("{BASE_URL}/en/genre/{genre}");
        let html = Request::get(&url)?.html()?;
        self.parse_manga_listing(html, page)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let url = format!("{BASE_URL}/{}", manga.key);
        let html = Request::get(&url)?.html()?;
        
        if needs_details {
            manga.title = html.select_first("h1.subj")
                .and_then(|el| el.text())
                .unwrap_or(manga.title);
            
            manga.cover = html.select_first("div.detail_header img")
                .and_then(|el| el.attr("abs:src"));
            
            manga.authors = html.select_first("div.info a[href*='creator']")
                .and_then(|el| el.text())
                .map(|s| vec![s]);
            
            manga.description = html.select_first("p.summary")
                .and_then(|el| el.text());
            
            manga.url = Some(url);
            
            manga.tags = html.select("span.genre")
                .map(|els| els.filter_map(|el| el.text()).collect());
            
            manga.status = if html.select_first("p.day_info:contains(COMPLETED)").is_some() {
                MangaStatus::Completed
            } else {
                MangaStatus::Ongoing
            };
            
            manga.content_rating = ContentRating::Safe;
            manga.viewer = Viewer::Webtoon;
        }
        
        if needs_chapters {
            let chapters = html.select("ul#_episodeList li")
                .map(|els| {
                    els.enumerate().filter_map(|(i, el)| {
                        let link = el.select_first("a")?.attr("abs:href")?;
                        let id = link.split("viewer?").nth(1)?.to_string();
                        
                        let title = el.select_first("span.subj span").and_then(|el| el.text());
                        
                        let chapter_num = (els.len() - i) as f32;
                        
                        let date_str = el.select_first("span.date").and_then(|el| el.text());
                        let date_uploaded = date_str.and_then(|s| parse_relative_date(&s));
                        
                        let is_locked = el.select_first("span.ico_locked").is_some();
                        let is_fastpass = el.select_first("span.ico_fastpass").is_some();
                        
                        let scanlators = if is_fastpass {
                            Some(vec!["🔒 Fast Pass".to_string()])
                        } else if is_locked {
                            Some(vec!["🔒 Locked".to_string()])
                        } else {
                            Some(vec!["Free".to_string()])
                        };
                        
                        Some(Chapter {
                            key: id.clone(),
                            title,
                            chapter_number: Some(chapter_num),
                            date_uploaded,
                            scanlators,
                            url: Some(link),
                            locked: is_locked || is_fastpass,
                            ..Default::default()
                        })
                    }).collect()
                })
                .unwrap_or_default();
            
            manga.chapters = Some(chapters);
        }
        
        Ok(manga)
    }

    fn get_chapter_pages(
        &self,
        _manga: Manga,
        chapter: Chapter,
    ) -> Result<Vec<Page>> {
        let url = format!("{BASE_URL}/en/viewer?{}", chapter.key);
        let html = Request::get(&url)?.html()?;
        
        let pages = html.select("div#_imageList img")
            .map(|els| {
                els.enumerate().filter_map(|(i, el)| {
                    let url = el.attr("data-url").or_else(|| el.attr("abs:src"))?;
                    Some(Page {
                        index: i as i32,
                        content: PageContent::url(url),
                        ..Default::default()
                    })
                }).collect()
            })
            .unwrap_or_default();
        
        Ok(pages)
    }
}

impl Webtoons {
    fn parse_manga_listing(&self, html: aidoku::std::html::Node, _page: i32) -> Result<MangaPageResult> {
        let entries = html.select("ul.card_lst li")
            .map(|els| {
                els.filter_map(|el| {
                    let link = el.select_first("a")?.attr("abs:href")?;
                    let id = link.trim_start_matches("https://www.webtoons.com/")
                        .trim_start_matches("http://www.webtoons.com/")
                        .to_string();
                    
                    let title = el.select_first("p.subj").and_then(|el| el.text())?;
                    let cover = el.select_first("img").and_then(|el| el.attr("abs:src"));
                    let authors = el.select_first("p.author").and_then(|el| el.text()).map(|s| vec![s]);
                    
                    Some(Manga {
                        key: id,
                        title,
                        cover,
                        authors,
                        ..Default::default()
                    })
                }).collect()
            })
            .unwrap_or_default();
        
        let has_next_page = html.select_first("a.pg_next").is_some();
        
        Ok(MangaPageResult {
            entries,
            has_next_page,
        })
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
                let hex_chars = b"0123456789ABCDEF";
                result.push(hex_chars[(byte >> 4) as usize] as char);
                result.push(hex_chars[(byte & 0x0F) as usize] as char);
            }
        }
    }
    result
}

fn parse_relative_date(date_str: &str) -> Option<i64> {
    use aidoku::imports::std::current_date;
    
    let now = current_date();
    
    if date_str.contains("hour") {
        let hours: i64 = date_str.split_whitespace().next()?.parse().ok()?;
        Some(now - (hours * 3600))
    } else if date_str.contains("day") {
        let days: i64 = date_str.split_whitespace().next()?.parse().ok()?;
        Some(now - (days * 86400))
    } else if date_str.contains("week") {
        let weeks: i64 = date_str.split_whitespace().next()?.parse().ok()?;
        Some(now - (weeks * 604800))
    } else {
        parse_date(date_str.to_string(), "MMM dd, yyyy")
    }
}

#[no_mangle]
pub extern "C" fn create_source() -> *mut dyn Source {
    Box::into_raw(Box::new(Webtoons::new()))
}
