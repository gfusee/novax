use serde::{Deserialize, Serializer, Deserializer, ser::SerializeStruct};
use novax::CodeMetadata;

pub fn code_metadata_serialize<S>(value: &Option<CodeMetadata>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    match value {
        Some(code_metadata) => {
            let mut state = serializer.serialize_struct("CodeMetadata", 1)?;
            state.serialize_field("bits", &code_metadata.bits())?;
            state.end()
        }
        None => serializer.serialize_none(),
    }
}

pub fn code_metadata_deserialize<'de, D>(deserializer: D) -> Result<Option<CodeMetadata>, D::Error>
where
    D: Deserializer<'de>
{
    #[derive(Deserialize)]
    struct CodeMetadataHelper {
        bits: u16,
    }

    let opt: Option<CodeMetadataHelper> = Option::deserialize(deserializer)?;
    Ok(opt.map(|helper| CodeMetadata::from(helper.bits)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde_json;

    #[derive(Serialize, Deserialize)]
    struct CodeMetadataWrapper {
        #[serde(serialize_with = "code_metadata_serialize")]
        #[serde(deserialize_with = "code_metadata_deserialize")]
        #[serde(rename = "codeMetadata")]
        code_metadata: Option<CodeMetadata>,
    }

    #[test]
    fn test_serialize_some_code_metadata() {
        let wrapper = CodeMetadataWrapper {
            code_metadata: Some(CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE),
        };
        let serialized = serde_json::to_string(&wrapper).expect("Serialization failed");
        assert_eq!(serialized, "{\"codeMetadata\":{\"bits\":1280}}");
    }

    #[test]
    fn test_serialize_none_code_metadata() {
        let wrapper = CodeMetadataWrapper { code_metadata: None };
        let serialized = serde_json::to_string(&wrapper).expect("Serialization failed");
        assert_eq!(serialized, "{\"codeMetadata\":null}");
    }

    #[test]
    fn test_deserialize_some_code_metadata() {
        let data = "{\"codeMetadata\":{\"bits\":1280}}";
        let deserialized: CodeMetadataWrapper = serde_json::from_str(data).expect("Deserialization failed");
        assert_eq!(deserialized.code_metadata, Some(CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE));
    }

    #[test]
    fn test_deserialize_none_code_metadata() {
        let data = "{\"codeMetadata\":null}";
        let deserialized: CodeMetadataWrapper = serde_json::from_str(data).expect("Deserialization failed");
        assert_eq!(deserialized.code_metadata, None);
    }
}