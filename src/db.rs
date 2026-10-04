use std::error::Error;

use bytes::BytesMut;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use deadpool_postgres::{GenericClient, Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use futures_util::TryStreamExt;
use openapi::{
    models::{Result as SqlResult, Statement},
    types::Object as JsonObject,
};
use serde_json::Value;
use tokio_postgres::{
    types::{to_sql_checked, Format, FromSql, IsNull, Kind, ToSql, Type},
    NoTls,
};
use uuid::Uuid;

pub use deadpool_postgres::PoolError;

const POOL_SIZE: usize = 10;

// Requests run as `app`, which owns the `app` database but isn't a superuser.
pub async fn open() -> Pool {
    let mut config = tokio_postgres::Config::new();
    config
        .host("/tmp")
        .user("app")
        .dbname("app")
        .options("-c statement_timeout=5s");
    let manager = Manager::from_config(
        config,
        NoTls,
        // Resets session state (settings, temporary tables, ...) between requests.
        ManagerConfig {
            recycling_method: RecyclingMethod::Clean,
        },
    );
    let pool = Pool::builder(manager).max_size(POOL_SIZE).build().unwrap();
    drop(pool.get().await.expect("open database"));
    pool
}

pub struct SqlError {
    pub message: String,
    // SQLSTATE, e.g. 23505 for unique_violation.
    pub code: Option<String>,
}

impl From<tokio_postgres::Error> for SqlError {
    fn from(e: tokio_postgres::Error) -> Self {
        match e.as_db_error() {
            Some(db) => SqlError {
                message: db.to_string(),
                code: Some(db.code().code().into()),
            },
            None => SqlError {
                // e.g. "error deserializing column 0: unsupported column type ..."
                message: match e.source() {
                    Some(source) => format!("{e}: {source}"),
                    None => e.to_string(),
                },
                code: None,
            },
        }
    }
}

pub async fn execute(
    pool: &Pool,
    statements: Vec<Statement>,
) -> Result<Result<Vec<SqlResult>, SqlError>, PoolError> {
    let client = pool.get().await?;
    // Run to the end even if the request is dropped, so the transaction is always finished.
    let task = tokio::spawn(async move { run_all(client, &statements).await });
    Ok(task.await.unwrap())
}

// Two or more statements run in one transaction.
async fn run_all(mut client: Object, statements: &[Statement]) -> Result<Vec<SqlResult>, SqlError> {
    if let Some(statement) = statements.iter().find(|s| is_transaction_control(&s.sql)) {
        return Err(SqlError {
            message: format!("transaction control is not allowed: {}", statement.sql),
            code: None,
        });
    }
    if let [statement] = statements {
        return Ok(vec![run(&client, statement).await?]);
    }
    let transaction = client.transaction().await?;
    let mut results = Vec::with_capacity(statements.len());
    for statement in statements {
        results.push(run(&transaction, statement).await?);
    }
    transaction.commit().await?;
    Ok(results)
}

// BEGIN, COMMIT, ROLLBACK and the like would end the request's transaction early or leave one open
// on a pooled connection. Prepared statements hold a single statement, so its first keyword decides.
fn is_transaction_control(sql: &str) -> bool {
    let mut sql = sql.trim_start();
    loop {
        if let Some(rest) = sql.strip_prefix("--") {
            sql = rest.split_once('\n').map_or("", |(_, rest)| rest);
        } else if sql.starts_with("/*") {
            // Block comments nest.
            let (bytes, mut depth, mut i) = (sql.as_bytes(), 0, 0);
            while i < bytes.len() {
                match &bytes[i..] {
                    [b'/', b'*', ..] => (depth, i) = (depth + 1, i + 2),
                    [b'*', b'/', ..] => (depth, i) = (depth - 1, i + 2),
                    _ => i += 1,
                }
                if depth == 0 {
                    break;
                }
            }
            sql = &sql[i..];
        } else {
            break;
        }
        sql = sql.trim_start();
    }
    let keyword = sql
        .split(|c: char| !c.is_ascii_alphabetic())
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    matches!(
        keyword.as_str(),
        "BEGIN" | "START" | "COMMIT" | "END" | "ROLLBACK" | "ABORT"
    )
}

async fn run(client: &impl GenericClient, statement: &Statement) -> Result<SqlResult, SqlError> {
    let prepared = client.prepare(&statement.sql).await?;
    let columns = prepared.columns();
    let names = columns.iter().map(|c| c.name().to_string()).collect();
    let types = columns
        .iter()
        .map(|c| c.type_().name().to_string())
        .collect();
    let params: Vec<Param> = statement.params.iter().flatten().map(to_param).collect();

    let stream = client.query_raw(&prepared, params).await?;
    tokio::pin!(stream);
    let mut rows = Vec::new();
    while let Some(row) = stream.try_next().await? {
        let values = (0..row.len())
            .map(|i| row.try_get::<_, Json>(i).map(|json| json.0))
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(values);
    }
    let changes = stream.rows_affected().unwrap_or_default();
    Ok(SqlResult::new(names, types, rows, changes as i64))
}

// Parameters are sent as text, so PostgreSQL converts them to whatever type the placeholder has.
#[derive(Debug)]
struct Param(Option<String>);

impl ToSql for Param {
    fn to_sql(&self, _: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        match &self.0 {
            Some(text) => {
                out.extend_from_slice(text.as_bytes());
                Ok(IsNull::No)
            }
            None => Ok(IsNull::Yes),
        }
    }

    fn accepts(_: &Type) -> bool {
        true
    }

    fn encode_format(&self, _: &Type) -> Format {
        Format::Text
    }

    to_sql_checked!();
}

fn to_param(param: &JsonObject) -> Param {
    Param(match serde_json::to_value(param).unwrap() {
        Value::Null => None,
        Value::String(s) => Some(s),
        // Booleans and numbers have the same text form in PostgreSQL; arrays and objects are JSON.
        value => Some(value.to_string()),
    })
}

// Results come in PostgreSQL's binary format; this decodes the common types to JSON.
struct Json(JsonObject);

impl<'a> FromSql<'a> for Json {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let value: Value = match *ty {
            Type::BOOL => bool::from_sql(ty, raw)?.into(),
            Type::INT2 => i16::from_sql(ty, raw)?.into(),
            Type::INT4 => i32::from_sql(ty, raw)?.into(),
            Type::INT8 => i64::from_sql(ty, raw)?.into(),
            Type::OID => u32::from_sql(ty, raw)?.into(),
            Type::FLOAT4 => match f32::from_sql(ty, raw)? {
                f if f.is_finite() => f.into(),
                f => non_finite(f.into()),
            },
            Type::FLOAT8 => match f64::from_sql(ty, raw)? {
                f if f.is_finite() => f.into(),
                f => non_finite(f),
            },
            Type::NUMERIC => numeric(raw).into(),
            Type::JSON | Type::JSONB => Value::from_sql(ty, raw)?,
            // Lowercase hex, like SQLite's hex().
            Type::BYTEA => raw
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .into(),
            Type::DATE => NaiveDate::from_sql(ty, raw)?.to_string().into(),
            Type::TIME => NaiveTime::from_sql(ty, raw)?.to_string().into(),
            Type::TIMESTAMP => NaiveDateTime::from_sql(ty, raw)?.to_string().into(),
            Type::TIMESTAMPTZ => DateTime::<Utc>::from_sql(ty, raw)?.to_rfc3339().into(),
            Type::UUID => Uuid::from_sql(ty, raw)?.to_string().into(),
            // Text types and enums are sent as text.
            _ if <&str as FromSql>::accepts(ty) || matches!(ty.kind(), Kind::Enum(_)) => {
                <&str>::from_sql(ty, raw)?.into()
            }
            _ => return Err(format!("unsupported column type {ty}; cast it to text").into()),
        };
        Ok(Json(serde_json::from_value(value).unwrap()))
    }

    fn from_sql_null(_: &Type) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Ok(Json(serde_json::from_value(Value::Null).unwrap()))
    }

    fn accepts(_: &Type) -> bool {
        true
    }
}

// JSON has no NaN or infinities (serde_json would turn them into null), so they're returned in
// PostgreSQL's text form, like NUMERIC's.
fn non_finite(f: f64) -> Value {
    match f {
        f if f.is_nan() => "NaN",
        f if f > 0.0 => "Infinity",
        _ => "-Infinity",
    }
    .into()
}

// NUMERIC is sent as base-10000 digits with a weight (the power of the first digit), a sign and a
// display scale. It's returned as a string, like PostgreSQL's text output, to keep its precision.
fn numeric(raw: &[u8]) -> String {
    let word = |i: usize| i16::from_be_bytes([raw[2 * i], raw[2 * i + 1]]);
    let (ndigits, weight, sign, dscale) = (word(0) as i32, word(1) as i32, word(2) as u16, word(3));
    match sign {
        0xC000 => return "NaN".into(),
        0xD000 => return "Infinity".into(),
        0xF000 => return "-Infinity".into(),
        _ => {}
    }
    // The i-th digit is worth 10000^(weight - i).
    let digit = |i: i32| match (0..ndigits).contains(&i) {
        true => word(4 + i as usize),
        false => 0,
    };
    let mut text = String::from(if sign == 0x4000 { "-" } else { "" });
    if weight < 0 {
        text.push('0');
    } else {
        text += &digit(0).to_string();
        for i in 1..=weight {
            text += &format!("{:04}", digit(i));
        }
    }
    if dscale > 0 {
        let groups = (dscale as usize).div_ceil(4);
        let fraction: String = (weight + 1..)
            .take(groups)
            .map(|i| format!("{:04}", digit(i)))
            .collect();
        text = format!("{text}.{}", &fraction[..dscale as usize]);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_control() {
        for sql in [
            "BEGIN",
            "begin;",
            "  Commit",
            "START TRANSACTION",
            "END",
            "ROLLBACK TO SAVEPOINT s",
            "abort",
            "-- comment\nCOMMIT",
            "/* a /* nested */ comment */ COMMIT",
        ] {
            assert!(is_transaction_control(sql), "{sql}");
        }
        for sql in [
            "SELECT 1",
            "SAVEPOINT s",
            "INSERT INTO t VALUES ('COMMIT')",
            "/* COMMIT */ SELECT 1",
            "-- COMMIT\nSELECT 1",
            "BEGINNING",
        ] {
            assert!(!is_transaction_control(sql), "{sql}");
        }
    }

    #[test]
    fn non_finite_floats() {
        let json = |ty: &Type, raw: &[u8]| serde_json::to_value(Json::from_sql(ty, raw).unwrap().0);
        assert_eq!(json(&Type::FLOAT8, &f64::NAN.to_be_bytes()).unwrap(), "NaN");
        assert_eq!(
            json(&Type::FLOAT8, &f64::INFINITY.to_be_bytes()).unwrap(),
            "Infinity"
        );
        assert_eq!(
            json(&Type::FLOAT4, &f32::NEG_INFINITY.to_be_bytes()).unwrap(),
            "-Infinity"
        );
        assert_eq!(json(&Type::FLOAT8, &1.5f64.to_be_bytes()).unwrap(), 1.5);
    }

    #[test]
    fn numeric_text() {
        let encode = |words: &[i16]| {
            words
                .iter()
                .flat_map(|w| w.to_be_bytes())
                .collect::<Vec<_>>()
        };
        // 123.45
        assert_eq!(numeric(&encode(&[2, 0, 0, 2, 123, 4500])), "123.45");
        // -10000
        assert_eq!(numeric(&encode(&[1, 1, 0x4000, 0, 1])), "-10000");
        // 0.00000012
        assert_eq!(numeric(&encode(&[1, -2, 0, 8, 12])), "0.00000012");
        // 0
        assert_eq!(numeric(&encode(&[0, 0, 0, 0])), "0");
        // 12345678.9
        assert_eq!(
            numeric(&encode(&[3, 1, 0, 1, 1234, 5678, 9000])),
            "12345678.9"
        );
    }
}
