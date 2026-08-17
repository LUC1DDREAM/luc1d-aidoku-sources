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

const BASE_URL: &str = "https://www.webtoons.com";
const API_BASE: &str = "https://www.webtoons.com/en";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    helper::search_manga(filters, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    let mut url = String::from(API_BASE);
    
    match listing.name.as_str() {
        "Canvas" => url.push_str("/canvas/genre"),
        "Originals" => url.push_str("/originals/genre"),
        "Top" => url.push_str("/top"),
        "Rising" => url.push_str("/rising"),
        _ => url.push_str("/daily"),
    }
    
    let html = Request::new(&url, aidoku::std::net::HttpMethod::Get).html()?;
    parser::parse_manga_listing(html, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
    let mut url = String::from(BASE_URL);
    url.push('/');
    url.push_str(&id);
    
    let html = Request::new(&url, aidoku::std::net::HttpMethod::Get).html()?;
    parser::parse_manga_details(html, id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
    let mut url = String::from(BASE_URL);
    url.push('/');
    url.push_str(&id);
    
    let html = Request::new(&url, aidoku::std::net::HttpMethod::Get).html()?;
    parser::parse_chapter_list(html)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
    let mut url = String::from(BASE_URL);
    url.push('/');
    url.push_str(&chapter_id);
    
    let html = Request::new(&url, aidoku::std::net::HttpMethod::Get).html()?;
    parser::parse_page_list(html)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
    helper::parse_incoming_url(url)
}
