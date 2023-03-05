use super::{
    database::DatabaseKind,
    models::{Page, Parent, PropertyValue},
    Client,
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

#[derive(Debug, Serialize, Deserialize)]
struct CreatePageProps {
    parent: Parent,
    properties: HashMap<String, PropertyValue>,
}

impl<'a> Client<'a> {
    pub async fn create_page(
        &self,
        kind: DatabaseKind,
        properties: HashMap<String, PropertyValue>,
    ) -> Result<Page, Box<dyn Error>> {
        let db_id = match kind {
            DatabaseKind::Source => &self.config.notion_source_database_id,
            DatabaseKind::Feed => &self.config.notion_feed_database_id,
        };

        let path = "/pages";

        let create_page_props = CreatePageProps {
            parent: Parent {
                parent_type: "database_id".to_string(),
                database_id: Some(db_id.to_string()),
            },
            properties,
        };

        let res = self
            .build_request(Method::POST, path)
            .json(&create_page_props)
            .send()
            .await?
            .error_for_status()?;

        Ok(res.json::<Page>().await?)
    }
}
