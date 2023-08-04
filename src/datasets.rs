use crate::annotation::AnnotationClass;
use crate::client::V7Methods;
use crate::expect_http_ok;
use crate::filter::Filter;
use crate::item::{
    AddDataPayload, DataPayloadLevel, DatasetItem, DatasetItemStatus, DatasetItemTypes,
    DatasetItemV2, ExistingSimpleItem,
};
use crate::team::TypeCount;
use crate::workflow::{WorkflowBuilder, WorkflowMethodsV2, WorkflowTemplate, WorkflowV2};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use csv_async::AsyncReaderBuilder;
#[allow(unused_imports)]
use fake::{Dummy, Fake};
use futures::io::Cursor;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct AnnotationHotKeys {
    pub key: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy)]
pub struct Dataset {
    pub active: Option<bool>,
    pub archived: Option<bool>,
    pub archived_at: Option<String>,
    // TODO: Find out what the annotation_hotkeys HashMap actually is
    // the HashMap below is a placeholder for now, it is some kind of hashmap
    pub annotation_hotkeys: Option<HashMap<String, String>>,
    pub annotators_can_create_tags: Option<bool>,
    pub annotators_can_instantiate_workflows: Option<bool>,
    pub anyone_can_double_assign: Option<bool>,

    // TODO: annotations
    #[serde(skip)]
    pub annotation_classes: Vec<String>,

    pub default_workflow_template_id: Option<u32>,

    pub id: u32,
    pub inserted_at: Option<String>,
    pub instructions: Option<String>,

    pub name: Option<String>,
    pub num_annotations: Option<Option<u32>>,
    pub num_annotators: Option<Option<u32>>,
    pub num_classes: Option<u32>,
    pub num_complete_files: Option<u32>,
    pub num_images: Option<u32>,
    pub num_items: Option<u32>,
    pub num_videos: Option<u32>,
    pub owner_id: Option<u32>,
    pub parent_id: Option<u32>,
    pub pdf_fit_page: Option<bool>,
    pub progress: Option<f64>,
    pub public: Option<bool>,
    pub reviewers_can_annotate: Option<bool>,
    pub slug: String,
    pub team_id: Option<u32>,
    pub team_slug: Option<String>,

    // TODO: thumbnails
    #[serde(skip)]
    pub thumbnails: Option<Vec<String>>,
    pub updated_at: Option<String>,
    pub version: Option<u32>,
    pub work_size: Option<u32>,
    pub work_prioritization: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy)]
pub struct ExportMetadata {
    pub annotation_classes: Vec<AnnotationClass>,
    pub annotation_types: Vec<TypeCount>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy)]
pub struct Export {
    pub name: String,
    pub download_url: Option<String>,
    pub format: ExportFormat,
    pub inserted_at: String,
    pub latest: bool,
    #[serde(skip_deserializing)]
    pub metadata: ExportMetadata,
    pub status: Option<String>,
    pub version: u16,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    #[default]
    Json,
    Xml,
    Coco,
    Cvat,
    #[serde(rename = "pascal_voc")]
    PascalVoc,
    #[serde(rename = "semantic-mask")]
    SemanticMask,
    #[serde(rename = "instance-mask")]
    InstanceMask,
}

impl From<ExportFormat> for &str {
    fn from(value: ExportFormat) -> Self {
        match value {
            ExportFormat::Json => "json",
            ExportFormat::Xml => "xml",
            ExportFormat::Coco => "coco",
            ExportFormat::Cvat => "cvat",
            ExportFormat::PascalVoc => "pascal_voc",
            ExportFormat::SemanticMask => "semantic-mask",
            ExportFormat::InstanceMask => "instance-mask",
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
struct DatasetName {
    pub name: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
struct AddDataItemsPayload {
    pub items: Vec<AddDataPayload>,
    pub storage_name: String,
}

/// Version 2.0 equivalent of `AddDataItemsPayload`
#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct RegisterExistingItemPayload {
    /// Slug name of the Dataset to upload images to
    pub dataset_slug: String,
    /// Registered S3 storage bucket
    pub storage_slug: String,
    /// Details about image file
    pub items: Vec<ExistingSimpleItem>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct ResponseItem {
    pub dataset_item_id: u64,
    pub filename: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct AddDataItemsResponse {
    pub blocked_items: Vec<ResponseItem>,
    pub items: Vec<ResponseItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct SlotResponse {
    pub as_frames: bool,
    pub extract_views: bool,
    pub file_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub metadata: DataPayloadLevel,
    pub slot_name: String,
    pub size_bytes: u64,
    #[serde(rename = "type")]
    pub item_type: DatasetItemTypes,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct RegistrationResponseItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub slots: Vec<SlotResponse>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct RegisterExistingItemResponse {
    pub blocked_items: Vec<RegistrationResponseItem>,
    pub items: Vec<RegistrationResponseItem>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
struct ArchiveItemPayload {
    pub filter: Filter,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
struct AssignItemPayload {
    pub assignee_id: u32,
    pub filter: Filter,
}

#[derive(Serialize, Deserialize)]
struct GenerateExportPayload {
    pub name: String,
    pub format: String,
    pub include_authorship: bool,
    pub include_export_token: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResetToNewPayload {
    pub filter: Filter,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetStagePayload {
    pub workflow_stage_template_id: u32,
    pub filter: Filter,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetStageFilter {
    pub dataset_ids: Vec<u32>,
    pub select_all: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetStagePayloadV2 {
    pub filters: SetStageFilter,
    pub stage_id: String,
    pub workflow_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetStageResponse {
    pub created_commands: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct ItemReport {
    /// Original filename of the item
    pub filename: String,
    /// Timestamp of when item was added to the dataset
    pub uploaded_date: String,
    /// Current status of the dataset
    pub status: DatasetItemStatus,
    /// Timestamp of when item was first entered into a workflow
    pub workflow_start_date: Option<String>,
    /// Timestamp of when work on the item was completed. null if in progress
    pub workflow_complete_date: Option<String>,
    /// For playback videos, the number of frames in the video
    pub number_of_frames: Option<u32>,
    /// Path the item was assigned in the dataset
    pub folder: String,
    /// Total duration of work perform by annotators
    pub time_spent_annotating_sec: u64,
    /// Total duration of work perform by reviewers
    pub time_spent_reviewing_sec: u64,
    /// Total duration of automation actions performed in annotate stages
    pub automation_time_annotating_sec: u64,
    /// Total duration of automation actions performed in review stages
    pub automation_time_reviewing_sec: u64,
    /// Emails of all annotators who performed work on this item, joined by semicolon
    pub annotators: String,
    /// Emails of all reviewers who performed work on this item, joined by semicolon
    pub reviewers: String,
    /// True if item was every rejected in any review stage
    pub was_rejected_in_review: bool,
    /// Darwin Workview URL for the item
    pub url: String,
}

pub async fn item_reports_from_bytes(contents: &[u8]) -> Result<Vec<ItemReport>> {
    let cursor = Cursor::new(contents);
    let mut rdr = AsyncReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .create_deserializer(cursor);

    let mut records = rdr.deserialize::<ItemReport>();
    let mut results: Vec<ItemReport> = Vec::new();

    while let Some(record) = records.next().await {
        let record = record?;
        results.push(record)
    }

    Ok(results)
}

impl Dataset {
    #[allow(dead_code)]
    pub async fn create_dataset<C>(client: &C, name: &str) -> Result<Dataset>
    where
        C: V7Methods,
    {
        let response = client
            .post(
                "datasets",
                &DatasetName {
                    name: name.to_string(),
                },
            )
            .await?;

        expect_http_ok!(response, Dataset)
    }
}

#[async_trait]
pub trait DatasetArchiveMethods<C>
where
    C: V7Methods,
{
    // async fn archive_all_files(&self, client: &C) -> Result<()>;
    async fn archive_items(&self, client: &C, filter: &Filter) -> Result<()>;
    async fn archive_dataset(&self, client: &C) -> Result<Dataset>;
}

#[async_trait]
pub trait DatasetDataMethods<C>
where
    C: V7Methods,
{
    async fn assign_items(&self, client: &C, assignee_id: &u32, filter: &Filter) -> Result<()>;
    #[deprecated = "V2 of the V7 API requires use of `register_items_to_dataset`"]
    async fn add_data_to_dataset(
        &self,
        client: &C,
        data: Vec<AddDataPayload>,
        external_storage: String,
    ) -> Result<AddDataItemsResponse>;
    async fn register_items_to_dataset(
        &self,
        client: &C,
        data: Vec<ExistingSimpleItem>,
        external_storage: String,
    ) -> Result<RegisterExistingItemResponse>;
}

#[async_trait]
pub trait DatasetExportMethods<C>
where
    C: V7Methods,
{
    async fn generate_export(
        &self,
        client: &C,
        export_name: &'life2 str,
        format: &ExportFormat,
        include_authorship: bool,
        include_export_token: bool,
        filter: Option<&Filter>,
    ) -> Result<()>;
    async fn list_exports(&self, client: &C) -> Result<Vec<Export>>;
}

#[async_trait]
pub trait DatasetDescribeMethods<C>
where
    C: V7Methods,
{
    async fn list_datasets(client: &C) -> Result<Vec<Dataset>>;
    #[deprecated = "V2 of the V7 API requires use of `list_dataset_items_v2`"]
    async fn list_dataset_items(&self, client: &C) -> Result<Vec<DatasetItem>>;
    async fn list_dataset_items_v2(&self, client: &C) -> Result<Vec<DatasetItemV2>>;
    async fn show_dataset(client: &C, id: &u32) -> Result<Dataset>;
}

#[async_trait]
pub trait DatasetWorkflowMethods<C>
where
    C: V7Methods,
{
    async fn reset_to_new(&self, client: &C, filter: &Filter) -> Result<()>;
    async fn set_stage(&self, client: &C, stage_template_id: &u32, filter: &Filter) -> Result<()>;
    async fn set_workflow(
        &self,
        client: &C,
        workflow: &WorkflowTemplate,
    ) -> Result<WorkflowTemplate>;
    async fn set_workflow_v2(&self, client: &C, workflow: &WorkflowBuilder) -> Result<WorkflowV2>;
    async fn set_default_workflow(
        &self,
        client: &C,
        workflow: &WorkflowTemplate,
    ) -> Result<Dataset>;
    async fn get_workflow_v2(&self, client: &C) -> Result<Option<WorkflowV2>>;
    async fn set_stage_v2(
        &self,
        client: &C,
        stage_id: String,
        workflow_id: String,
    ) -> Result<SetStageResponse>;
}

#[async_trait]
pub trait DatasetItemReportMethods<C>
where
    C: V7Methods,
{
    async fn get_item_reports(&self, client: &C) -> Result<Vec<ItemReport>>;
}

#[async_trait]
impl<C> DatasetArchiveMethods<C> for Dataset
where
    C: V7Methods + std::marker::Sync,
{
    // async fn archive_all_files(&self, client: &C) -> Result<()> {
    //     let item_ids: Vec<u32> = Dataset::list_dataset_items(client)
    //         .await?
    //         .iter()
    //         .filter(|x| !x.archived)
    //         .map(|x| x.id.clone())
    //         .collect();
    //     let mut filter = Filter::default();
    //     filter.dataset_item_ids = Some(item_ids);

    //     self.archive_items(client, &filter).await
    // }

    /// The docs say a reason is required, but the call actually fails if it is provided
    /// https://docs.v7labs.com/v1.0/reference/archive
    async fn archive_items(&self, client: &C, filter: &Filter) -> Result<()> {
        let payload = ArchiveItemPayload {
            filter: filter.clone(),
        };

        let endpoint = &format!("datasets/{}/items/archive", self.id);
        let response = client.put(endpoint, Some(&payload)).await?;

        let status = response.status();

        // 204 is correct operation for this endpoint
        if status != 204 {
            bail!("Invalid status code {status}")
        }
        Ok(())
    }

    async fn archive_dataset(&self, client: &C) -> Result<Dataset> {
        let response = client
            .put::<String>(&format!("datasets/{}/archive", &self.id), None)
            .await?;

        expect_http_ok!(response, Dataset)
    }
}

#[async_trait]
impl<C> DatasetDataMethods<C> for Dataset
where
    C: V7Methods + std::marker::Sync,
{
    async fn assign_items(&self, client: &C, assignee_id: &u32, filter: &Filter) -> Result<()> {
        let payload = AssignItemPayload {
            assignee_id: *assignee_id,
            filter: filter.clone(),
        };

        let response = client
            .post(&format!("datasets/{}/assign_items", self.id), &payload)
            .await?;

        let status = response.status();

        // 204 is correct operation for this endpoint
        if status != 204 {
            bail!("Invalid status code {status}")
        }

        Ok(())
    }

    async fn add_data_to_dataset(
        &self,
        client: &C,
        data: Vec<AddDataPayload>,
        external_storage: String,
    ) -> Result<AddDataItemsResponse> {
        let api_payload = AddDataItemsPayload {
            items: data,
            storage_name: external_storage,
        };

        let endpoint = format!(
            "teams/{}/datasets/{}/data",
            self.team_slug
                .as_ref()
                .context("Dataset is missing team slug")?,
            self.slug
        );

        let response = client.put(&endpoint, Some(&api_payload)).await?;

        expect_http_ok!(response, AddDataItemsResponse)
    }

    // V7 Version 2
    async fn register_items_to_dataset(
        &self,
        client: &C,
        data: Vec<ExistingSimpleItem>,
        external_storage_slug: String,
    ) -> Result<RegisterExistingItemResponse> {
        let api_payload = RegisterExistingItemPayload {
            dataset_slug: self.slug.to_string(),
            storage_slug: external_storage_slug,
            items: data,
        };
        let endpoint = format!(
            "v2/teams/{}/items/register_existing_readonly",
            self.team_slug
                .as_ref()
                .context("Dataset is missing team slug")?
        );
        let response = client.post(&endpoint, &api_payload).await?;

        expect_http_ok!(response, RegisterExistingItemResponse)
    }
}

#[async_trait]
impl<C> DatasetExportMethods<C> for Dataset
where
    C: V7Methods + std::marker::Sync,
{
    async fn generate_export(
        &self,
        client: &C,
        export_name: &'life2 str,
        format: &ExportFormat,
        include_authorship: bool,
        include_export_token: bool,
        filter: Option<&Filter>,
    ) -> Result<()> {
        let endpoint = format!(
            "teams/{}/datasets/{}/exports",
            self.team_slug.as_ref().context("Missing team slug")?,
            self.slug
        );

        let payload = GenerateExportPayload {
            name: export_name.to_string(),
            format: Into::<&str>::into(format.clone()).to_string(),
            include_authorship,
            include_export_token,
            filter: filter.cloned(),
        };

        let response = client.post(&endpoint, &payload).await?;

        if response.status() != 200 {
            bail!(format!(
                "Invalid status code {} {}",
                response.status(),
                response.text().await?
            ))
        }

        Ok(())
    }

    async fn list_exports(&self, client: &C) -> Result<Vec<Export>> {
        let endpoint = format!(
            "teams/{}/datasets/{}/exports",
            self.team_slug.as_ref().context("Missing team slug")?,
            self.slug
        );

        let response = client.get(&endpoint).await?;

        expect_http_ok!(response, Vec<Export>)
    }
}

#[async_trait]
impl<C> DatasetDescribeMethods<C> for Dataset
where
    C: V7Methods + std::marker::Sync,
{
    async fn list_datasets(client: &C) -> Result<Vec<Dataset>> {
        let response = client.get("datasets").await?;

        expect_http_ok!(response, Vec<Dataset>)
    }

    async fn list_dataset_items(&self, client: &C) -> Result<Vec<DatasetItem>> {
        let response = client.get(&format!("datasets/{}/items", self.id)).await?;

        expect_http_ok!(response, Vec<DatasetItem>)
    }

    async fn list_dataset_items_v2(&self, client: &C) -> Result<Vec<DatasetItemV2>> {
        let response = client
            .get(&format!(
                "v2/teams/{}/items?dataset_ids={}",
                self.team_slug.as_ref().context("Missing team slug")?,
                self.id
            ))
            .await?;

        expect_http_ok!(response, Vec<DatasetItemV2>)
    }

    async fn show_dataset(client: &C, id: &u32) -> Result<Dataset> {
        let response = client.get(&format!("datasets/{}", id)).await?;

        expect_http_ok!(response, Dataset)
    }
}

#[async_trait]
impl<C> DatasetWorkflowMethods<C> for Dataset
where
    C: V7Methods + std::marker::Sync,
{
    async fn reset_to_new(&self, client: &C, filter: &Filter) -> Result<()> {
        let payload = ResetToNewPayload {
            filter: filter.clone(),
        };

        let response = client
            .put(
                &format!("datasets/{}/items/move_to_new", self.id),
                Some(&payload),
            )
            .await?;

        let status = response.status();

        // 204 is correct operation for this endpoint
        if status != 204 {
            bail!("Invalid status code {status}")
        }

        Ok(())
    }

    async fn set_stage(&self, client: &C, stage_template_id: &u32, filter: &Filter) -> Result<()> {
        let payload = SetStagePayload {
            workflow_stage_template_id: *stage_template_id,
            filter: filter.clone(),
        };

        let response = client
            .put(&format!("datasets/{}/set_stage", self.id), Some(&payload))
            .await?;

        let status = response.status();

        // 204 is correct operation for this endpoint
        if status != 204 {
            bail!("Invalid status code {status}")
        }

        Ok(())
    }

    async fn set_workflow(
        &self,
        client: &C,
        workflow: &WorkflowTemplate,
    ) -> Result<WorkflowTemplate> {
        let response = client
            .post(
                &format!("datasets/{}/workflow_templates", self.id),
                workflow,
            )
            .await?;

        expect_http_ok!(response, WorkflowTemplate)
    }

    async fn set_workflow_v2(&self, client: &C, workflow: &WorkflowBuilder) -> Result<WorkflowV2> {
        let response = client
            .post(&format!("v2/teams/{}/workflows", client.team()), workflow)
            .await?;

        expect_http_ok!(response, WorkflowV2)
    }

    async fn set_default_workflow(
        &self,
        client: &C,
        workflow: &WorkflowTemplate,
    ) -> Result<Dataset> {
        let workflow_id = workflow.id.as_ref().context("Workflow id not provided")?;

        let endpoint = format!(
            "datasets/{}/default_workflow_template/{}",
            self.id, workflow_id
        );
        let payload: Option<&WorkflowTemplate> = None;
        let response = client.put(&endpoint, payload).await?;

        expect_http_ok!(response, Dataset)
    }

    async fn get_workflow_v2(&self, client: &C) -> Result<Option<WorkflowV2>> {
        let workflows = WorkflowV2::get_workflows(client).await?;
        let dataset_name = self.name.as_ref().context("Missing dataset name")?;
        Ok(workflows
            .into_iter()
            .filter(|workflow| workflow.dataset.name == dataset_name.to_string())
            .collect::<Vec<_>>()
            .first()
            .cloned())
    }

    async fn set_stage_v2(
        &self,
        client: &C,
        stage_id: String,
        workflow_id: String,
    ) -> Result<SetStageResponse> {
        let payload = SetStagePayloadV2 {
            filters: SetStageFilter {
                dataset_ids: vec![self.id],
                select_all: false,
            },
            stage_id,
            workflow_id,
        };
        let response = client
            .post(&format!("v2/teams/{}/items/stage", client.team()), &payload)
            .await?;
        expect_http_ok!(response, SetStageResponse)
    }
}

#[async_trait]
impl<C> DatasetItemReportMethods<C> for Dataset
where
    C: V7Methods + std::marker::Sync,
{
    async fn get_item_reports(&self, client: &C) -> Result<Vec<ItemReport>> {
        let endpoint = format!(
            "teams/{}/datasets/{}/item_reports",
            self.team_slug.as_ref().context("Missing team slug")?,
            self.slug
        );
        let response = client.get(&endpoint).await?;
        let status = response.status();
        let result = response.text().await?;
        if status != 200 {
            bail!(format!("Invalid status code {} {}", status, result))
        } else {
            item_reports_from_bytes(result.as_bytes()).await
        }
    }
}

impl Display for Dataset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}/{}",
            self.id,
            self.team_slug.as_ref().unwrap_or(&"team-slug".to_string()),
            self.slug
        )
    }
}

#[cfg(test)]
mod test_client_calls {

    use super::*;
    use crate::client::V7Client;
    use fake::{Fake, Faker};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_list_datasets() {
        let mock_server = MockServer::start().await;
        let mock_data: Vec<Dataset> = fake::vec![Dataset; 2];

        let client: V7Client = V7Client::new(
            format!("{}/", mock_server.uri()),
            "api-key".to_string(),
            "some-team".to_string(),
        )
        .unwrap();

        Mock::given(method("GET"))
            .and(path("/datasets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_data.clone()))
            .mount(&mock_server)
            .await;

        let datasets = Dataset::list_datasets(&client).await.unwrap();

        // Pick a few values to avoid f64 comparison issues
        assert_eq!(datasets.len(), mock_data.len());
        assert_eq!(datasets[0].id, mock_data[0].id);
        assert_eq!(datasets[0].slug, mock_data[0].slug);
        assert_eq!(datasets[1].inserted_at, mock_data[1].inserted_at);
    }

    #[tokio::test]
    async fn test_list_datasets_status_error() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/datasets"))
            .respond_with(ResponseTemplate::new(412))
            .mount(&mock_server)
            .await;

        let client: V7Client = V7Client::new(
            format!("{}/", mock_server.uri()),
            "api-key".to_string(),
            "some-team".to_string(),
        )
        .unwrap();

        Dataset::list_datasets(&client)
            .await
            .expect_err("Invalid status code 412");
    }

    #[tokio::test]
    async fn test_list_datasets_data_error() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/datasets"))
            .respond_with(ResponseTemplate::new(200).set_body_string("dont_render"))
            .mount(&mock_server)
            .await;

        let client: V7Client = V7Client::new(
            format!("{}/", mock_server.uri()),
            "api-key".to_string(),
            "some-team".to_string(),
        )
        .unwrap();

        Dataset::list_datasets(&client)
            .await
            .expect_err("error decoding response body: expected value at line 1 column 1");
    }

    #[tokio::test]
    async fn test_list_dataset_items() {
        let mock_server = MockServer::start().await;
        let mock_data: Dataset = Faker.fake();
        let dset_id = mock_data.id;

        // Just generate two random values for comparison
        let mock_result_vec: Vec<DatasetItem> = fake::vec![DatasetItem; 2];

        let client: V7Client = V7Client::new(
            format!("{}/", mock_server.uri()),
            "api-key".to_string(),
            "some-team".to_string(),
        )
        .unwrap();

        Mock::given(method("GET"))
            .and(path(format!("/datasets/{dset_id}/items")))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_result_vec.clone()))
            .mount(&mock_server)
            .await;

        #[allow(deprecated)]
        let result: Vec<DatasetItem> = mock_data.list_dataset_items(&client).await.unwrap();

        // Only compare a few values, this is mostly testing the endpoint
        // invocation and not serde.
        assert_eq!(result.len(), mock_result_vec.len());
        assert_eq!(result[0].status, mock_result_vec[0].status);
        assert_eq!(
            result[result.len() - 1].id,
            mock_result_vec[mock_result_vec.len() - 1].id
        );
    }

    #[tokio::test]
    async fn test_list_dataset_items_status_error() {
        let mock_server = MockServer::start().await;
        let mock_data: Dataset = Faker.fake();
        let dset_id = mock_data.id;

        Mock::given(method("GET"))
            .and(path(format!("/datasets/{dset_id}/items")))
            .respond_with(ResponseTemplate::new(412))
            .mount(&mock_server)
            .await;

        let client: V7Client = V7Client::new(
            format!("{}/", mock_server.uri()),
            "api-key".to_string(),
            "some-team".to_string(),
        )
        .unwrap();

        #[allow(deprecated)]
        mock_data
            .list_dataset_items(&client)
            .await
            .expect_err("Invalid status code 412");
    }

    #[tokio::test]
    async fn test_get_item_reports() {
        let mock_server = MockServer::start().await;
        let mock_data = "filename,uploaded_date,status,workflow_start_date,workflow_complete_date,number_of_frames,folder,time_spent_annotating_sec,time_spent_reviewing_sec,automation_time_annotating_sec,automation_time_reviewing_sec,annotators,reviewers,was_rejected_in_review,url
somefilename,2023-05-10 14:15:27,complete,2023-05-10 14:16:17,2023-05-17 01:28:13,,/,320,1,2,3,kevin@mail.com,,false,https://darwin.v7labs.com/workview?dataset=123456&image=789";

        let mut dataset: Dataset = Faker.fake();
        let dataset_slug = dataset.slug.clone();
        let team_slug = "some-team";
        dataset.team_slug = Some(team_slug.to_string());

        Mock::given(method("GET"))
            .and(path(format!(
                "/teams/{team_slug}/datasets/{dataset_slug}/item_reports",
            )))
            .respond_with(ResponseTemplate::new(200).set_body_string(mock_data.to_string()))
            .mount(&mock_server)
            .await;

        let client: V7Client = V7Client::new(
            format!("{}/", mock_server.uri()),
            "api-key".to_string(),
            "some-team".to_string(),
        )
        .unwrap();

        let results = dataset.get_item_reports(&client).await.unwrap();

        assert_eq!(results.len(), 1);

        let result = results.first().unwrap();

        assert_eq!(result.filename, "somefilename".to_string());
    }

    #[tokio::test]
    async fn test_item_reports_from_bytes() {
        let filename = "somefilename";
        let uploaded_date = "2023-05-10 14:15:27";
        let status = "complete";
        let workflow_start_date = "2023-05-10 14:16:17";
        let workflow_complete_date = "2023-05-17 01:28:13";
        let number_of_frames = "";
        let folder = "/";
        let time_spent_annotating_sec = 320;
        let time_spent_reviewing_sec = 1;
        let automation_time_annotating_sec = 2;
        let automation_time_reviewing_sec = 3;
        let annotators = "kevin@mail.com";
        let reviewers = "";
        let was_rejected_in_review = false;
        let url = "https://darwin.v7labs.com/workview?dataset=123456&image=789";

        let mut content =
            "filename,uploaded_date,status,workflow_start_date,workflow_complete_date,\
        number_of_frames,folder,time_spent_annotating_sec,time_spent_reviewing_sec,\
        automation_time_annotating_sec,automation_time_reviewing_sec,annotators,reviewers,\
        was_rejected_in_review,url\n"
                .to_string();
        content.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            filename,
            uploaded_date,
            status,
            workflow_start_date,
            workflow_complete_date,
            number_of_frames,
            folder,
            time_spent_annotating_sec,
            time_spent_reviewing_sec,
            automation_time_annotating_sec,
            automation_time_reviewing_sec,
            annotators,
            reviewers,
            was_rejected_in_review,
            url
        ));

        let results = item_reports_from_bytes(content.as_bytes()).await.unwrap();
        assert_eq!(results.len(), 1);

        let result = results.first().unwrap();

        assert_eq!(result.filename, filename.to_string());
        assert_eq!(result.uploaded_date, uploaded_date.to_string());
        assert_eq!(result.status, DatasetItemStatus::Complete);
        assert_eq!(
            result.workflow_start_date,
            Some(workflow_start_date.to_string())
        );
        assert_eq!(
            result.workflow_complete_date,
            Some(workflow_complete_date.to_string())
        );
        assert_eq!(result.number_of_frames, None);
        assert_eq!(result.folder, folder.to_string());
        assert_eq!(result.time_spent_annotating_sec, time_spent_annotating_sec);
        assert_eq!(result.time_spent_reviewing_sec, time_spent_reviewing_sec);
        assert_eq!(
            result.automation_time_annotating_sec,
            automation_time_annotating_sec
        );
        assert_eq!(
            result.automation_time_reviewing_sec,
            automation_time_reviewing_sec
        );
        assert_eq!(result.annotators, annotators.to_string());
        assert_eq!(result.reviewers, reviewers.to_string());
        assert_eq!(result.was_rejected_in_review, was_rejected_in_review);
        assert_eq!(result.url, url);
    }
}
