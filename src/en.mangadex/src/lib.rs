#![no_std]

use aidoku::{
    prelude::*,
    Chapter, ContentRating, Filter, FilterValue, Listing, Manga, MangaPageResult, MangaStatus, 
    Page, PageContent, Result, Source, Viewer,
    alloc::{String, Vec, string::ToString, format},
    error::AidokuError,
    helpers::uri::QueryParameters,
    imports::{
        json::parse,
        net::Request,
    },
};

mod helper;

const BASE_URL: &str = "https://api.mangadex.org";
const CDN_URL: &str = "https://uploads.mangadex.org";

struct MangaDex;

impl Source for MangaDex {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let offset = (page - 1) * 20;
        let mut url = format!("{BASE_URL}/manga?limit=20&offset={offset}");
        
        if let Some(q) = query {
            url.push_str("&title=");
            url.push_str(&helper::urlencode(&q));
        }
        
        url.push_str(&helper::get_filter_string(filters));
        url.push_str("&includes[]=cover_art&includes[]=author&includes[]=artist");
        url.push_str("&contentRating[]=safe&contentRating[]=suggestive&contentRating[]=erotica");
        url.push_str("&order[relevance]=desc");
        
        let json = Request::get(&url)?.json()?;
        self.parse_manga_list(json)
    }

    fn get_listing_manga_list(
        &self,
        listing: Listing,
        page: i32,
    ) -> Result<MangaPageResult> {
        let offset = (page - 1) * 20;
        let mut url = format!("{BASE_URL}/manga?limit=20&offset={offset}");
        url.push_str("&includes[]=cover_art&includes[]=author&includes[]=artist");
        url.push_str("&contentRating[]=safe&contentRating[]=suggestive&contentRating[]=erotica");
        
        match listing.name.as_str() {
            "Latest Updates" => url.push_str("&order[latestUploadedChapter]=desc"),
            "Recently Added" => url.push_str("&order[createdAt]=desc"),
            "Top Rated" => url.push_str("&order[rating]=desc"),
            "Most Follows" => url.push_str("&order[followedCount]=desc"),
            _ => url.push_str("&order[latestUploadedChapter]=desc"),
        }
        
        let json = Request::get(&url)?.json()?;
        self.parse_manga_list(json)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        if needs_details {
            let url = format!("{BASE_URL}/manga/{}?includes[]=cover_art&includes[]=author&includes[]=artist", manga.key);
            let json = Request::get(&url)?.json()?;
            manga = self.parse_manga_details(json, manga.key.clone())?;
        }
        
        if needs_chapters {
            manga.chapters = Some(self.get_chapter_list_internal(&manga.key)?);
        }
        
        Ok(manga)
    }

    fn get_chapter_pages(
        &self,
        _manga: Manga,
        chapter: Chapter,
    ) -> Result<Vec<Page>> {
        let url = format!("{BASE_URL}/at-home/server/{}", chapter.key);
        let json = Request::get(&url)?.json()?;
        
        let base_url = json.get("baseUrl").and_then(|v| v.as_string()).ok_or(AidokuError::ParseError)?;
        let chapter_obj = json.get("chapter").and_then(|v| v.as_object()).ok_or(AidokuError::ParseError)?;
        let hash = chapter_obj.get("hash").and_then(|v| v.as_string()).ok_or(AidokuError::ParseError)?;
        let data = chapter_obj.get("data").and_then(|v| v.as_array()).ok_or(AidokuError::ParseError)?;
        
        let pages = data.enumerate()
            .filter_map(|(i, filename)| {
                let filename = filename.as_string()?;
                let url = format!("{base_url}/data/{hash}/{filename}");
                Some(Page {
                    index: i as i32,
                    content: PageContent::url(url),
                    ..Default::default()
                })
            })
            .collect();
        
        Ok(pages)
    }
}

impl MangaDex {
    fn parse_manga_list(&self, json: aidoku::std::json::Json) -> Result<MangaPageResult> {
        let data = json.get("data").and_then(|v| v.as_array()).ok_or(AidokuError::ParseError)?;
        let total = json.get("total").and_then(|v| v.as_int()).unwrap_or(0);
        
        let entries = data.filter_map(|item| {
            let obj = item.as_object()?;
            let id = obj.get("id").and_then(|v| v.as_string())?;
            let attributes = obj.get("attributes").and_then(|v| v.as_object())?;
            
            let title = attributes.get("title")
                .and_then(|v| v.as_object())
                .and_then(|t| t.get("en").and_then(|v| v.as_string()))
                .or_else(|| {
                    attributes.get("title")
                        .and_then(|v| v.as_object())
                        .and_then(|t| t.values().next())
                        .and_then(|v| v.as_string())
                })?;
            
            let mut cover = String::new();
            let mut authors = Vec::new();
            
            if let Some(rels) = obj.get("relationships").and_then(|v| v.as_array()) {
                for rel in rels {
                    let rel_obj = rel.as_object()?;
                    let rel_type = rel_obj.get("type").and_then(|v| v.as_string())?;
                    
                    match rel_type.as_str() {
                        "cover_art" => {
                            if let Some(attr) = rel_obj.get("attributes").and_then(|v| v.as_object()) {
                                if let Some(filename) = attr.get("fileName").and_then(|v| v.as_string()) {
                                    cover = format!("{CDN_URL}/covers/{id}/{filename}");
                                }
                            }
                        }
                        "author" | "artist" => {
                            if let Some(attr) = rel_obj.get("attributes").and_then(|v| v.as_object()) {
                                if let Some(name) = attr.get("name").and_then(|v| v.as_string()) {
                                    if !authors.contains(&name) {
                                        authors.push(name);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            
            Some(Manga {
                key: id,
                title,
                cover: Some(cover),
                authors: Some(authors),
                ..Default::default()
            })
        }).collect();
        
        Ok(MangaPageResult {
            entries,
            has_next_page: total > entries.len() as i64,
        })
    }
    
    fn parse_manga_details(&self, json: aidoku::std::json::Json, id: String) -> Result<Manga> {
        let data = json.get("data").and_then(|v| v.as_object()).ok_or(AidokuError::ParseError)?;
        let attributes = data.get("attributes").and_then(|v| v.as_object()).ok_or(AidokuError::ParseError)?;
        
        let title = attributes.get("title")
            .and_then(|v| v.as_object())
            .and_then(|t| t.get("en").and_then(|v| v.as_string()))
            .unwrap_or_default();
        
        let description = attributes.get("description")
            .and_then(|v| v.as_object())
            .and_then(|d| d.get("en").and_then(|v| v.as_string()));
        
        let status = match attributes.get("status").and_then(|v| v.as_string()).map(|s| s.as_str()) {
            Some("ongoing") => MangaStatus::Ongoing,
            Some("completed") => MangaStatus::Completed,
            Some("hiatus") => MangaStatus::Hiatus,
            Some("cancelled") => MangaStatus::Cancelled,
            _ => MangaStatus::Unknown,
        };
        
        let mut cover = String::new();
        let mut authors = Vec::new();
        let mut tags = Vec::new();
        
        if let Some(rels) = data.get("relationships").and_then(|v| v.as_array()) {
            for rel in rels {
                let rel_obj = rel.as_object()?;
                let rel_type = rel_obj.get("type").and_then(|v| v.as_string())?;
                
                match rel_type.as_str() {
                    "cover_art" => {
                        if let Some(attr) = rel_obj.get("attributes").and_then(|v| v.as_object()) {
                            if let Some(filename) = attr.get("fileName").and_then(|v| v.as_string()) {
                                cover = format!("{CDN_URL}/covers/{id}/{filename}");
                            }
                        }
                    }
                    "author" | "artist" => {
                        if let Some(attr) = rel_obj.get("attributes").and_then(|v| v.as_object()) {
                            if let Some(name) = attr.get("name").and_then(|v| v.as_string()) {
                                if !authors.contains(&name) {
                                    authors.push(name);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        if let Some(tag_arr) = attributes.get("tags").and_then(|v| v.as_array()) {
            for tag in tag_arr {
                if let Some(tag_obj) = tag.as_object() {
                    if let Some(attrs) = tag_obj.get("attributes").and_then(|v| v.as_object()) {
                        if let Some(name_obj) = attrs.get("name").and_then(|v| v.as_object()) {
                            if let Some(en_name) = name_obj.get("en").and_then(|v| v.as_string()) {
                                tags.push(en_name);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(Manga {
            key: id.clone(),
            title,
            cover: Some(cover),
            authors: Some(authors),
            description,
            url: Some(format!("https://mangadex.org/title/{id}")),
            tags: Some(tags),
            status,
            content_rating: ContentRating::Safe,
            viewer: Viewer::RightToLeft,
            ..Default::default()
        })
    }
    
    fn get_chapter_list_internal(&self, manga_id: &str) -> Result<Vec<Chapter>> {
        let mut chapters = Vec::new();
        let mut offset = 0;
        let limit = 500;
        
        loop {
            let url = format!(
                "{BASE_URL}/manga/{manga_id}/feed?limit={limit}&offset={offset}&translatedLanguage[]=en&includes[]=scanlation_group&order[chapter]=desc&contentRating[]=safe&contentRating[]=suggestive&contentRating[]=erotica&contentRating[]=pornographic"
            );
            
            let json = Request::get(&url)?.json()?;
            let data = json.get("data").and_then(|v| v.as_array()).ok_or(AidokuError::ParseError)?;
            
            if data.len() == 0 {
                break;
            }
            
            for item in data {
                let obj = item.as_object()?;
                let id = obj.get("id").and_then(|v| v.as_string())?;
                let attributes = obj.get("attributes").and_then(|v| v.as_object())?;
                
                let chapter_num = attributes.get("chapter")
                    .and_then(|v| v.as_string())
                    .and_then(|s| s.parse::<f32>().ok());
                
                let volume = attributes.get("volume")
                    .and_then(|v| v.as_string())
                    .and_then(|s| s.parse::<f32>().ok());
                
                let title = attributes.get("title").and_then(|v| v.as_string());
                
                let date_uploaded = attributes.get("publishAt")
                    .and_then(|v| v.as_string())
                    .and_then(|s| {
                        let cleaned = s.replace('+', "Z");
                        aidoku::imports::std::parse_date(cleaned, "yyyy-MM-dd'T'HH:mm:ss'Z'")
                    });
                
                let mut scanlators = Vec::new();
                if let Some(rels) = obj.get("relationships").and_then(|v| v.as_array()) {
                    for rel in rels {
                        if let Some(rel_obj) = rel.as_object() {
                            if let Some("scanlation_group") = rel_obj.get("type").and_then(|v| v.as_string()).map(|s| s.as_str()) {
                                if let Some(attr) = rel_obj.get("attributes").and_then(|v| v.as_object()) {
                                    if let Some(name) = attr.get("name").and_then(|v| v.as_string()) {
                                        scanlators.push(name);
                                    }
                                }
                            }
                        }
                    }
                }
                
                chapters.push(Chapter {
                    key: id,
                    title,
                    chapter_number: chapter_num,
                    volume_number: volume,
                    date_uploaded,
                    scanlators: if scanlators.is_empty() { None } else { Some(scanlators) },
                    url: Some(format!("https://mangadex.org/chapter/{}", obj.get("id").and_then(|v| v.as_string())?)),
                    ..Default::default()
                });
            }
            
            offset += limit;
        }
        
        Ok(chapters)
    }
}

#[no_mangle]
pub extern "C" fn create_source() -> *mut dyn Source {
    Box::into_raw(Box::new(MangaDex::new()))
}
