use deadpool_postgres::Pool;
use openapi::{
    apis::default::ExecuteSqlResponse,
    models::{ErrorResponse, Statement},
};

use crate::db;

// POST /sql
pub async fn execute_sql(pool: Option<&Pool>, statements: Vec<Statement>) -> ExecuteSqlResponse {
    let Some(pool) = pool else {
        return ExecuteSqlResponse::Status503_NoDatabaseConnectionAvailable(ErrorResponse::new(
            503,
            "the database was handed off to a new revision".into(),
        ));
    };
    match db::execute(pool, statements).await {
        Ok(Ok(result)) => ExecuteSqlResponse::Status200_StatementsExecuted(result),
        Ok(Err(e)) => ExecuteSqlResponse::Status400_SQLError(ErrorResponse {
            code: e.code,
            ..ErrorResponse::new(400, e.message)
        }),
        Err(e) => ExecuteSqlResponse::Status503_NoDatabaseConnectionAvailable(ErrorResponse::new(
            503,
            e.to_string(),
        )),
    }
}
