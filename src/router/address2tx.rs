use crate::client::SolanaClient;
use actix_web::web;
use actix_web::HttpResponse;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Address2txFormData {
    pub address: String,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct NewTx {
    pub tx_signature: String,
    pub address: String,
    pub sequence_number: i64,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, solana_client),
    fields(
        subscriber_email = %form.address,
    )
)]
// Let's start simple: we always return a 200 OK
pub async fn subscrib_address(
    form: web::Form<Address2txFormData>,
    // Retrieving a connection from the application state!
    pool: web::Data<PgPool>,
    solana_client: web::Data<SolanaClient>,
) -> HttpResponse {
    // query address2tx table address if not exist set sequence_number is 0
    // or get the max sequence_number and increment it

    let Address2txFormData { address } = form.into_inner();

    // Retrieve the current maximum sequence number for the provided address
    let result = sqlx::query!(
        r#"
           SELECT MAX(sequence_number) as max_sequence FROM address_tx WHERE address = $1
           "#,
        address
    )
    .fetch_optional(pool.get_ref())
    .await;

    let current_sequence = match result {
        Ok(Some(record)) => record.max_sequence.unwrap_or(0) + 1,
        Ok(None) => 1, // If no records are found, start with sequence number 1
        Err(e) => {
            tracing::error!("Failed to fetch max sequence number: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let new_txs = tokio::task::spawn_blocking(move || {
        let new_txs = get_address_tx_signature(&address, solana_client.get_ref())?
            .into_iter()
            .into_iter()
            .enumerate()
            .map(|(idx, tx_signature)| NewTx {
                tx_signature,
                address: address.clone(),
                sequence_number: current_sequence + idx as i64,
            })
            .collect::<Vec<_>>();
        Ok::<Vec<_>, anyhow::Error>(new_txs)
    })
    .await;

    let new_txs = match new_txs {
        Ok(Ok(new_txs)) => new_txs,
        Ok(Err(e)) => {
            tracing::error!("Failed to fetch new transactions: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
        Err(e) => {
            tracing::error!("Failed to spawn blocking task: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    for tx in new_txs {
        match insert_address2tx(&pool, &tx).await {
            Ok(_) => (),
            Err(e) => {
                // Yes, this error log falls outside of `query_span`
                // We'll rectify it later, pinky swear!
                tracing::error!("Failed to execute query: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        }
    }

    HttpResponse::Ok().finish()
}

pub fn get_address_tx_signature(
    address: &str,
    solana_client: &SolanaClient,
) -> anyhow::Result<Vec<String>> {
    let mut sigs = solana_client.get_tx_signature_by_address_before(address, None)?;
    sigs.reverse();
    Ok(sigs)
}

#[tracing::instrument(
    name = "Saving new address2tx details in the database",
    skip(new_tx, pool)
)]
pub async fn insert_address2tx(pool: &PgPool, new_tx: &NewTx) -> anyhow::Result<()> {
    // 执行插入操作
    sqlx::query!(
        r#"
            INSERT INTO address_tx (id, tx_signature, address, sequence_number)
            VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        new_tx.tx_signature,
        new_tx.address,
        new_tx.sequence_number
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(name = "Fetching all transactions for an address", skip(form, pool))]
pub async fn txs(form: web::Query<Address2txFormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let result = sqlx::query_as!(
        NewTx,
        "SELECT tx_signature, address, sequence_number FROM address_tx WHERE address = $1",
        form.address
    )
    .fetch_all(pool.as_ref())
    .await;

    match result {
        Ok(items) => HttpResponse::Ok().json(
            items
                .into_iter()
                .map(|item| item.tx_signature)
                .collect::<Vec<_>>(),
        ),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
