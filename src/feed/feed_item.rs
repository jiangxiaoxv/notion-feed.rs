use crate::notion::models::{Page, PropertyValue};

#[derive(Debug)]
pub struct FeedItem {
    pub link: String,
}

impl FeedItem {
    pub fn new(page: &Page) -> Option<FeedItem> {
        let properties = page.properties.as_ref()?;

        let link = match properties.get("Link") {
            Some(PropertyValue::Url { url }) => url.as_ref().map(|url| url.to_string()),
            _ => None,
        };

        if let Some(link) = link {
            return Some(Self { link });
        }

        None
    }
}
