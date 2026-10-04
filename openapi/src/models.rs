#![allow(unused_qualifications)]

use http::HeaderValue;
use validator::Validate;

#[cfg(feature = "server")]
use crate::header;
use crate::{models, types::*};

#[allow(dead_code)]
pub fn check_xss_string(v: &str) -> std::result::Result<(), validator::ValidationError> {
    if ammonia::is_html(v) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_vec_string(v: &[String]) -> std::result::Result<(), validator::ValidationError> {
    if v.iter().any(|i| ammonia::is_html(i)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map_string(
    v: &std::collections::HashMap<String, String>,
) -> std::result::Result<(), validator::ValidationError> {
    if v.keys().any(|k| ammonia::is_html(k)) || v.values().any(|v| ammonia::is_html(v)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map_nested<T>(
    v: &std::collections::HashMap<String, T>,
) -> std::result::Result<(), validator::ValidationError>
where
    T: validator::Validate,
{
    if v.keys().any(|k| ammonia::is_html(k)) || v.values().any(|v| v.validate().is_err()) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map<T>(v: &std::collections::HashMap<String, T>) -> std::result::Result<(), validator::ValidationError> {
    if v.keys().any(|k| ammonia::is_html(k)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}




    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
    #[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
    pub struct StopQueryParams {
            /// The caller's revision. An instance doesn't stop on behalf of its own revision.
                #[serde(rename = "revision")]
                    pub revision: String,
    }




#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ErrorResponse {
    /// HTTP status code.
    #[serde(rename = "statusCode")]
    pub status_code: i32,

    /// Human-readable error message.
    #[serde(rename = "message")]
          #[validate(custom(function = "check_xss_string"))]
    pub message: String,

    /// PostgreSQL SQLSTATE, when the error comes from PostgreSQL (e.g. 23505 for unique_violation, 23503 for foreign_key_violation). 
    #[serde(rename = "code")]
          #[validate(custom(function = "check_xss_string"))]
    #[serde(skip_serializing_if="Option::is_none")]
    pub code: Option<String>,

}



impl ErrorResponse {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(status_code: i32, message: String, ) -> ErrorResponse {
        ErrorResponse {
 status_code,
 message,
 code: None,
        }
    }
}

/// Converts the ErrorResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("statusCode".to_string()),
            Some(self.status_code.to_string()),


            Some("message".to_string()),
            Some(self.message.to_string()),


            self.code.as_ref().map(|code| {
                [
                    "code".to_string(),
                    code.to_string(),
                ].join(",")
            }),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ErrorResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ErrorResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub status_code: Vec<i32>,
            pub message: Vec<String>,
            pub code: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ErrorResponse".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "statusCode" => intermediate_rep.status_code.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "message" => intermediate_rep.message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "code" => intermediate_rep.code.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ErrorResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ErrorResponse {
            status_code: intermediate_rep.status_code.into_iter().next().ok_or_else(|| "statusCode missing in ErrorResponse".to_string())?,
            message: intermediate_rep.message.into_iter().next().ok_or_else(|| "message missing in ErrorResponse".to_string())?,
            code: intermediate_rep.code.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ErrorResponse> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<ErrorResponse>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ErrorResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for ErrorResponse - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<ErrorResponse> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ErrorResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into ErrorResponse - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Result {
    #[serde(rename = "columns")]
          #[validate(custom(function = "check_xss_vec_string"))]
    pub columns: Vec<String>,

    /// PostgreSQL type of each column (e.g. `int4`, `timestamptz`)
    #[serde(rename = "types")]
          #[validate(custom(function = "check_xss_vec_string"))]
    pub types: Vec<String>,

    #[serde(rename = "rows")]
    pub rows: Vec<Vec<crate::types::Object>>,

    /// Row count of the command, e.g. rows inserted, updated, deleted or selected.
    #[serde(rename = "changes")]
    pub changes: i64,

}



impl Result {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(columns: Vec<String>, types: Vec<String>, rows: Vec<Vec<crate::types::Object>>, changes: i64, ) -> Result {
        Result {
 columns,
 types,
 rows,
 changes,
        }
    }
}

/// Converts the Result value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("columns".to_string()),
            Some(self.columns.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")),


            Some("types".to_string()),
            Some(self.types.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")),

            // Skipping rows in query parameter serialization


            Some("changes".to_string()),
            Some(self.changes.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Result value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Result {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub columns: Vec<Vec<String>>,
            pub types: Vec<Vec<String>>,
            pub rows: Vec<Vec<Vec<crate::types::Object>>>,
            pub changes: Vec<i64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Result".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "columns" => return std::result::Result::Err("Parsing a container in this style is not supported in Result".to_string()),
                    "types" => return std::result::Result::Err("Parsing a container in this style is not supported in Result".to_string()),
                    "rows" => return std::result::Result::Err("Parsing a container in this style is not supported in Result".to_string()),
                    #[allow(clippy::redundant_clone)]
                    "changes" => intermediate_rep.changes.push(<i64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Result".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Result {
            columns: intermediate_rep.columns.into_iter().next().ok_or_else(|| "columns missing in Result".to_string())?,
            types: intermediate_rep.types.into_iter().next().ok_or_else(|| "types missing in Result".to_string())?,
            rows: intermediate_rep.rows.into_iter().next().ok_or_else(|| "rows missing in Result".to_string())?,
            changes: intermediate_rep.changes.into_iter().next().ok_or_else(|| "changes missing in Result".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Result> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Result>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Result>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Result - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Result> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Result as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Result - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Statement {
    /// A single SQL statement with `$1`, `$2`, ... placeholders.
    #[serde(rename = "sql")]
          #[validate(custom(function = "check_xss_string"))]
    pub sql: String,

    /// Values bound to the placeholders in order. They are sent as text and PostgreSQL converts them to the placeholder's type; arrays and objects are sent as JSON. 
    #[serde(rename = "params")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub params: Option<Vec<crate::types::Object>>,

}



impl Statement {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(sql: String, ) -> Statement {
        Statement {
 sql,
 params: None,
        }
    }
}

/// Converts the Statement value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("sql".to_string()),
            Some(self.sql.to_string()),

            // Skipping params in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Statement value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Statement {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub sql: Vec<String>,
            pub params: Vec<Vec<crate::types::Object>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Statement".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "sql" => intermediate_rep.sql.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "params" => return std::result::Result::Err("Parsing a container in this style is not supported in Statement".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing Statement".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Statement {
            sql: intermediate_rep.sql.into_iter().next().ok_or_else(|| "sql missing in Statement".to_string())?,
            params: intermediate_rep.params.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Statement> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Statement>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Statement>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Statement - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Statement> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Statement as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Statement - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}


