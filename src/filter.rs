#[allow(unused_imports)]
use fake::{Dummy, Fake};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct Filter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub accuracy_from: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_statuses: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_workflow_stage_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_item_name_contains: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub iou_threshold: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_class_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_current_assignees: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_assignees: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_paths: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_names: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_item_names: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_assignees: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_comments: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_path_prefix: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_name_prefix: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_item_paths: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_name_contains: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_annotation_class_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_from: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation_metrics_run_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_item_ids: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_ids: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_item_name_prefix: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub accuracy_to: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation_metrics_run_otucomes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_item_path_prefix: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_stage_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_types: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_to: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_all: Option<bool>,
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_serializing_default() {
        let filter = Filter::default();

        assert_eq!("{}", serde_json::to_string(&filter).unwrap());
    }

    #[test]
    fn test_simple_serde() {
        let mut filter = Filter {
            select_all: Some(true),
            ..Default::default()
        };
        let val: Vec<u32> = vec![1, 2, 3, 4];
        filter.annotation_class_ids = Some(val);

        let filter_str = r#"{"annotation_class_ids":[1,2,3,4],"select_all":true}"#;

        assert_eq!(filter_str, serde_json::to_string(&filter).unwrap());

        let new_filter: Filter = serde_json::from_str(filter_str).unwrap();

        assert_eq!(new_filter, filter);
    }
}
