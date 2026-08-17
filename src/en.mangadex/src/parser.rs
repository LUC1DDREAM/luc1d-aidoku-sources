extern crate alloc;

use aidoku::{
    error::{Error, Result},
    prelude::*,
    std::{String, Vec, ArrayRef, ObjectRef},
    Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
};
use alloc::string::ToString;

const CDN_URL: &str = "https://uploads.mangadex.org";

pub fn parse_manga_list(json: ObjectRef) -> Result<MangaPageResult> {
    let data = json.get("data").as_array()?;
    let total = json.get("total").as_int().unwrap_or(0) as i32;
    
    let mut manga_list = Vec::new();
    
    for item in data {
        let obj = item.as_object()?;
        if let Ok(manga) = parse_manga_object(obj) {
            manga_list.push(manga);
        }
    }
    
    Ok(MangaPageResult {
        manga: manga_list,
        has_more: total > manga_list.len() as i32,
    })
}

pub fn parse_manga_details(json: ObjectRef) -> Result<Manga> {
    let data = json.get("data").as_object()?;
    parse_manga_object(data)
}

fn parse_manga_object(obj: ObjectRef) -> Result<Manga> {
    let id = obj.get("id").as_string()?.read();
    let attributes = obj.get("attributes").as_object()?;
    
    let title_obj = attributes.get("title").as_object()?;
    let title = title_obj.get("en").as_string()
        .or_else(|_| title_obj.values().next().ok_or(Error::new("")))
        .map(|s| s.read())
        .unwrap_or_default();
    
    let description_obj = attributes.get("description").as_object().ok();
    let description = description_obj
        .and_then(|d| d.get("en").as_string().ok())
        .or_else(|| description_obj.and_then(|d| d.values().next()))
        .map(|s| s.read())
        .unwrap_or_default();
    
    let mut cover_url = String::new();
    if let Ok(relationships) = obj.get("relationships").as_array() {
        for rel in relationships {
            let rel_obj = rel.as_object()?;
            let rel_type = rel_obj.get("type").as_string()?.read();
            
            if rel_type == "cover_art" {
                if let Ok(attr) = rel_obj.get("attributes").as_object() {
                    if let Ok(filename) = attr.get("fileName").as_string() {
                        cover_url = format!("{}/covers/{}/{}", CDN_URL, id, filename.read());
                    }
                }
            }
        }
    }
    
    let status = match attributes.get("status").as_string().map(|s| s.read()).as_deref() {
        Ok("ongoing") => MangaStatus::Ongoing,
        Ok("completed") => MangaStatus::Completed,
        Ok("hiatus") => MangaStatus::Hiatus,
        Ok("cancelled") => MangaStatus::Cancelled,
        _ => MangaStatus::Unknown,
    };
    
    let nsfw = match attributes.get("contentRating").as_string().map(|s| s.read()).as_deref() {
        Ok("safe") | Ok("suggestive") => MangaContentRating::Safe,
        Ok("erotica") => MangaContentRating::Nsfw,
        Ok("pornographic") => MangaContentRating::Nsfw,
        _ => MangaContentRating::Safe,
    };
    
    let categories = attributes.get("tags").as_array()
        .map(|tags| {
            tags.filter_map(|tag| {
                let tag_obj = tag.as_object().ok()?;
                let attr = tag_obj.get("attributes").as_object().ok()?;
                let name = attr.get("name").as_object().ok()?;
                name.get("en").as_string().ok().map(|s| s.read())
            })
            .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    
    let author = attributes.get("author").as_string()
        .or_else(|_| attributes.get("artist").as_string())
        .map(|s| s.read())
        .unwrap_or_default();
    
    Ok(Manga {
        id,
        cover: cover_url,
        title,
        author,
        artist: String::new(),
        description,
        url: format!("https://mangadex.org/title/{}", id),
        categories,
        status,
        nsfw,
        viewer: MangaViewer::Rtl,
    })
}

pub fn parse_chapter_list(json: ObjectRef) -> Result<Vec<Chapter>> {
    let data = json.get("data").as_array()?;
    let mut chapters = Vec::new();
    
    for item in data {
        let obj = item.as_object()?;
        let id = obj.get("id").as_string()?.read();
        let attributes = obj.get("attributes").as_object()?;
        
        let chapter_num = attributes.get("chapter").as_string()
            .ok()
            .and_then(|s| s.read().parse::<f32>().ok())
            .unwrap_or(-1.0);
        
        let title = attributes.get("title").as_string()
            .map(|s| s.read())
            .unwrap_or_default();
        
        let volume = attributes.get("volume").as_string()
            .ok()
            .and_then(|s| s.read().parse::<f32>().ok())
            .unwrap_or(-1.0);
        
        let date_updated = attributes.get("publishAt").as_date("yyyy-MM-dd'T'HH:mm:ss", Some("UTC"), None)
            .unwrap_or(0.0);
        
        // Enhanced: Get scanlation group info
        let mut scanlator = String::new();
        if let Ok(relationships) = obj.get("relationships").as_array() {
            for rel in relationships {
                if let Ok(rel_obj) = rel.as_object() {
                    if let Ok(rel_type) = rel_obj.get("type").as_string() {
                        if rel_type.read() == "scanlation_group" {
                            if let Ok(attr) = rel_obj.get("attributes").as_object() {
                                if let Ok(name) = attr.get("name").as_string() {
                                    scanlator = name.read();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        let lang = attributes.get("translatedLanguage").as_string()
            .map(|s| s.read())
            .unwrap_or_else(|_| String::from("en"));
        
        chapters.push(Chapter {
            id,
            title,
            volume,
            chapter: chapter_num,
            date_updated,
            scanlator,
            url: String::new(),
            lang,
        });
    }
    
    Ok(chapters)
}

pub fn parse_page_list(json: ObjectRef) -> Result<Vec<Page>> {
    let base_url = json.get("baseUrl").as_string()?.read();
    let chapter = json.get("chapter").as_object()?;
    let hash = chapter.get("hash").as_string()?.read();
    
    // Use data-saver for better performance, or "data" for high quality
    let images = chapter.get("data").as_array()
        .or_else(|_| chapter.get("dataSaver").as_array())?;
    
    let mut pages = Vec::new();
    
    for (index, img) in images.enumerate() {
        let filename = img.as_string()?.read();
        let url = format!("{}/data/{}/{}", base_url, hash, filename);
        
        pages.push(Page {
            index: index as i32,
            url,
            base64: String::new(),
            text: String::new(),
        });
    }
    
    Ok(pages)
}
