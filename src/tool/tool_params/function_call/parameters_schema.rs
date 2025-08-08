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
