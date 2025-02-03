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
    Discard,
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
            StageType::Discard => "Discard",
        };
        write!(f, "{val}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct MetaData {
    pub ready_for_completion: Option<bool>,
    pub previous_stage_number: Option<u32>,
    pub review_status: Option<String>,
    pub review_status_modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
pub struct TemplateAssignee {
    pub assignee_id: Option<u32>,
    pub sampling_rate: Option<f64>,
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
    pub created_commands: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowDataset {
    pub annotation_hotkeys: Option<HashMap<String, String>>,
    pub annotators_can_instantiate_workflows: Option<bool>,
    pub id: Option<u32>,
    pub instructions: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowProgress {
    pub complete: Option<u32>,
    pub idle: Option<u32>,
    pub in_progress: Option<u32>,
    pub total: Option<u32>,
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
    pub class_mapping: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_non_default_v1_template: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_annotations: Option<bool>,
    pub initial: Option<bool>,
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
    pub rules: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skippable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_stage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub x: Option<u32>,
    pub y: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct StageEdge {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: Option<String>, //FIXME: What are the different types?
    pub source_stage_id: Option<String>,
    pub target_stage_id: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowStageAssignees {
    pub stage_id: Option<String>,
    pub user_id: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowStageV2 {
    pub assignable_users: Vec<Option<WorkflowStageAssignees>>,
    pub config: Option<StageConfig>,
    pub edges: Vec<Option<StageEdge>>,
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub stage_type: Option<StageType>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct WorkflowV2 {
    pub dataset: Option<WorkflowDataset>,
    pub id: Option<String>,
    pub inserted_at: Option<String>,
    pub name: Option<String>,
    pub progress: Option<WorkflowProgress>,
    pub stages: Vec<Option<WorkflowStageV2>>,
    pub team_id: Option<u32>,
    pub thumbnails: Vec<Option<String>>,
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_batch_requested: Option<bool>,
    #[serde(rename = "additionalProp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_prop: Option<u32>,
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
    async fn update_workflow(
        &self,
        client: &C,
        update_payload: &WorkflowBuilder,
    ) -> Result<WorkflowV2>;
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

    async fn update_workflow(
        &self,
        client: &C,
        update_payload: &WorkflowBuilder,
    ) -> Result<WorkflowV2> {
        let response = client
            .put(
                &format!(
                    "v2/teams/{}/workflows/{}",
                    client.team(),
                    self.id.as_ref().context("Id required")?
                ),
                Some(update_payload),
            )
            .await?;
        expect_http_ok!(response, WorkflowV2)
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
            "assignable_users": [],
            "config": {
              "allowed_class_ids": null,
              "annotation_group_id": null,
              "assignable_to": "anyone",
              "authorization_header": null,
              "auto_instantiate": false,
              "champion_stage_id": null,
              "class_mapping": [],
              "dataset_id": null,
              "from_non_default_v1_template": null,
              "include_annotations": false,
              "initial": false,
              "iou_thresholds": null,
              "model_id": null,
              "model_type": "gust",
              "parallel_stage_ids": null,
              "readonly": false,
              "retry_if_fails": false,
              "rules": [],
              "skippable": true,
              "test_stage_id": null,
              "threshold": null,
              "url": null,
              "x": 3746,
              "y": 2896
            },
            "edges": [
              {
                "id": "fe2370d3-4091-4c35-83fe-871c9dc45f45",
                "name": "reject",
                "source_stage_id": "9bba4506-694d-4dd3-afd8-ab354c5a21ba",
                "target_stage_id": "c95b5763-6da2-417a-bc06-106a45240ac0"
              },
              {
                "id": "5b233988-46f7-4009-955d-4dbd89239d1e",
                "name": "approve",
                "source_stage_id": "9bba4506-694d-4dd3-afd8-ab354c5a21ba",
                "target_stage_id": "3175bb69-b14e-4255-8bb3-3f19411dab4a"
              }
            ],
            "id": "9bba4506-694d-4dd3-afd8-ab354c5a21ba",
            "name": "Review",
            "type": "review"
        }
        "#;

        let stage: WorkflowStageV2 = serde_json::from_str(contents).unwrap();

        assert_eq!(stage.stage_type, Some(StageType::Review));
        assert_eq!(
            stage.id,
            Some("9bba4506-694d-4dd3-afd8-ab354c5a21ba".to_string())
        );
        assert_eq!(
            stage.edges[0]
                .clone()
                .expect("Missing edge")
                .source_stage_id,
            stage.edges[1]
                .clone()
                .expect("Missing edge")
                .source_stage_id
        )
    }
}
