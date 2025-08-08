use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;

use super::validated_parameters_schema::ValidatedParametersSchema;

fn validate_schema(schema: &Value) -> Result<()> {
    // Try to create a validator - this validates the schema structure
    jsonschema::validator_for(schema).map_err(|err| anyhow!("{err}"))?;

    Ok(())
}

#[derive(Default, Deserialize, Serialize)]
pub struct RawParametersSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: Option<Map<String, Value>>,
    pub required: Option<Vec<String>>,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<Value>,
}

impl TryFrom<RawParametersSchema> for ValidatedParametersSchema {
    type Error = Error;

    fn try_from(raw: RawParametersSchema) -> Result<Self, Self::Error> {
        if let (Some(required), Some(properties)) = (&raw.required, &raw.properties) {
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

        if let Some(ref additional) = raw.additional_properties
            && !additional.is_boolean()
        {
            validate_schema(additional)
                .map_err(|err| anyhow!("Invalid additionalProperties schema: {err}"))?;
        }

        Ok(ValidatedParametersSchema {
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

        let raw_schema: RawParametersSchema = serde_json::from_value(input).unwrap();
        let schema: ValidatedParametersSchema = raw_schema.try_into().unwrap();

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

        let raw_schema: RawParametersSchema = serde_json::from_value(input).unwrap();
        let result: Result<ValidatedParametersSchema, _> = raw_schema.try_into();

        assert!(result.is_err());

        let error = result.unwrap_err().to_string();

        assert!(error.contains("Invalid schema for property 'name'"));
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

        let raw_schema: RawParametersSchema = serde_json::from_value(input).unwrap();
        let result: Result<ValidatedParametersSchema, _> = raw_schema.try_into();

        assert!(result.is_err());

        let error = result.unwrap_err().to_string();

        assert!(error.contains("Required field 'missing_field' not found in properties"));
    }

    #[test]
    fn test_deserialize_invalid_additional_properties_schema() {
        let input = json!({
            "type": "object",
            "additionalProperties": {"type": "not_a_type"}
        });

        let raw_schema: RawParametersSchema = serde_json::from_value(input).unwrap();
        let result: Result<ValidatedParametersSchema, _> = raw_schema.try_into();

        assert!(result.is_err());

        let error = result.unwrap_err().to_string();

        assert!(error.contains("Invalid additionalProperties schema"));
    }
}
