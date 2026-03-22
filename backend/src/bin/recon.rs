use scraper::{Html, Selector};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = fs::read_to_string("/tmp/motoscout_rendered.html")?;
    let document = Html::parse_document(&html);
    
    // We try to find article tags
    let article_selector = Selector::parse("article").unwrap();
    let articles: Vec<_> = document.select(&article_selector).collect();
    println!("Found {} <article> tags.", articles.len());

    let h2_selector = Selector::parse("h2").unwrap();
    let h3_selector = Selector::parse("h3").unwrap();
    let span_selector = Selector::parse("span").unwrap();
    let p_selector = Selector::parse("p").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    for (i, article) in articles.iter().take(3).enumerate() {
        println!("--- Listing {} ---", i + 1);
        
        let h2s: Vec<_> = article.select(&h2_selector).map(|e| e.text().collect::<String>()).collect();
        let h3s: Vec<_> = article.select(&h3_selector).map(|e| e.text().collect::<String>()).collect();
        let spans: Vec<_> = article.select(&span_selector).map(|e| e.text().collect::<String>()).collect();
        let ps: Vec<_> = article.select(&p_selector).map(|e| e.text().collect::<String>()).collect();
        let links: Vec<_> = article.select(&a_selector).filter_map(|e| e.value().attr("href")).collect();
        
        println!("H2: {:?}", h2s);
        println!("H3: {:?}", h3s);
        // Spans can be noisy, so we filter out empty ones
        let non_empty_spans: Vec<_> = spans.into_iter().filter(|s| !s.trim().is_empty()).collect();
        println!("Spans: {:?}", non_empty_spans);
        println!("P: {:?}", ps);
        println!("Links: {:?}", links);
    }
    
    Ok(())
}
