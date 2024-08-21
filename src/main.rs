use reqwest::blocking::get;
use reqwest::Url;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <URL>", args[0]);
        return;
    }

    let start_url = &args[1];
    let mut visited_urls = HashSet::new();
    let mut image_urls = HashSet::new();

    scrape_page(start_url, &mut visited_urls, &mut image_urls);

    for image_url in image_urls {
        println!("{}", image_url);
    }
}

fn scrape_page(url: &str, visited_urls: &mut HashSet<String>, image_urls: &mut HashSet<String>) {
    if visited_urls.contains(url) {
        return;
    }

    visited_urls.insert(url.to_string());

    let response = match get(url) {
        Ok(response) => response,
        Err(_) => {
            eprintln!("Failed to fetch URL: {}", url);
            return;
        },
    };

    let html_content = match response.text() {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to read response text for URL: {}", url);
            return;
        },
    };

    let document = Html::parse_document(&html_content);
    let img_selector = Selector::parse("img").unwrap();
    let css_selector = Selector::parse("[style]").unwrap();

    for img in document.select(&img_selector) {
        if let Some(src) = img.value().attr("src") {
            let full_url = resolve_url(&url, src);
            image_urls.insert(full_url);
        }
    }

    for element in document.select(&css_selector) {
        if let Some(style) = element.value().attr("style") {
            for url in extract_background_image_urls(style) {
                let full_url = resolve_url(&url, &url);
                image_urls.insert(full_url);
            }
        }
    }
}

fn resolve_url(base: &str, relative: &str) -> String {
    let base_url = match Url::parse(base) {
        Ok(url) => url,
        Err(_) => return relative.to_string(),
    };
    let resolved_url = match base_url.join(relative) {
        Ok(url) => url,
        Err(_) => return relative.to_string(),
    };
    resolved_url.to_string()
}

fn extract_background_image_urls(style: &str) -> Vec<String> {
    use regex::Regex;

    let re = Regex::new(r#"background(?:-image)?\s*:\s*url\(["']?(.*?)["']?\)"#).unwrap();
    re.captures_iter(style)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}
