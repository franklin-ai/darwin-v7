use crate::classes::BoundingBox;
use crate::client::V7Methods;
use crate::expect_http_ok;
use crate::item::DatasetItemV2;
use anyhow::{bail, Result};
use async_trait::async_trait;
#[allow(unused_imports)]
use fake::{Dummy, Fake};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct CommentBody {
    pub body: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct CommentLine {
    pub author_id: u32,
    pub body: String,
    pub comment_thread_id: String,
    pub created_by_system: bool,
    pub id: String,
    pub inserted_at: String,
    pub updated_at: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct CommentThreadResponse {
    pub author_id: f32,
    pub bounding_box: BoundingBox,
    pub comment_count: f32,
    pub dataset_item_id: String,
    pub first_comment: CommentLine,
    pub id: String,
    pub inserted_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_types: Option<String>,
    pub last_comment_at: String,
    pub resolved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_index: Option<String>,
    pub slot_name: String,
    pub updated_at: String,
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
                &format!("v2/teams/{}/items/{}/comment_threads", team_slug, self.id),
                &data,
            )
            .await?;
        expect_http_ok!(response, CommentThreadResponse)
    }
}
