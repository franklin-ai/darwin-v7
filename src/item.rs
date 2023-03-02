use crate::workflow::Workflow;
use fake::{Dummy, Fake, Faker};
use serde::ser::SerializeMap;
use serde::{de::MapAccess, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{self, Display};

// Most fields in the following structures are listed as optional
// as depending upon the context the V7 endpoint may return a null
// value.  There is a high degree of variability as to when and why
// a null may be provided in the JSON payload.

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct ImageLevel {
    pub format: String,
    pub pixel_ratio: Option<u16>,
    pub tile_height: Option<u32>,
    pub tile_width: Option<u32>,
    pub x_tiles: Option<u32>,
    pub y_tiles: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Levels {
    pub image_levels: HashMap<String, ImageLevel>,
    pub base_key: String,
}

// JSON levels have a mix of ImageLevel information as well
// as the base_key.  A custom serializer / deserializer
// is required due to the mix of data types in the json structures
// e.g.
// {
//     "0": {
//         "format": "png",
//         "pixel_ratio": 1,
//         "tile_height": 2048,
//         "tile_width": 2048,
//         "x_tiles": 82,
//         "y_tiles": 22
//     },
//     "base_key": "some-base-key.jpg"
// }"

impl Serialize for Levels {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map_length = self.image_levels.len() + 1; // Levels + base_key
        let mut seq = serializer.serialize_map(Some(map_length))?;

        for (key, val) in self.image_levels.iter() {
            seq.serialize_entry(key, val)?;
        }

        seq.serialize_entry("base_key", &self.base_key)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Levels {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(LevelVisitor)
    }
}

struct LevelVisitor;

impl<'de> Visitor<'de> for LevelVisitor {
    type Value = Levels;

    #[cfg(not(tarpaulin_include))]
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a map with keys '0'..'N' and 'base_key'")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut image_levels: HashMap<String, ImageLevel> = HashMap::new();
        let mut base_key = String::new();

        while let Some(k) = map.next_key::<&str>()? {
            if k == "base_key" {
                base_key = map.next_value::<String>()?;
            } else {
                let level_key = match k.parse::<usize>() {
                    Ok(val) => val,
                    Err(_) => return Err(serde::de::Error::custom(&format!("Invalid key: {}", k))),
                };

                let level: ImageLevel = map.next_value()?;

                image_levels.insert(level_key.to_string(), level);
            }
        }

        Ok(Levels {
            image_levels,
            base_key,
        })
    }
}

impl Dummy<fake::Faker> for Levels {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &fake::Faker, rng: &mut R) -> Self {
        let max_levels: u32 = (2..5).fake_with_rng(rng);
        let base_key: String = Faker.fake_with_rng(rng);

        let mut image_levels = HashMap::new();
        for lvl in 1..max_levels {
            let img_level: ImageLevel = Faker.fake_with_rng(rng);
            let lvl = lvl - 1;
            image_levels.insert(lvl.to_string(), img_level);
        }

        Self {
            image_levels,
            base_key,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct Image {
    pub external: Option<bool>,
    pub format: Option<String>,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub id: u32,
    pub key: Option<String>,
    // Levels can either be a hashmap of levels, sometimes
    // there is a key "base_key" that has a string value
    // not a Level struct
    // TODO: manage this better
    pub levels: Option<Levels>,
    pub original_filename: Option<String>,
    pub thumbnail_url: Option<String>,
    pub uploaded: Option<bool>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct DatasetImage {
    pub dataset_id: u32,
    pub dataset_video_id: Option<u32>,
    pub id: u32,
    pub image: Image,
    pub seq: u32,
    pub set: u32,
}

// TODO: Define this struct
#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct DatasetVideo {}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatasetItemTypes {
    Image,
    Video,
}

impl Display for DatasetItemTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatasetItemTypes::Image => write!(f, "Image"),
            DatasetItemTypes::Video => write!(f, "Video"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatasetItemStatus {
    Annotate,
    Archived,
    Complete,
    Error,
    New,
    Processing,
    Review,
    Uploading,
}

impl Display for DatasetItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatasetItemStatus::Annotate => write!(f, "Annotate"),
            DatasetItemStatus::Archived => write!(f, "Archived"),
            DatasetItemStatus::Complete => write!(f, "Complete"),
            DatasetItemStatus::Error => write!(f, "Error"),
            DatasetItemStatus::New => write!(f, "New"),
            DatasetItemStatus::Processing => write!(f, "Processing"),
            DatasetItemStatus::Review => write!(f, "Review"),
            DatasetItemStatus::Uploading => write!(f, "Uploading"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct DatasetItem {
    pub archived: bool,
    pub archived_reason: Option<String>,
    pub current_workflow: Option<Workflow>,
    pub current_workflow_id: Option<u32>,
    pub dataset_id: Option<u32>,
    pub dataset_image: DatasetImage,
    pub dataset_image_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_video: Option<DatasetVideo>,
    pub dataset_video_id: Option<u32>,
    pub file_size: Option<u32>,
    pub filename: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub id: u32,
    pub inserted_at: Option<String>,
    pub updated_at: Option<String>,
    pub labels: Option<Vec<u32>>,
    pub path: Option<String>,
    pub priority: Option<u32>,
    pub seq: Option<u32>,
    pub set: Option<u32>,
    pub status: DatasetItemStatus,
    #[serde(rename = "type")]
    pub item_type: DatasetItemTypes, // This can probably be an enum
}

impl Display for DatasetItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{id-{}}}:{}/{}[{}]",
            self.id, self.filename, self.status, self.item_type
        )
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_levels_dummy() {
        let level: Levels = Faker.fake();
        assert!(level.image_levels.keys().len() >= 1);
    }

    #[test]
    fn test_image_level_ser_deser() {
        // Test serialization
        let contents = r#"
        {
            "0": {
                "format": "png",
                "pixel_ratio": 1,
                "tile_height": 2048,
                "tile_width": 2048,
                "x_tiles": 82,
                "y_tiles": 22
            },
            "base_key": "some-base-key.jpg"
        }"#;

        let image_level: Levels = serde_json::from_str(contents).unwrap();

        assert_eq!(image_level.base_key, "some-base-key.jpg".to_string());
        assert_eq!(
            image_level
                .image_levels
                .get(&"0".to_string())
                .unwrap()
                .format,
            "png".to_string()
        );

        // Test deserialization
        let image_level_deser = serde_json::to_string(&image_level).unwrap();

        assert_eq!(
            &image_level_deser,
            &contents.replace("\n", "").replace(' ', "")
        );
    }

    #[test]
    fn test_image_level_ser_deser_error() {
        // Test serialization
        let contents = r#"
        {
            "an-image": {
                "format": "png",
                "pixel_ratio": 1,
                "tile_height": 2048,
                "tile_width": 2048,
                "x_tiles": 82,
                "y_tiles": 22
            },
            "base_key": "some-base-key.jpg"
        }"#;
        serde_json::from_str::<Levels>(contents).expect_err("Invalid key: an-image");

        serde_json::from_str::<Levels>(r#"{"key": "value"}"#)
            .expect_err("a map with keys '0'..'N' and 'base_key'");
    }

    #[test]
    fn test_shortened_item() {
        let contents = r#"
    {
        "archived": false,
        "archived_reason": null,
        "current_workflow": {
            "current_stage_number": 4,
            "current_workflow_stage_template_id": 166369,
            "dataset_item_id": 650713507,
            "id": 43051890,
            "stages": {
                "1": [
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
                ]
            },
            "status": "complete",
            "workflow_template_id": 53975
        },
        "current_workflow_id": 43051890,
        "dataset_id": 587733,
        "dataset_image": {
            "dataset_id": 587733,
            "dataset_video_id": null,
            "id": 646799980,
            "image": {
                "external": true,
                "format": "tiled",
                "height": 44038,
                "id": 620657191,
                "key": "9841162f-3f4c-4434-be99-2c2d26c6856b",
                "levels": {
                    "0": {
                        "format": "png",
                        "pixel_ratio": 1,
                        "tile_height": 2048,
                        "tile_width": 2048,
                        "x_tiles": 82,
                        "y_tiles": 22
                    },
                    "base_key": "some-base-key.jpg"
                },
                "original_filename": "9841162f-3f4c-4434-be99-2c2d26c6856b",
                "thumbnail_url": "https://great-thumbnail.thing.foo",
                "uploaded": true,
                "url": "https://url-to.thing.foo",
                "width": 166918
            },
            "seq": 1,
            "set": 1669945183
        },
        "dataset_image_id": 646799980,
        "dataset_video": null,
        "dataset_video_id": null,
        "file_size": 0,
        "filename": "9841162f-3f4c-4434-be99-2c2d26c6856b",
        "height": 44038,
        "id": 650713507,
        "inserted_at": "2022-12-02T01:39:43",
        "labels": [
            189358,
            189395,
            189403
        ],
        "path": "/",
        "priority": 0,
        "seq": 1,
        "set": 1669945183,
        "status": "complete",
        "type": "image",
        "updated_at": "2022-12-14T00:28:33",
        "width": 166918
    }
        "#;

        let ser_item: DatasetItem = serde_json::from_str(contents).unwrap();

        assert_eq!(ser_item.status, DatasetItemStatus::Complete);
        assert_eq!(ser_item.dataset_image_id, Some(646799980));
        assert_eq!(ser_item.labels.unwrap().len(), 3);
        assert!(ser_item
            .current_workflow
            .unwrap()
            .stages
            .keys()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .contains(&"1".to_string()));

        let level_0 = ser_item.dataset_image.image.levels.unwrap();
        assert_eq!(
            level_0.image_levels.get("0").unwrap().format,
            "png".to_string()
        );
        assert_eq!(level_0.base_key, "some-base-key.jpg".to_string());
    }
}
