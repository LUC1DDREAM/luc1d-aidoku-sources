#![no_std]
extern crate alloc;

use aidoku::{
    error::Result,
    prelude::*,
    std::{net::Request, String, Vec},
    Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
};

mod parser;
mod helper;

const BASE_URL: &str = "https://api.mangadex.org";
const CDN_URL: &str = "https://uploads.mangadex.org";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    let mut url = String::from(BASE_URL);
    url.push_str("/manga?limit=20&offset=");
    url.push_str(&helper::i32_to_string((page - 1) * 20));
    url.push_str(&helper::get_filter_string(filters));
    url.push_str("&includes[]=cover_art&includes[]=author&includes[]=artist");
    url.push_str("&contentRating[]=safe&contentRating[]=suggestive&contentRating[]=erotica");
    url.push_str("&order[relevance]=desc");
    
    let json = Request::new(url, aidoku::std::net::HttpMethod::Get).json()?;
    parser::parse_manga_list(json)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    let mut url = String::from(BASE_URL);
    url.push_str("/manga?limit=20&offset=");
    url.push_str(&helper::i32_to_string((page - 1) * 20));
    url.push_str("&includes[]=cover_art&includes[]=author&includes[]=artist");
    url.push_str("&contentRating[]=safe&contentRating[]=suggestive&contentRating[]=erotica");
    
    match listing.name.as_str() {
        "Latest Updates" => {
            url.push_str("&order[latestUploadedChapter]=desc");
        }
        "Recently Added" => {
            url.push_str("&order[createdAt]=desc");
        }
        "Top Rated" => {
            url.push_str("&order[rating]=desc");
        }
        "Most Follows" => {
            url.push_str("&order[followedCount]=desc");
        }
        _ => {
            url.push_str("&order[latestUploadedChapter]=desc");
        }
    }
    
    let json = Request::new(url, aidoku::std::net::HttpMethod::Get).json()?;
    parser::parse_manga_list(json)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
    let mut url = String::from(BASE_URL);
    url.push_str("/manga/");
    url.push_str(&id);
    url.push_str("?includes[]=cover_art&includes[]=author&includes[]=artist");
    
    let json = Request::new(url, aidoku::std::net::HttpMethod::Get).json()?;
    parser::parse_manga_details(json)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
    let mut chapters = Vec::new();
    let mut offset = 0;
    let limit = 500;
    
    loop {
        let mut url = String::from(BASE_URL);
        url.push_str("/manga/");
        url.push_str(&id);
        url.push_str("/feed?limit=");
        url.push_str(&helper::i32_to_string(limit));
        url.push_str("&offset=");
        url.push_str(&helper::i32_to_string(offset));
        url.push_str("&translatedLanguage[]=en");
        url.push_str("&includes[]=scanlation_group");
        url.push_str("&order[chapter]=desc");
        url.push_str("&contentRating[]=safe&contentRating[]=suggestive&contentRating[]=erotica&contentRating[]=pornographic");
        
        let json = Request::new(url, aidoku::std::net::HttpMethod::Get).json()?;
        let batch = parser::parse_chapter_list(json)?;
        
        if batch.is_empty() {
            break;
        }
        
        chapters.extend(batch);
        offset += limit;
    }
    
    Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
    let mut url = String::from(BASE_URL);
    url.push_str("/at-home/server/");
    url.push_str(&chapter_id);
    
    let json = Request::new(url, aidoku::std::net::HttpMethod::Get).json()?;
    parser::parse_page_list(json)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
    helper::parse_incoming_url(url)
}
