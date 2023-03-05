use chrono::{Months, NaiveDate, Utc};
use regex::Regex;

use crate::notion::models::{Page, PropertyValue, RichText};

#[derive(Debug, Clone)]
pub struct Source {
    pub link: String,
    pub offset_date: Option<NaiveDate>,
}

impl Source {
    pub fn new(page: &Page) -> Option<Source> {
        let properties = page.properties.as_ref()?;

        let link = match properties.get("Link") {
            Some(PropertyValue::Url { url }) => url.as_ref().map(|url| url.to_string()),
            _ => None,
        };

        let offset_date = match properties.get("Offset date") {
            Some(PropertyValue::Text { rich_text }) => rich_text.first().map(|RichText::Text { rich_text: _, text }| &text.content),
            _ => None,
        };

        let offset_date = match offset_date {
            Some(offset_date) => extract_offset(offset_date).unwrap_or(0),
            None => 0,
        };

        let offset_date = match offset_date {
            offset if offset > 0 => Utc::now()
                .date_naive()
                .and_hms(0, 0, 0)
                .date()
                .checked_sub_months(Months::new(offset)),
            _ => None,
        };

        if let Some(link) = link {
            return Some(Self { link, offset_date });
        }

        None
    }
}

fn extract_offset(input: &str) -> Option<u32> {
    let months_re = Regex::new(r#"((^1 month$)|(^[2-9]|[1-9]\d+ months$))"#).unwrap();
    let nr_re = Regex::new(r#"\d+"#).unwrap();

    if !months_re.is_match(input) {
        return None;
    }

    if let Some(offset) = nr_re.find(input) {
        return Some(
            offset
                .as_str()
                .to_string()
                .trim()
                .parse::<u32>()
                .unwrap_or(0),
        );
    }

    None
}

#[cfg(test)]
mod tests {
    use super::extract_offset;

    #[test]
    fn test_extract_offset() {
        assert_eq!(1, extract_offset("1 month").unwrap());
        assert!(extract_offset("0 month").is_none());
        assert!(extract_offset("1 months").is_none());
        assert!(extract_offset("-1 months").is_none());
        assert!(extract_offset("12 month").is_none());
        assert_eq!(12, extract_offset("12 months").unwrap());
    }
}
