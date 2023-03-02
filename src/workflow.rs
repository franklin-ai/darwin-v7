use crate::client::V7Client;
use crate::expect_http_ok;
use anyhow::{bail, Result};
use fake::{Dummy, Fake};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StageType {
    Annotate,
    Complete,
    Consensus,
    Model,
    New,
    Review,
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
        };
        write!(f, "{val}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct TemplateMetadata {
    pub assignable_to: Option<String>,
    pub base_sampling_rate: Option<f64>,
    pub parallel: Option<u32>,
    pub user_sampling_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct MetaData {
    pub ready_for_completion: Option<bool>,
    pub previous_stage_number: Option<u32>,
    pub review_status: Option<String>,
    pub review_status_modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct TemplateAssignee {
    pub assignee_id: u32,
    pub sampling_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct WorkflowStageTemplate {
    pub id: u32,
    pub metadata: TemplateMetadata,
    pub name: Option<String>,
    pub workflow_stage_template_assignees: Vec<TemplateAssignee>,
    pub stage_number: Option<u32>,
    #[serde(rename = "type")]
    pub stage_type: StageType,
    pub workflow_template_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct Workflow {
    pub current_stage_number: u32,
    pub current_workflow_stage_template_id: u32,
    pub dataset_item_id: u32,
    pub id: u32,
    pub stages: HashMap<String, Vec<Stage>>,
    pub status: String, // This can probably be an enum
    pub workflow_template_id: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct WorkflowTemplate {
    pub dataset_id: u32,
    pub id: u32,
    pub name: Option<String>,
    pub workflow_stage_templates: Vec<WorkflowStageTemplate>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
struct UserId {
    pub user_id: u32,
}

impl Workflow {
    pub async fn assign(&self, client: &V7Client, user_id: &u32) -> Result<Workflow> {
        let user = UserId { user_id: *user_id };

        let response = client
            .post(&format!("workflow_stages/{}/assign", self.id), &user)
            .await?;

        expect_http_ok!(response, Workflow)
    }
}

impl WorkflowTemplate {
    pub async fn get(client: &V7Client, id: &u32) -> Result<WorkflowTemplate> {
        let response = client.get(&format!("workflow_templates/{}", id)).await?;

        expect_http_ok!(response, WorkflowTemplate)
    }
}

impl WorkflowStageTemplate {
    pub async fn assign(&self, client: &V7Client) -> Result<WorkflowStageTemplate> {
        let response = client
            .put(
                &format!("workflow_stage_templates/{}", self.id),
                Some(&self),
            )
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
