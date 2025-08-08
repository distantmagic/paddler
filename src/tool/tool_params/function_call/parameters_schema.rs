use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;

fn validate_schema(schema: &Value) -> Result<()> {
    // Try to create a validator - this validates the schema structure
    jsonschema::validator_for(schema).map_err(|err| anyhow!("{err}"))?;

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(try_from = "RawParametersSchema")]
pub struct ParametersSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<Value>,
}

#[derive(Deserialize)]
struct RawParametersSchema {
    #[serde(rename = "type")]
    schema_type: String,
    properties: Option<Map<String, Value>>,
    required: Option<Vec<String>>,
    #[serde(rename = "additionalProperties")]
    additional_properties: Option<Value>,
}

impl TryFrom<RawParametersSchema> for ParametersSchema {
    type Error = Error;

    fn try_from(raw: RawParametersSchema) -> Result<Self, Self::Error> {
        if let (Some(ref required), Some(ref properties)) = (&raw.required, &raw.properties) {
            for field in required {
                if !properties.contains_key(field) {
                    return Err(anyhow!("Required field '{field}' not found in properties"));
                }
            }
        }

        if let Some(ref properties) = raw.properties {
            for (key, schema) in properties {
                validate_schema(schema)
                    .map_err(|err| anyhow!("Invalid schema for property '{key}': {err}"))?;
            }
        }

        if let Some(ref additional) = raw.additional_properties {
            if !additional.is_boolean() {
                validate_schema(additional)
                    .map_err(|err| anyhow!("Invalid additionalProperties schema: {err}"))?;
            }
        }

        Ok(ParametersSchema {
            schema_type: raw.schema_type,
            properties: raw.properties,
            required: raw.required,
            additional_properties: raw.additional_properties,
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_serialize_minimal_schema() {
        let schema = ParametersSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: None,
        };

        let serialized = serde_json::to_value(&schema).unwrap();

        assert_eq!(serialized, json!({"type": "object"}));
    }

    #[test]
    fn test_serialize_full_schema() {
        let mut properties = Map::new();
        properties.insert("name".to_string(), json!({"type": "string"}));
        properties.insert("age".to_string(), json!({"type": "integer", "minimum": 0}));

        let schema = ParametersSchema {
            schema_type: "object".to_string(),
            properties: Some(properties),
            required: Some(vec!["name".to_string()]),
            additional_properties: Some(json!(false)),
        };

        let serialized = serde_json::to_value(&schema).unwrap();
        let expected = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer", "minimum": 0}
            },
            "required": ["name"],
            "additionalProperties": false
        });

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_deserialize_minimal_schema() {
        let input = json!({"type": "object"});
        let schema: ParametersSchema = serde_json::from_value(input).unwrap();

        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties.is_none());
        assert!(schema.required.is_none());
        assert!(schema.additional_properties.is_none());
    }

    #[test]
    fn test_deserialize_with_valid_properties() {
        let input = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer", "minimum": 0}
            },
            "required": ["name"],
            "additionalProperties": false
        });

        let schema: ParametersSchema = serde_json::from_value(input).unwrap();

        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties.is_some());
        assert_eq!(schema.properties.as_ref().unwrap().len(), 2);
        assert_eq!(schema.required, Some(vec!["name".to_string()]));
        assert_eq!(schema.additional_properties, Some(json!(false)));
    }

    #[test]
    fn test_deserialize_with_invalid_property_schema() {
        let input = json!({
            "type": "object",
            "properties": {
                "name": {"type": "invalid_type"}
            }
        });

        let result: Result<ParametersSchema, _> = serde_json::from_value(input);

        assert!(result.is_err());

        let error = result.unwrap_err().to_string();

        assert!(error.contains("Invalid schema for property 'name'"));
    }

    #[test]
    fn test_deserialize_with_complex_valid_schema() {
        let input = json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": {"type": "string"}
                },
                "metadata": {
                    "type": "object",
                    "properties": {
                        "created": {"type": "string", "format": "date-time"}
                    }
                }
            }
        });

        let schema: ParametersSchema = serde_json::from_value(input).unwrap();

        assert!(schema.properties.is_some());
        assert_eq!(schema.properties.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_deserialize_required_field_not_in_properties() {
        let input = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": ["name", "missing_field"]
        });

        let result: Result<ParametersSchema, _> = serde_json::from_value(input);

        assert!(result.is_err());

        let error = result.unwrap_err().to_string();

        assert!(error.contains("Required field 'missing_field' not found in properties"));
    }

    #[test]
    fn test_deserialize_additional_properties_as_schema() {
        let input = json!({
            "type": "object",
            "additionalProperties": {"type": "string"}
        });

        let schema: ParametersSchema = serde_json::from_value(input).unwrap();

        assert_eq!(
            schema.additional_properties,
            Some(json!({"type": "string"}))
        );
    }

    #[test]
    fn test_deserialize_additional_properties_as_boolean() {
        let input = json!({
            "type": "object",
            "additionalProperties": true
        });

        let schema: ParametersSchema = serde_json::from_value(input).unwrap();

        assert_eq!(schema.additional_properties, Some(json!(true)));
    }

    #[test]
    fn test_deserialize_invalid_additional_properties_schema() {
        let input = json!({
            "type": "object",
            "additionalProperties": {"type": "not_a_type"}
        });

        let result: Result<ParametersSchema, _> = serde_json::from_value(input);

        assert!(result.is_err());

        let error = result.unwrap_err().to_string();

        assert!(error.contains("Invalid additionalProperties schema"));
    }

    #[test]
    fn test_roundtrip_serialization() {
        let mut properties = Map::new();
        properties.insert(
            "count".to_string(),
            json!({"type": "integer", "minimum": 0, "maximum": 100}),
        );

        let original = ParametersSchema {
            schema_type: "object".to_string(),
            properties: Some(properties),
            required: Some(vec!["count".to_string()]),
            additional_properties: Some(json!({"type": "string"})),
        };

        let serialized = serde_json::to_value(&original).unwrap();
        let deserialized: ParametersSchema = serde_json::from_value(serialized).unwrap();

        assert_eq!(original.schema_type, deserialized.schema_type);
        assert_eq!(original.properties, deserialized.properties);
        assert_eq!(original.required, deserialized.required);
        assert_eq!(
            original.additional_properties,
            deserialized.additional_properties
        );
    }

    #[test]
    fn test_deserialize_with_enum_property() {
        let input = json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["active", "inactive", "pending"]
                }
            }
        });

        let schema: ParametersSchema = serde_json::from_value(input).unwrap();
        assert!(schema.properties.is_some());
        let props = schema.properties.as_ref().unwrap();
        assert!(props.contains_key("status"));
        assert!(props["status"]["enum"].is_array());
    }

    #[test]
    fn test_deserialize_nested_object_property() {
        let input = json!({
            "type": "object",
            "properties": {
                "address": {
                    "type": "object",
                    "properties": {
                        "street": {"type": "string"},
                        "city": {"type": "string"}
                    },
                    "required": ["city"]
                }
            }
        });

        let schema: ParametersSchema = serde_json::from_value(input).unwrap();

        assert!(schema.properties.is_some());
    }
}
