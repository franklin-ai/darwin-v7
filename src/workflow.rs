use crate::classes::BoundingBox;
use crate::client::V7Methods;
use crate::expect_http_ok;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
#[allow(unused_imports)]
use fake::{Dummy, Fake};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{self, Display};

#[derive(Default, Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StageType {
    #[default]
    Annotate,
    Complete,
    Consensus,
    Model,
    New,
    Review,
    Dataset,
}

impl Display for StageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            StageType::Annotate => "Annotate",
            StageType::Complete => "Complete",
            StageType::Consensus => "Consensus",
            StageType::New => "New",
            StageType::Model => "Model",
            StageType::Review => "Review",
            StageType::Dataset => "Dataset",
        };
        write!(f, "{val}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
pub struct TemplateMetadata {
    pub assignable_to: Option<String>,
    pub base_sampling_rate: Option<f64>,
    pub parallel: Option<u32>,
    pub user_sampling_rate: Option<f64>,
    pub readonly: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct MetaData {
    pub ready_for_completion: Option<bool>,
    pub previous_stage_number: Option<u32>,
    pub review_status: Option<String>,
    pub review_status_modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
pub struct Stage {
    pub assignee_id: Option<u32>,
    pub completed: Option<bool>,
    pub completes_at: Option<String>,
    pub dataset_item_id: u32,
    pub id: u32,
    pub metadata: Option<MetaData>,
    pub number: Option<u32>,
    pub skipped: Option<bool>,
    pub skipped_reason: Option<String>,
    pub template_metadata: Option<TemplateMetadata>,
    #[serde(rename = "type")]
    pub stage_type: Option<StageType>,
    pub workflow_id: u32,
    pub workflow_stage_template_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
pub struct TemplateAssignee {
    pub assignee_id: u32,
    pub sampling_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
pub struct WorkflowStageTemplate {
    pub id: Option<u32>,
    pub metadata: TemplateMetadata,
    pub name: Option<String>,
    pub workflow_stage_template_assignees: Vec<TemplateAssignee>,
    pub stage_number: Option<usize>,
    #[serde(rename = "type")]
    pub stage_type: StageType,
    pub workflow_template_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
pub struct Workflow {
    pub current_stage_number: u32,
    pub current_workflow_stage_template_id: u32,
    pub dataset_item_id: u32,
    pub id: u32,
    pub stages: HashMap<u32, Vec<Stage>>,
    pub status: String, // This can probably be an enum
    pub workflow_template_id: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy)]
pub struct WorkflowTemplate {
    pub dataset_id: u32,
    pub id: Option<u32>,
    pub name: Option<String>,
    pub workflow_stage_templates: Vec<WorkflowStageTemplate>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowBody {
    pub body: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct LocatedWorkflowComments {
    pub bounding_box: BoundingBox,
    pub workflow_comments: Vec<WorkflowBody>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowCommentThread {
    pub author_id: u32,
    pub bounding_box: BoundingBox,
    pub comment_count: u32,
    pub frame_index: Option<u32>,
    pub id: u32,
    pub inserted_at: String,
    pub resolved: bool,
    pub updated_at: String,
    pub workflow_id: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
struct UserId {
    pub user_id: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct FilterAssignItemPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<StageType>>,
    pub dataset_ids: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_ids: Option<Vec<String>>,
    pub select_all: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct AssignItemPayload {
    pub filters: FilterAssignItemPayload,
    pub assignee_email: String,
    pub workflow_id: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct AssignItemResponse {
    pub created_commands: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct AnnotationHotkeys {}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowDataset {
    pub annotation_hotkeys: AnnotationHotkeys,
    pub annotators_can_instantiate_workflows: bool,
    pub id: u64,
    pub instructions: String,
    pub name: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowProgress {
    pub complete: u32,
    pub idle: u32,
    pub in_progress: u32,
    pub total: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct StageConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_class_ids: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignable_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_instantiate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub champion_stage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_mapping: Option<Vec<String>>,
    pub dataset_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_non_default_v1_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_annotations: Option<bool>,
    pub initial: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iou_thresholds: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_stage_ids: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readonly: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_if_fails: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skippable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_stage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct StageEdge {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String, //FIXME: What are the different types?
    pub source_stage_id: String,
    pub target_stage_id: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowStageAssignees {
    pub stage_id: String,
    pub user_id: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowStageV2 {
    pub assignable_users: Vec<WorkflowStageAssignees>,
    pub config: StageConfig,
    pub edges: Vec<StageEdge>,
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub stage_type: StageType,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowV2 {
    pub dataset: WorkflowDataset,
    pub id: String,
    pub inserted_at: String,
    pub name: String,
    pub progress: WorkflowProgress,
    pub stages: Vec<WorkflowStageV2>,
    pub team_id: String,
    pub thumbnails: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowBuilder {
    pub stages: Vec<WorkflowStageV2>,
    pub name: Option<String>,
}

#[async_trait]
pub trait WorkflowMethods<C>
where
    C: V7Methods,
{
    async fn list_workflows(client: &C, contains_str: Option<String>) -> Result<Vec<WorkflowV2>>;
    async fn assign_items(client: &C, data: &AssignItemPayload) -> Result<AssignItemResponse>;
    async fn get_workflows(client: &C) -> Result<Vec<WorkflowV2>>;
}

#[async_trait]
impl<C> WorkflowMethods<C> for WorkflowV2
where
    C: V7Methods + std::marker::Sync,
{
    async fn list_workflows(client: &C, contains_str: Option<String>) -> Result<Vec<WorkflowV2>>
    where
        C: V7Methods,
    {
        let response = if let Some(filter) = contains_str {
            client
                .get(&format!(
                    "v2/teams/{}/workflows?name_contains={}",
                    client.team(),
                    filter
                ))
                .await?
        } else {
            client
                .get(&format!("v2/teams/{}/workflows", client.team()))
                .await?
        };
        expect_http_ok!(response, Vec<WorkflowV2>)
    }

    async fn assign_items(client: &C, data: &AssignItemPayload) -> Result<AssignItemResponse> {
        let response = client
            .post(&format!("v2/teams/{}/items/assign", client.team()), &data)
            .await?;
        expect_http_ok!(response, AssignItemResponse)
    }

    async fn get_workflows(client: &C) -> Result<Vec<WorkflowV2>> {
        let response = client
            .get(&format!("v2/teams/{}/workflows", client.team()))
            .await?;
        expect_http_ok!(response, Vec<WorkflowV2>)
    }
}

impl Workflow {
    pub async fn assign<C>(&self, client: &C, user_id: &u32) -> Result<Workflow>
    where
        C: V7Methods,
    {
        let user = UserId { user_id: *user_id };

        let response = client
            .post(&format!("workflow_stages/{}/assign", self.id), &user)
            .await?;

        expect_http_ok!(response, Workflow)
    }

    /// Warning undocumented
    pub async fn add_comment<C>(
        &self,
        client: &C,
        comments: &LocatedWorkflowComments,
    ) -> Result<WorkflowCommentThread>
    where
        C: V7Methods,
    {
        let response = client
            .post(
                &format!("workflows/{}/workflow_comment_threads", self.id),
                comments,
            )
            .await?;

        expect_http_ok!(response, WorkflowCommentThread)
    }
}

impl WorkflowTemplate {
    pub async fn get<C>(client: &C, id: &u32) -> Result<WorkflowTemplate>
    where
        C: V7Methods,
    {
        let response = client.get(&format!("workflow_templates/{}", id)).await?;

        expect_http_ok!(response, WorkflowTemplate)
    }
}

impl WorkflowStageTemplate {
    pub async fn assign<C>(&self, client: &C) -> Result<WorkflowStageTemplate>
    where
        C: V7Methods,
    {
        let id = self
            .id
            .as_ref()
            .context("WorkflowStageTemplate id not specified")?;
        let response = client
            .put(&format!("workflow_stage_templates/{}", id), Some(&self))
            .await?;

        expect_http_ok!(response, WorkflowStageTemplate)
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_empty_ser_metadata() {
        let contents = "{}";

        let meta: MetaData = serde_json::from_str(contents).unwrap();

        assert_eq!(meta.ready_for_completion, None);
        assert_eq!(meta.previous_stage_number, None);
        assert_eq!(meta.review_status, None);
        assert_eq!(meta.review_status_modified_at, None);
    }

    #[test]
    fn test_ready_completion_ser_metadata() {
        let contents = r#"{"ready_for_completion": true}"#;

        let meta: MetaData = serde_json::from_str(contents).unwrap();

        assert_eq!(meta.ready_for_completion, Some(true));
        assert_eq!(meta.previous_stage_number, None);
        assert_eq!(meta.review_status, None);
        assert_eq!(meta.review_status_modified_at, None);
    }

    #[test]
    fn test_all_ser_metadata() {
        let contents = r#"{
            "previous_stage_number": 2,
            "ready_for_completion": true,
            "review_status": "approved",
            "review_status_modified_at": "2022-12-14T00:28:28.759303"
        }"#;

        let meta: MetaData = serde_json::from_str(contents).unwrap();

        assert_eq!(meta.ready_for_completion, Some(true));
        assert_eq!(meta.previous_stage_number, Some(2));
        assert_eq!(meta.review_status, Some("approved".to_string()));
        assert_eq!(
            meta.review_status_modified_at,
            Some("2022-12-14T00:28:28.759303".to_string())
        );
    }

    #[test]
    fn test_template_metadata() {
        let contents = "{
            \"assignable_to\": \"any_user\",
            \"base_sampling_rate\": 1.0,
            \"parallel\": 1,
            \"user_sampling_rate\": 1.0
        }";

        let template_meta: TemplateMetadata = serde_json::from_str(contents).unwrap();

        assert_eq!(template_meta.assignable_to, Some("any_user".to_string()));
        assert_eq!(template_meta.base_sampling_rate, Some(1.0));
        assert_eq!(template_meta.parallel, Some(1));
        assert_eq!(template_meta.user_sampling_rate, Some(1.0));
    }

    #[test]
    fn test_display_stage_type() {
        assert_eq!(format!("{}", StageType::Annotate), "Annotate");
        assert_eq!(format!("{}", StageType::Complete), "Complete");
        assert_eq!(format!("{}", StageType::Consensus), "Consensus");
        assert_eq!(format!("{}", StageType::Model), "Model");
        assert_eq!(format!("{}", StageType::New), "New");
        assert_eq!(format!("{}", StageType::Review), "Review");
    }

    #[test]
    fn test_serde_workflow_type() {
        // The JSON rep is lower case
        assert_eq!(
            serde_json::to_string(&StageType::Annotate).unwrap(),
            r#""annotate""#
        );
        assert_eq!(
            serde_json::to_string(&StageType::Review).unwrap(),
            r#""review""#
        );
        assert_eq!(
            serde_json::to_string(&StageType::Consensus).unwrap(),
            r#""consensus""#
        );
        assert_eq!(
            serde_json::to_string(&StageType::Model).unwrap(),
            r#""model""#
        );

        // The Enum rep is PascalCase
        assert_eq!(
            StageType::Annotate,
            serde_json::from_str(r#""annotate""#).unwrap()
        );

        assert_eq!(
            StageType::Review,
            serde_json::from_str(r#""review""#).unwrap()
        );

        assert_eq!(
            StageType::Consensus,
            serde_json::from_str(r#""consensus""#).unwrap()
        );

        assert_eq!(
            StageType::Model,
            serde_json::from_str(r#""model""#).unwrap()
        );
    }

    #[test]
    fn test_ser_stage() {
        let contents = r#"
        {
            "assignee_id": 12974,
            "completed": false,
            "completes_at": null,
            "dataset_item_id": 650713507,
            "id": 115470255,
            "metadata": {},
            "number": 1,
            "skipped": false,
            "skipped_reason": null,
            "template_metadata": {
                "assignable_to": "any_user",
                "base_sampling_rate": 1.0,
                "parallel": 1,
                "user_sampling_rate": 1.0
            },
            "type": "annotate",
            "workflow_id": 43051890,
            "workflow_stage_template_id": 166366
        }
        "#;

        let stage: Stage = serde_json::from_str(contents).unwrap();

        assert_eq!(stage.assignee_id, Some(12974));
        assert_eq!(stage.completed, Some(false));
        assert_eq!(stage.completes_at, None);
        assert_eq!(stage.id, 115470255);
        assert_eq!(stage.metadata.unwrap().ready_for_completion, None);
        assert_eq!(
            stage.template_metadata.unwrap().assignable_to,
            Some("any_user".to_string())
        );
        assert_eq!(stage.stage_type, Some(StageType::Annotate));
    }
}
