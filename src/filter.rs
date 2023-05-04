use fake::Dummy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct Filter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_class_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_annotation_class_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_current_assignees: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_item_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_dataset_item_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename_contains: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_filename_contains: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filenames: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_filenames: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_paths: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_path_prefix: Option<String>,

    #[serde(skip_serializing_if = "Filter::no_serialize_select")]
    pub select_all: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_statuses: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_types: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_video_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_stage_template_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_workflow_stage_template_ids: Option<Vec<u32>>,
}

impl Filter {
    // To my knowledge serde doesn't provide a
    // simpler way to not serialize false or default
    // values (without using nightly)
    fn no_serialize_select(val: &bool) -> bool {
        !*val
    }
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
            select_all: true,
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
