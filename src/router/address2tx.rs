use actix_web::web;
use actix_web::HttpResponse;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Address2txFormData {
    pub address: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct NewTx {
    pub tx_signature: String,
    pub address: String,
    pub sequence_number: u64,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.address,
    )
)]
// Let's start simple: we always return a 200 OK
pub async fn subscrib_address(
    form: web::Form<Address2txFormData>,
    // Retrieving a connection from the application state!
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // TODO
    let new_subscriber = NewTx {
        tx_signature: "123".to_string(),
        address: form.address.clone(),
        sequence_number: 1,
    };
    match insert_address2tx(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            // Yes, this error log falls outside of `query_span`
            // We'll rectify it later, pinky swear!
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_address_tx_signature(address: &str) -> anyhow::Result<Vec<String>> {
    todo!("get_address_tx_signature")
}

#[tracing::instrument(
    name = "Saving new address2tx details in the database",
    skip(new_tx, pool)
)]
pub async fn insert_address2tx(pool: &PgPool, new_tx: &NewTx) -> anyhow::Result<()> {
    // 首先查询给定地址的最新 sequence_number
    let max_sequence: Option<i64> = sqlx::query!(
        r#"
         SELECT MAX(sequence_number) AS max_sequence FROM address_tx WHERE address = $1
         "#,
        new_tx.address
    )
    .fetch_one(pool)
    .await?
    .max_sequence;

    let new_sequence_number = max_sequence.unwrap_or(0) + 1;

    // 执行插入操作
    sqlx::query!(
        r#"
            INSERT INTO address_tx (id, tx_signature, address, sequence_number)
            VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        new_tx.tx_signature,
        new_tx.address,
        new_sequence_number
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
