#[allow(unused_imports)]
use fake::Dummy;

use crate::annotation::AnnotationClass;
use crate::client::V7Methods;
use crate::expect_http_ok;
use crate::filter::Filter;
use crate::imports::AnnotationImport;
use crate::item::{
    AddDataPayload, DataPayloadLevel, DatasetItemStatus, DatasetItemTypes, ExistingSimpleItem, Item,
};
use crate::team::TypeCount;
use crate::workflow::{WorkflowBuilder, WorkflowMethods, WorkflowV2};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use csv_async::AsyncReaderBuilder;
use futures::io::Cursor;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Display;

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnnotationHotKeys {
    pub key: String,
}

#[derive(Debug, Default, Clone, Dummy, Serialize, Deserialize)]
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
    pub annotation_classes: Vec<Option<String>>,

    pub default_workflow_template_id: Option<u32>,

    pub id: Option<u32>,
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
    pub slug: Option<String>,
    pub team_id: Option<u32>,
    pub team_slug: Option<String>,

    // TODO: thumbnails
    #[serde(skip)]
    pub thumbnails: Vec<Option<String>>,
    pub updated_at: Option<String>,
    pub version: Option<u32>,
    pub work_size: Option<u32>,
    pub work_prioritization: Option<String>,
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DatasetUpdate {
    pub annotation_hotkeys: Option<HashMap<String, String>>,
    pub annotators_can_create_tags: Option<bool>,
    pub annotators_can_instantiate_workflows: Option<bool>,
    pub anyone_can_double_assign: Option<bool>,

    pub instructions: Option<String>,

    pub name: Option<String>,
    pub public: Option<bool>,
    pub reviewers_can_annotate: Option<bool>,
    pub work_size: Option<u32>,
    pub work_prioritization: Option<String>,
}

impl From<&Dataset> for DatasetUpdate {
    fn from(value: &Dataset) -> Self {
        DatasetUpdate {
            annotation_hotkeys: value.annotation_hotkeys.clone(),
            annotators_can_create_tags: value.annotators_can_create_tags,
            annotators_can_instantiate_workflows: value.annotators_can_instantiate_workflows,
            anyone_can_double_assign: value.anyone_can_double_assign,
            instructions: value.instructions.clone(),
            name: value.name.clone(),
            public: value.public,
            reviewers_can_annotate: value.reviewers_can_annotate,
            work_size: value.work_size,
            work_prioritization: value.work_prioritization.clone(),
        }
    }
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub annotation_classes: Vec<Option<AnnotationClass>>,
    pub annotation_types: Vec<Option<TypeCount>>,
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Export {
    pub name: Option<String>,
    pub download_url: Option<String>,
    pub format: Option<ExportFormat>,
    pub inserted_at: Option<String>,
    pub latest: Option<bool>,
    #[serde(skip_deserializing)]
    pub metadata: ExportMetadata,
    pub status: Option<String>,
    pub version: Option<u16>,
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    #[default]
    #[serde(rename = "darwin_json_2")]
    DarwinJson2,
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
            ExportFormat::DarwinJson2 => "darwin_json_2",
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct DatasetName {
    pub name: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct AddDataItemsPayload {
    pub items: Vec<AddDataPayload>,
    pub storage_name: String,
}

/// Version 2.0 equivalent of `AddDataItemsPayload`
///
#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterExistingItemPayload {
    /// Slug name of the Dataset to upload images to
    pub dataset_slug: String,
    /// Registered S3 storage bucket
    pub storage_slug: String,
    /// Details about image file
    pub items: Vec<ExistingSimpleItem>,
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResponseItem {
    pub dataset_item_id: Option<u64>,
    pub filename: Option<String>,
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchiveResponseItems {
    pub affected_item_count: Option<i32>,
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddDataItemsResponse {
    pub blocked_items: Vec<Option<ResponseItem>>,
    pub items: Vec<Option<ResponseItem>>,
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistrationResponseItem {
    pub id: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub slots: Vec<Option<SlotResponse>>,
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterExistingItemResponse {
    pub blocked_items: Vec<Option<RegistrationResponseItem>>,
    pub items: Vec<Option<RegistrationResponseItem>>,
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ArchiveItemPayload {
    pub filters: Filter,
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub filters: Option<Filter>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_stage_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetStagePayloadV2 {
    pub filters: SetStageFilter,
    pub stage_id: String,
    pub workflow_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetStageResponse {
    pub created_commands: Option<u32>,
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemReport {
    /// Original filename of the item
    pub filename: Option<String>,
    /// Timestamp of when item was added to the dataset
    pub uploaded_date: Option<String>,
    /// Current status of the dataset
    pub status: Option<DatasetItemStatus>,
    /// Timestamp of when item was first entered into a workflow
    pub workflow_start_date: Option<String>,
    /// Timestamp of when work on the item was completed. null if in progress
    pub workflow_complete_date: Option<String>,
    /// For playback videos, the number of frames in the video
    pub number_of_frames: Option<u32>,
    /// Path the item was assigned in the dataset
    pub folder: Option<String>,
    /// Total duration of work perform by annotators
    pub time_spent_annotating_sec: Option<u64>,
    /// Total duration of work perform by reviewers
    pub time_spent_reviewing_sec: Option<u64>,
    /// Total duration of automation actions performed in annotate stages
    pub automation_time_annotating_sec: Option<u64>,
    /// Total duration of automation actions performed in review stages
    pub automation_time_reviewing_sec: Option<u64>,
    /// Emails of all annotators who performed work on this item, joined by semicolon
    pub annotators: Option<String>,
    /// Emails of all reviewers who performed work on this item, joined by semicolon
    pub reviewers: Option<String>,
    /// True if item was every rejected in any review stage
    pub was_rejected_in_review: Option<bool>,
    /// Darwin Workview URL for the item
    pub url: Option<String>,
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
    async fn archive_items(&self, client: &C, filter: &Filter) -> Result<ArchiveResponseItems>;
    async fn archive_dataset(&self, client: &C) -> Result<Dataset>;
}

#[async_trait]
pub trait DatasetDataMethods<C>
where
    C: V7Methods,
{
    async fn assign_items(&self, client: &C, assignee_id: &u32, filter: &Filter) -> Result<()>;
    async fn update_batch_size(&self, client: &C, size: &u32) -> Result<()>;
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

    async fn update_annotation_hotkeys(
        &self,
        client: &C,
        hotkeys: HashMap<String, String>,
    ) -> Result<()>;

    /// Asynchronously imports an annotation into this dataset.
    ///
    /// This function takes a reference to a client, an item ID, and an annotation import object,
    /// and attempts to add the annotation to the specified item in the dataset. The operation
    /// is performed asynchronously.
    ///
    /// # Arguments
    /// * `client` - A reference to the client used to access V7.
    /// * `item_id` - The identifier of the item to which the annotation should be added.
    /// * `annotation_import` - A reference to the `AnnotationImport` object containing the
    ///   annotation data to be imported.
    async fn import_annotation(
        &self,
        client: &C,
        item_id: &str,
        annotation_import: &AnnotationImport,
    ) -> Result<()>;
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
    async fn list_exports(&self, client: &C) -> Result<Vec<Option<Export>>>;
}

#[async_trait]
pub trait DatasetDescribeMethods<C>
where
    C: V7Methods,
{
    async fn list_datasets(client: &C) -> Result<Vec<Option<Dataset>>>;
    async fn list_dataset_items_v2(&self, client: &C) -> Result<Item>;
    async fn show_dataset(client: &C, id: &u32) -> Result<Dataset>;
    async fn update_instructions(&self, client: &C, instructions: &str) -> Result<()>;
}

#[async_trait]
pub trait DatasetWorkflowMethods<C>
where
    C: V7Methods,
{
    async fn reset_to_new(&self, client: &C, filter: &Filter) -> Result<()>;
    async fn set_workflow_v2(&self, client: &C, workflow: &WorkflowBuilder) -> Result<WorkflowV2>;
    async fn get_workflow_v2(&self, client: &C) -> Result<Option<WorkflowV2>>;
    async fn set_stage_v2(
        &self,
        client: &C,
        stage_id: String,
        workflow_id: String,
        filters: Option<SetStageFilter>,
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
    /// The docs say a reason is required, but the call actually fails if it is provided
    /// https://docs.v7labs.com/v1.0/reference/archive
    async fn archive_items(&self, client: &C, filter: &Filter) -> Result<ArchiveResponseItems> {
        let payload = ArchiveItemPayload {
            filters: filter.clone(),
        };

        let endpoint = &format!(
            "v2/teams/{}/items/archive",
            self.team_slug
                .as_ref()
                .context("Dataset is missing team slug")?
        );
        let response = client.post(endpoint, &payload).await?;
        expect_http_ok!(response, ArchiveResponseItems)
    }

    async fn archive_dataset(&self, client: &C) -> Result<Dataset> {
        let response = client
            .put::<String>(
                &format!("datasets/{}/archive", &self.id.context("Id required")?),
                None,
            )
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
            .post(
                &format!("datasets/{}/assign_items", self.id.context("Id required")?),
                &payload,
            )
            .await?;

        let status = response.status();

        // 204 is correct operation for this endpoint
        if status != 204 {
            bail!("Invalid status code {status}")
        }

        Ok(())
    }

    async fn update_batch_size(&self, client: &C, size: &u32) -> Result<()> {
        let mut payload = DatasetUpdate::from(self);
        payload.work_size = Some(*size); // this PUT path requires every parameter
                                         // even if we're not updating them
                                         // so we have to replicate the rest of the existing settings

        let response = client
            .put(
                &format!("datasets/{}", self.id.context("Id required")?),
                Some(&payload),
            )
            .await?;
        let status = response.status();

        if status != 200 {
            bail!("Invalid status code {status}");
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
            self.slug.as_ref().context("Dataset is missing slug")?
        );

        let response = client.put(&endpoint, Some(&api_payload)).await?;

        expect_http_ok!(response, AddDataItemsResponse)
    }

    async fn register_items_to_dataset(
        &self,
        client: &C,
        data: Vec<ExistingSimpleItem>,
        external_storage_slug: String,
    ) -> Result<RegisterExistingItemResponse> {
        let api_payload = RegisterExistingItemPayload {
            dataset_slug: self
                .slug
                .as_ref()
                .context("Dataset is missing slug")?
                .to_string(),
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

    async fn update_annotation_hotkeys(
        &self,
        client: &C,
        hotkeys: HashMap<String, String>,
    ) -> Result<()> {
        let mut payload = DatasetUpdate::from(self);
        payload.annotation_hotkeys = Some(hotkeys);
        let response = client
            .put(
                &format!("datasets/{}", self.id.context("Dataset is missing Id")?),
                Some(&payload),
            )
            .await?;
        let status = response.status();
        if status != 200 {
            bail!("Invalid status code {status}");
        }
        Ok(())
    }

    /// Asynchronously imports an annotation into a dataset.
    ///
    /// Posts `annotation_import` data to a constructed endpoint using `item_id`. Checks for
    /// a successful 200 HTTP response status. Errors if the dataset lacks a team slug or
    /// if the response status indicates failure.
    ///
    /// # Arguments
    /// * `client` - Client for HTTP operations.
    /// * `item_id` - Identifier for the target item.
    /// * `annotation_import` - Data for the annotation to be imported.
    ///
    /// # Returns
    /// `Result<()>` indicating success or failure.
    async fn import_annotation(
        &self,
        client: &C,
        item_id: &str,
        annotation_import: &AnnotationImport,
    ) -> Result<()> {
        let endpoint = format!(
            "v2/teams/{team_slug}/items/{item_id}/import",
            team_slug = self.team_slug.as_ref().with_context(|| format!(
                "Dataset is missing team slug. dataset slug: {:?}",
                self.slug
            ))?
        );
        let response = client.post(&endpoint, annotation_import).await?;
        let status = response.status();
        if status != 200 {
            bail!("Import Annotation: Invalid status code {status}");
        }
        Ok(())
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
            "v2/teams/{}/datasets/{}/exports",
            self.team_slug.as_ref().context("Missing team slug")?,
            self.slug.as_ref().context("Dataset is missing slug")?
        );

        let payload = GenerateExportPayload {
            name: export_name.to_string(),
            format: Into::<&str>::into(format.clone()).to_string(),
            include_authorship,
            include_export_token,
            filters: filter.cloned(),
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

    async fn list_exports(&self, client: &C) -> Result<Vec<Option<Export>>> {
        let endpoint = format!(
            "v2/teams/{}/datasets/{}/exports",
            self.team_slug.as_ref().context("Missing team slug")?,
            self.slug.as_ref().context("Dataset is missing slug")?
        );

        let response = client.get(&endpoint).await?;

        expect_http_ok!(response, Vec<Option<Export>>)
    }
}

#[async_trait]
impl<C> DatasetDescribeMethods<C> for Dataset
where
    C: V7Methods + std::marker::Sync,
{
    async fn list_datasets(client: &C) -> Result<Vec<Option<Dataset>>> {
        let response = client.get("datasets").await?;

        expect_http_ok!(response, Vec<Option<Dataset>>)
    }
    async fn list_dataset_items_v2(&self, client: &C) -> Result<Item> {
        let response = client
            .get(&format!(
                "v2/teams/{}/items?dataset_ids={}",
                self.team_slug.as_ref().context("Missing team slug")?,
                self.id.context("Dataset is missing Id")?
            ))
            .await?;

        expect_http_ok!(response, Item)
    }

    async fn show_dataset(client: &C, id: &u32) -> Result<Dataset> {
        let response = client.get(&format!("datasets/{id}")).await?;

        expect_http_ok!(response, Dataset)
    }
    async fn update_instructions(&self, client: &C, instructions: &str) -> Result<()> {
        let mut payload = DatasetUpdate::from(self);
        payload.instructions = Some(instructions.to_string());
        let response = client
            .put(
                &format!("datasets/{}", self.id.context("Id required")?),
                Some(&payload),
            )
            .await?;
        if response.status() != 200 {
            bail!("Invalid status code {}", response.status());
        }
        Ok(())
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
                &format!(
                    "datasets/{}/items/move_to_new",
                    self.id.context("Dataset missing Id")?
                ),
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

    async fn set_workflow_v2(&self, client: &C, workflow: &WorkflowBuilder) -> Result<WorkflowV2> {
        let response = client
            .post(&format!("v2/teams/{}/workflows", client.team()), workflow)
            .await?;
        // 201 is correct operation for this endpoint
        if response.status() != 201 {
            bail!(
                "Invalid status code {}. Response: {}",
                response.status(),
                response.text().await?
            )
        }
        Ok(response.json().await?)
    }
    async fn get_workflow_v2(&self, client: &C) -> Result<Option<WorkflowV2>> {
        let workflows = WorkflowV2::get_workflows(client).await?;
        let dataset_name = self.name.as_ref().context("Missing dataset name")?;
        Ok(workflows
            .into_iter()
            .filter(|workflow| workflow.dataset.is_some())
            .filter(|workflow| {
                let dataset = workflow
                    .dataset
                    .as_ref()
                    .expect("No associated dataset to workflow");
                dataset.name.as_ref() == Some(dataset_name)
            })
            .collect::<Vec<_>>()
            .first()
            .cloned())
    }

    async fn set_stage_v2(
        &self,
        client: &C,
        stage_id: String,
        workflow_id: String,
        filters: Option<SetStageFilter>,
    ) -> Result<SetStageResponse> {
        let filters = if filters.is_none() {
            SetStageFilter {
                dataset_ids: vec![self.id.context("Dataset missing Id")?],
                select_all: true,
                workflow_stage_ids: None,
            }
        } else {
            filters.context("Invalid filter to set stage")?
        };

        let payload = SetStagePayloadV2 {
            filters,
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
            self.slug.as_ref().context("Dataset missing slug")?
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
            "{:?}:{}/{:?}",
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

    // Utilizing Faker with an AlwaysTrueRng to guarantee that all Option types are populated with Some values
    // This ensures consistent data generation where no field is left as None
    use crate::item::DatasetItemV2;
    use fake::utils::AlwaysTrueRng;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
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
        .expect("Failed to get V7Client");

        Mock::given(method("GET"))
            .and(path("/datasets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_data.clone()))
            .mount(&mock_server)
            .await;

        let datasets = Dataset::list_datasets(&client)
            .await
            .expect("Failed to list datasets");

        // Pick a few values to avoid f64 comparison issues
        assert_eq!(datasets.len(), mock_data.len());
        assert_eq!(
            datasets[0].as_ref().expect("Expected dataset").id,
            mock_data[0].id
        );
        assert_eq!(
            datasets[0].as_ref().expect("Expected dataset").slug,
            mock_data[0].slug
        );
        assert_eq!(
            datasets[1].as_ref().expect("Expected dataset").inserted_at,
            mock_data[1].inserted_at
        );
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
        .expect("Failed to get V7Client");

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
        .expect("Failed to get V7Client");

        Dataset::list_datasets(&client)
            .await
            .expect_err("error decoding response body: expected value at line 1 column 1");
    }

    #[tokio::test]
    async fn test_list_dataset_items() {
        let mock_server = MockServer::start().await;
        let mut rng = AlwaysTrueRng::default();
        let mut mock_data: Dataset = Faker.fake_with_rng(&mut rng);
        mock_data.team_slug = Some("some-team".to_string());

        let dset_id = mock_data.id.expect("Id must be set");

        // Just generate two random values for comparison
        // let mock_result_vec: Vec<DatasetItemV2> = vec![mock_dataset_item; 2];
        let mock_result: Item = Item {
            items: vec![Some(DatasetItemV2 {
                id: Some("123".to_string()),
                ..Default::default()
            })],
            ..Default::default()
        };

        let mock_items: Vec<_> = mock_result.items.iter().flatten().collect();

        let client: V7Client = V7Client::new(
            format!("{}/", mock_server.uri()),
            "api-key".to_string(),
            "some-team".to_string(),
        )
        .expect("Failed to get V7Client");

        Mock::given(method("GET"))
            .and(path("/v2/teams/some-team/items"))
            .and(query_param("dataset_ids".to_string(), dset_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_result.clone()))
            .mount(&mock_server)
            .await;

        let result = mock_data
            .list_dataset_items_v2(&client)
            .await
            .expect("Failed to list dataset items");
        let expected_items: Vec<_> = result.items.iter().flatten().collect();

        // Only compare a few values, this is mostly testing the endpoint
        // invocation and not serde.
        assert_eq!(expected_items.len(), mock_items.len());
        assert_eq!(expected_items[0].status, mock_items[0].status);
        assert_eq!(
            expected_items[expected_items.len() - 1].id,
            mock_items[mock_items.len() - 1].id
        );
    }

    #[tokio::test]
    async fn test_archive_dataset_items() {
        let mock_server = MockServer::start().await;
        let mut rng = AlwaysTrueRng::default();
        let mut mock_data: Dataset = Faker.fake_with_rng(&mut rng);
        let team_slug = mock_data.id;
        mock_data.team_slug = team_slug.map(|s| s.to_string());

        let dset_id: Option<Vec<u32>> = Some(vec![mock_data.id.expect("Id must be set")]);
        let complete_status: Option<Vec<String>> = Some(vec!["Complete".to_string()]);

        let filter = Filter {
            dataset_ids: dset_id,
            statuses: complete_status,
            ..Default::default()
        };

        let client: V7Client = V7Client::new(
            format!("{}/", mock_server.uri()),
            "api-key".to_string(),
            "some-team".to_string(),
        )
        .expect("Failed to get V7Client");

        Mock::given(method("POST"))
            .and(path(format!(
                "v2/teams/{}/items/archive",
                team_slug.expect("Team slug should be set")
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "affected_item_count": 1,
            })))
            .mount(&mock_server)
            .await;

        let result = mock_data
            .archive_items(&client, &filter)
            .await
            .expect("Failed to archive items");

        assert_eq!(result.affected_item_count, Some(1));
    }

    #[tokio::test]
    async fn test_list_dataset_items_status_error() {
        let mock_server = MockServer::start().await;

        let mut rng = AlwaysTrueRng::default();
        let mock_data: Dataset = Faker.fake_with_rng(&mut rng);
        let dset_id = mock_data.id.expect("Id must be set");

        Mock::given(method("GET"))
            .and(path("/v2/teams/some-team/items"))
            .and(query_param("dataset_ids".to_string(), dset_id.to_string()))
            .respond_with(ResponseTemplate::new(412))
            .mount(&mock_server)
            .await;

        let client: V7Client = V7Client::new(
            format!("{}/", mock_server.uri()),
            "api-key".to_string(),
            "some-team".to_string(),
        )
        .expect("Failed to get V7 Client");

        mock_data
            .list_dataset_items_v2(&client)
            .await
            .expect_err("Invalid status code 412");
    }

    #[tokio::test]
    async fn test_get_item_reports() {
        let mock_server = MockServer::start().await;
        let mock_data = "filename,uploaded_date,status,workflow_start_date,workflow_complete_date,number_of_frames,folder,time_spent_annotating_sec,time_spent_reviewing_sec,automation_time_annotating_sec,automation_time_reviewing_sec,annotators,reviewers,was_rejected_in_review,url
somefilename,2023-05-10 14:15:27,complete,2023-05-10 14:16:17,2023-05-17 01:28:13,,/,320,1,2,3,kevin@mail.com,,false,https://darwin.v7labs.com/workview?dataset=123456&image=789";

        let mut rng = AlwaysTrueRng::default();
        let mut dataset: Dataset = Faker.fake_with_rng(&mut rng);
        let dataset_slug = dataset
            .slug
            .clone()
            .expect("Dataset slug should not be null");
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
        .expect("Failed to create V7 client");

        let results = dataset
            .get_item_reports(&client)
            .await
            .expect("Failed to get item reports");

        assert_eq!(results.len(), 1);

        let result = results.first().expect("Get item reports had no result");

        assert_eq!(result.filename, Some("somefilename".to_string()));
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

        let results = item_reports_from_bytes(content.as_bytes())
            .await
            .expect("Failed to get item reports from bytes");
        assert_eq!(results.len(), 1);

        let result = results.first().expect("Item report results were empty");

        assert_eq!(result.filename, Some(filename.to_string()));
        assert_eq!(result.uploaded_date, Some(uploaded_date.to_string()));
        assert_eq!(result.status, Some(DatasetItemStatus::Complete));
        assert_eq!(
            result.workflow_start_date,
            Some(workflow_start_date.to_string())
        );
        assert_eq!(
            result.workflow_complete_date,
            Some(workflow_complete_date.to_string())
        );
        assert_eq!(result.number_of_frames, None);
        assert_eq!(result.folder, Some(folder.to_string()));
        assert_eq!(
            result.time_spent_annotating_sec,
            Some(time_spent_annotating_sec)
        );
        assert_eq!(
            result.time_spent_reviewing_sec,
            Some(time_spent_reviewing_sec)
        );
        assert_eq!(
            result.automation_time_annotating_sec,
            Some(automation_time_annotating_sec)
        );
        assert_eq!(
            result.automation_time_reviewing_sec,
            Some(automation_time_reviewing_sec)
        );
        assert_eq!(result.annotators, Some(annotators.to_string()));
        assert_eq!(result.reviewers, None);
        assert_eq!(result.was_rejected_in_review, Some(was_rejected_in_review));
        assert_eq!(result.url, Some(url.to_string()));
    }
}
