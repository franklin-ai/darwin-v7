use crate::client::V7Methods;
use crate::expect_http_ok;
use crate::filter::Filter;
use crate::item::{AddDataPayload, DatasetItem};
use crate::team::{AnnotationClass, TypeCount};
use crate::workflow::WorkflowTemplate;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use fake::{Dummy, Fake};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct AnnotationHotKeys {
    pub key: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct ExportMetadata {
    pub annotation_classes: Vec<AnnotationClass>,
    pub annotation_types: Vec<TypeCount>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct Export {
    pub download_url: String,
    pub format: String,
    pub inserted_at: String,
    pub latest: bool,
    pub metadata: ExportMetadata,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
struct DatasetName {
    pub name: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
struct AddDataItemsPayload {
    pub items: Vec<AddDataPayload>,
    pub storage_name: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct ResponseItem {
    pub dataset_item_id: u64,
    pub filename: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct AddDataItemsResponse {
    pub blocked_items: Vec<ResponseItem>,
    pub items: Vec<ResponseItem>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
struct ArchiveItemPayload {
    pub filter: Filter,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
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
    pub filter: Filter,
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
    async fn add_data_to_dataset(
        &self,
        client: &C,
        data: Vec<AddDataPayload>,
        external_storage: String,
    ) -> Result<AddDataItemsResponse>;
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
        filter: &Filter,
    ) -> Result<()>;
    async fn list_exports(&self, client: &C) -> Result<Vec<Export>>;
}

#[async_trait]
pub trait DatasetDescribeMethods<C>
where
    C: V7Methods,
{
    async fn list_datasets(client: &C) -> Result<Vec<Dataset>>;
    async fn list_dataset_items(&self, client: &C) -> Result<Vec<DatasetItem>>;
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
    async fn set_default_workflow(
        &self,
        client: &C,
        workflow: &WorkflowTemplate,
    ) -> Result<Dataset>;
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
        filter: &Filter,
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
            filter: filter.clone(),
        };

        let response = client.post(&endpoint, &payload).await?;

        expect_http_ok!(response, ())
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
    use fake::Faker;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_list_datasets() {
        let mock_server = MockServer::start().await;
        let mock_data: Vec<Dataset> = fake::vec![Dataset; 2];

        let client: V7Client = V7Client::new(
            format!("{}/", mock_server.uri()).to_string(),
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
            format!("{}/", mock_server.uri()).to_string(),
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
            format!("{}/", mock_server.uri()).to_string(),
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
            format!("{}/", mock_server.uri()).to_string(),
            "api-key".to_string(),
            "some-team".to_string(),
        )
        .unwrap();

        Mock::given(method("GET"))
            .and(path(format!("/datasets/{dset_id}/items")))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_result_vec.clone()))
            .mount(&mock_server)
            .await;

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
            format!("{}/", mock_server.uri()).to_string(),
            "api-key".to_string(),
            "some-team".to_string(),
        )
        .unwrap();

        mock_data
            .list_dataset_items(&client)
            .await
            .expect_err("Invalid status code 412");
    }
}
