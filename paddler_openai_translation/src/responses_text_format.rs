use std::fmt;
use std::fmt::Formatter;

use serde::Deserialize;
use serde::Deserializer;
use serde::de::Error as _;
use serde::de::IgnoredAny;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde_json::value::RawValue;

use crate::responses_text_format_field::ResponsesTextFormatField;
use crate::responses_text_format_type::ResponsesTextFormatType;

struct ResponsesTextFormatVisitor;

impl<'de> Visitor<'de> for ResponsesTextFormatVisitor {
    type Value = ResponsesTextFormat;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("internally tagged enum ResponsesTextFormat")
    }

    fn visit_map<TMapAccess>(
        self,
        mut format_fields: TMapAccess,
    ) -> Result<Self::Value, TMapAccess::Error>
    where
        TMapAccess: MapAccess<'de>,
    {
        let mut format_type = None;
        let mut schema = None;

        while let Some(format_field) = format_fields.next_key()? {
            match format_field {
                ResponsesTextFormatField::Schema => {
                    if schema
                        .replace(format_fields.next_value::<Box<RawValue>>()?)
                        .is_some()
                    {
                        return Err(TMapAccess::Error::duplicate_field("schema"));
                    }
                }
                ResponsesTextFormatField::Type => {
                    if format_type
                        .replace(format_fields.next_value::<ResponsesTextFormatType>()?)
                        .is_some()
                    {
                        return Err(TMapAccess::Error::duplicate_field("type"));
                    }
                }
                ResponsesTextFormatField::Unrecognized => {
                    format_fields.next_value::<IgnoredAny>()?;
                }
            }
        }

        match format_type {
            None => Err(TMapAccess::Error::missing_field("type")),
            Some(ResponsesTextFormatType::Text) => Ok(ResponsesTextFormat::Text),
            Some(ResponsesTextFormatType::JsonSchema) => schema
                .map(|schema| ResponsesTextFormat::JsonSchema { schema })
                .ok_or_else(|| TMapAccess::Error::missing_field("schema")),
        }
    }
}

pub enum ResponsesTextFormat {
    Text,
    JsonSchema { schema: Box<RawValue> },
}

impl<'de> Deserialize<'de> for ResponsesTextFormat {
    fn deserialize<TDeserializer>(deserializer: TDeserializer) -> Result<Self, TDeserializer::Error>
    where
        TDeserializer: Deserializer<'de>,
    {
        deserializer.deserialize_map(ResponsesTextFormatVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_str;

    use super::ResponsesTextFormat;

    fn refusal(format: &str) -> Option<String> {
        from_str::<ResponsesTextFormat>(format)
            .err()
            .map(|deserialization_error| deserialization_error.to_string())
    }

    #[test]
    fn keeps_the_schema_text_exactly_as_sent() {
        let schema = r#"{ "type": "integer", "maximum": 1e3 }"#;

        assert!(matches!(
            from_str::<ResponsesTextFormat>(&format!(
                r#"{{"type": "json_schema", "name": "out", "schema": {schema}}}"#
            )),
            Ok(ResponsesTextFormat::JsonSchema { schema: raw_schema }) if raw_schema.get() == schema
        ));
    }

    #[test]
    fn refuses_a_format_that_is_not_an_object() {
        assert_eq!(
            refusal(r#""json_schema""#),
            Some(
                r#"invalid type: string "json_schema", expected internally tagged enum ResponsesTextFormat at line 1 column 13"#
                    .to_owned()
            )
        );
    }

    #[test]
    fn refuses_a_format_without_a_type() {
        assert_eq!(
            refusal(r#"{"schema": {}}"#),
            Some("missing field `type` at line 1 column 14".to_owned())
        );
    }

    #[test]
    fn refuses_a_json_schema_format_without_a_schema() {
        assert_eq!(
            refusal(r#"{"type": "json_schema"}"#),
            Some("missing field `schema` at line 1 column 23".to_owned())
        );
    }

    #[test]
    fn refuses_a_format_whose_fields_are_malformed() {
        assert_eq!(
            [
                refusal(r#"{"type": "text", 1: 2}"#),
                refusal(r#"{"type": "json_schema", "schema": nul}"#),
                refusal(r#"{"type": "text", "name": nul}"#),
            ],
            [
                Some("key must be a string at line 1 column 18".to_owned()),
                Some("expected ident at line 1 column 38".to_owned()),
                Some("expected ident at line 1 column 29".to_owned()),
            ]
        );
    }

    #[test]
    fn refuses_a_format_that_repeats_a_field() {
        assert_eq!(
            [
                refusal(r#"{"type": "text", "type": "text"}"#),
                refusal(r#"{"type": "json_schema", "schema": {}, "schema": {}}"#),
            ],
            [
                Some("duplicate field `type` at line 1 column 32".to_owned()),
                Some("duplicate field `schema` at line 1 column 51".to_owned()),
            ]
        );
    }
}
