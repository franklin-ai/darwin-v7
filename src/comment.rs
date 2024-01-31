use crate::classes::BoundingBox;
use crate::client::V7Methods;
use crate::expect_http_ok;
use crate::item::DatasetItemV2;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
#[allow(unused_imports)]
use fake::{Dummy, Fake};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct CommentBody {
    pub body: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct CommentThread {
    pub bounding_box: BoundingBox,
    pub comments: Vec<CommentBody>,
    pub slot_name: String,
}

#[async_trait]
pub trait CommentMethods<C>
where
    C: V7Methods,
{
    async fn add_comment_thread(
        &self,
        client: &C,
        team_slug: String,
        data: CommentThread,
    ) -> Result<CommentThreadResponse>;
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct CommentLine {
    pub author_id: Option<u32>,
    pub body: Option<String>,
    pub comment_thread_id: Option<String>,
    pub created_by_system: Option<bool>,
    pub id: Option<String>,
    pub inserted_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct CommentThreadResponse {
    pub author_id: Option<u32>,
    pub bounding_box: Option<BoundingBox>,
    pub comment_count: Option<u32>,
    pub dataset_item_id: Option<String>,
    pub first_comment: Option<CommentLine>,
    pub id: Option<String>,
    pub inserted_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_types: Option<String>,
    pub last_comment_at: Option<String>,
    pub resolved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_index: Option<String>,
    pub slot_name: Option<String>,
    pub updated_at: Option<String>,
}

#[async_trait]
impl<C> CommentMethods<C> for DatasetItemV2
where
    C: V7Methods + std::marker::Sync,
{
    async fn add_comment_thread(
        &self,
        client: &C,
        team_slug: String,
        data: CommentThread,
    ) -> Result<CommentThreadResponse> {
        let response = client
            .post(
                &format!(
                    "v2/teams/{}/items/{}/comment_threads",
                    team_slug,
                    self.id.as_ref().context("Dataset has no Id")?
                ),
                &data,
            )
            .await?;
        expect_http_ok!(response, CommentThreadResponse)
    }
}
