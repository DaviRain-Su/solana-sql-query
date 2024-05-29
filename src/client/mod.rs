use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use std::str::FromStr;

pub struct SolanaClient {
    pub client: RpcClient,
}

impl SolanaClient {
    pub fn new(url: &str) -> Self {
        let client = RpcClient::new(url.to_string());
        Self { client }
    }

    pub fn get_tx_signature_by_address_before(
        &self,
        address: &str,
        mut before: Option<Signature>,
    ) -> anyhow::Result<Vec<String>> {
        let address = solana_sdk::pubkey::Pubkey::from_str(address)?;
        let mut all_txs = Vec::new();
        loop {
            let config = GetConfirmedSignaturesForAddress2Config {
                before,
                until: None,
                limit: Some(1000),
                commitment: Some(CommitmentConfig::confirmed()),
            };
            let mut result = self
                .client
                .get_signatures_for_address_with_config(&address, config)?
                .into_iter()
                .collect::<Vec<_>>();
            let last_signature = result.last();
            tracing::info!("last_signature: {:?}", last_signature);
            before = Some(Signature::from_str(
                &result
                    .last()
                    .ok_or(anyhow::anyhow!("get signatures is empty"))?
                    .signature
                    .clone(),
            )?);
            if result.len() < 1000 {
                all_txs.append(&mut result);
                break;
            } else {
                all_txs.append(&mut result);
                continue;
            }
        }
        tracing::info!("Address {} have {} transacition", address, all_txs.len());

        // filter error tx
        let all_txs = all_txs
            .into_iter()
            .filter(|tx| tx.err.is_none())
            .map(|tx| tx.signature)
            .collect::<Vec<_>>();
        tracing::info!(
            "Address {} have {} success transacition",
            address,
            all_txs.len()
        );
        Ok(all_txs)
    }

    pub async fn get_tx_signature_by_address_until(
        &self,
        address: &str,
        mut until: Option<Signature>,
    ) -> anyhow::Result<Vec<String>> {
        let address = solana_sdk::pubkey::Pubkey::from_str(address)?;
        let mut all_txs = Vec::new();
        loop {
            let config = GetConfirmedSignaturesForAddress2Config {
                before: None,
                until,
                limit: Some(1000),
                commitment: Some(CommitmentConfig::confirmed()),
            };
            let mut result = self
                .client
                .get_signatures_for_address_with_config(&address, config)?
                .into_iter()
                .collect::<Vec<_>>();
            let last_signature = result.last();
            tracing::info!("last_signature: {:?}", last_signature);
            until = Some(Signature::from_str(
                &result
                    .first()
                    .ok_or(anyhow::anyhow!("get signatures is empty"))?
                    .signature
                    .clone(),
            )?);
            if result.len() < 1000 {
                all_txs.append(&mut result);
                break;
            } else {
                all_txs.append(&mut result);
                continue;
            }
        }
        tracing::info!("Address {} have {} transacition", address, all_txs.len());

        // filter error tx
        let all_txs = all_txs
            .into_iter()
            .filter(|tx| tx.err.is_none())
            .map(|tx| tx.signature)
            .collect::<Vec<_>>();
        tracing::info!(
            "Address {} have {} success transacition",
            address,
            all_txs.len()
        );
        Ok(all_txs)
    }
}
