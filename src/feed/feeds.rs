use chrono::{DateTime, Utc};
use feed_rs::{model::Entry, parser};
use futures::{future, try_join};

use crate::notion::{
    database::{DatabaseFilter, DatabaseKind, DatabaseQuery, Filter, FilterKind},
    models::{Date, Page, PropertyValue, RichText, Text},
    Client,
};
use std::{collections::HashMap, error::Error};

use super::{feed_item::FeedItem, source::Source};

pub struct Feed<'a> {
    client: &'a Client<'a>,
}

impl<'a> Feed<'a> {
    pub fn new(client: &'a Client<'a>) -> Feed<'a> {
        Self { client }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let (source_list, feed_list) = try_join!(self.get_source_list(), self.get_feed_list())?;

        let feed_items =
            future::join_all(source_list.into_iter().map(Feed::<'_>::get_feed_items)).await;

        let mut all_feed_items = vec![];

        feed_items.iter().for_each(|item| {
            if let Ok(item) = item {
                all_feed_items.extend(item);
            }
        });

        let feed_list_links = feed_list
            .into_iter()
            .map(|item| item.link)
            .collect::<Vec<String>>();

        future::join_all(
            all_feed_items
                .iter()
                .filter(|item| match &item.links.first() {
                    Some(link) => !feed_list_links.contains(&link.href),
                    None => false,
                })
                .filter_map(|item| {
                    let href = &item.links.first().unwrap().href;
                    let title = item.title.as_ref().map(|t| t.content.as_str());
                    let pub_date = item.published.as_ref();
                    let updated_date = item.updated.as_ref();

                    let created_date = if let Some(pub_date) = pub_date {
                        Some(*pub_date)
                    } else if let Some(updated_date) = updated_date {
                        Some(*updated_date)
                    } else {
                        Some(Utc::now())
                    };

                    if let (Some(title), Some(created_date)) = (title, created_date) {
                        return Some(self.add_feed_entry(
                            title.to_string(),
                            href.to_string(),
                            created_date,
                        ));
                    } else {
                        None
                    }
                }),
        )
        .await;

        Ok(())
    }

    pub async fn get_source_list(&self) -> Result<Vec<Source>, Box<dyn Error>> {
        let filter = DatabaseFilter::Compound {
            filter: HashMap::from([(
                "or".to_string(),
                vec![Filter {
                    property: "Enabled".to_string(),
                    kind: FilterKind::Checkbox { equals: true },
                }],
            )]),
        };

        let query = DatabaseQuery {
            start_cursor: None,
            page_size: None,
            sorts: None,
            filter: Some(filter),
        };

        let pages = self
            .client
            .query_database(DatabaseKind::Source, Some(query))
            .await?
            .results;

        return Ok(pages
            .iter()
            .filter_map(Source::new)
            .collect::<Vec<Source>>());
    }

    pub async fn get_feed_list(&self) -> Result<Vec<FeedItem>, Box<dyn Error>> {
        let mut pages = vec![];
        let mut cursor: Option<String> = None;

        loop {
            let query = DatabaseQuery {
                start_cursor: cursor.as_ref().map(|c| c.to_string()),
                page_size: None, // use default value (100)
                filter: None,
                sorts: None,
            };

            let current_pages = self
                .client
                .query_database(DatabaseKind::Feed, Some(query))
                .await?;

            pages.extend(current_pages.results);
            cursor = current_pages.next_cursor;

            if !current_pages.has_more {
                break;
            }
        }

        return Ok(pages
            .iter()
            .filter_map(FeedItem::new)
            .collect::<Vec<FeedItem>>());
    }

    pub async fn add_feed_entry(
        &self,
        title: String,
        link: String,
        created_time: DateTime<Utc>,
    ) -> Result<Page, Box<dyn Error>> {
        let page_props = HashMap::from([
            (
                "Title".to_string(),
                PropertyValue::Title {
                    title: vec![RichText::Text {
                        rich_text: None,
                        text: Text {
                            content: title,
                            link: None,
                        },
                    }],
                },
            ),
            ("Link".to_string(), PropertyValue::Url { url: Some(link) }),
            (
                "Read".to_string(),
                PropertyValue::Checkbox { checkbox: false },
            ),
            (
                "Starred".to_string(),
                PropertyValue::Checkbox { checkbox: false },
            ),
            (
                "Published At".to_string(),
                PropertyValue::Date {
                    date: Some(Date {
                        start: Some(created_time),
                        end: None,
                    }),
                },
            ),
        ]);

        let result = self
            .client
            .create_page(DatabaseKind::Feed, page_props)
            .await;

        result
    }

    pub async fn get_feed_items(source: Source) -> Result<Vec<Entry>, Box<dyn Error>> {
        let content = reqwest::get(&source.link).await?.bytes().await?;
        let feed = parser::parse(&content[..])?;

        let offset_date = source.offset_date;

        if let Some(offset_date) = offset_date {
            let items = feed
                .entries
                .into_iter()
                .filter(|item| {
                    let pub_date = item.published.as_ref();
                    let updated_date = item.updated.as_ref();

                    if let Some(pub_date) = pub_date {
                        let pub_date = pub_date.date_naive();

                        return pub_date.ge(&offset_date);
                    } else if let Some(updated_date) = updated_date {
                        let updated_date = updated_date.date_naive();

                        return updated_date.ge(&offset_date);
                    }

                    true
                })
                .collect();

            return Ok(items);
        }

        Ok(feed.entries)
    }
}
