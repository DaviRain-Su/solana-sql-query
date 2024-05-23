use clap::Parser;
use redb::{Database, TableDefinition};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::UiInstruction;
use solana_transaction_status::UiParsedInstruction;
use solana_transaction_status::UiTransactionEncoding;
use solana_transaction_status::{EncodedTransaction, UiMessage};
use tracing::info;

const TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("solana-block");

mod db;

#[derive(Parser, Debug)]
pub struct Server {
    #[arg(long, default_value = "8080")]
    pub port: u16,
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long)]
    pub workers: Option<usize>,
    #[arg(long)]
    pub threads: Option<usize>,
    #[arg(long)]
    pub keep_alive: Option<usize>,
    #[arg(long)]
    pub keep_alive_timeout: Option<usize>,
}

impl Server {
    pub async fn run(&self) -> anyhow::Result<()> {
        let client = RpcClient::new("https://gayleen-v43l6p-fast-mainnet.helius-rpc.com");
        let db = Database::create("solana.redb")?;
        let write_txn = db.begin_write()?;
        // make parrallel request use thread pool
        for solt in 32030090.. {
            let mut table = write_txn.open_table(TABLE)?;
            info!("processing slot: {}", solt);
            // 配置请求参数，包含 maxSupportedTransactionVersion
            let config = RpcBlockConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                transaction_details: Some(solana_transaction_status::TransactionDetails::Full),
                rewards: None,
                commitment: Some(CommitmentConfig::finalized()),
                max_supported_transaction_version: Some(0),
            };
            let mut result = client.get_block_with_config(solt, config)?;
            let tx = result.transactions.unwrap_or_default();
            info!("tx len all have {}", tx.len());

            // filter success tx
            let txs_success = tx
                .into_iter()
                .filter(|tx| {
                    if let Some(meta) = &tx.meta {
                        meta.err.is_none()
                    } else {
                        true
                    }
                })
                .collect::<Vec<_>>();

            info!("tx_success length: {}", txs_success.len());

            // filter vote program instruction Vote111111111111111111111111111111111111111
            let mut filter_vote_program = Vec::new();
            for tx1 in txs_success.iter() {
                match &tx1.transaction {
                    EncodedTransaction::Json(tx) => match &tx.message {
                        // judege fisrt instruction is not vote program
                        UiMessage::Parsed(message) => match &message.instructions[0] {
                            UiInstruction::Compiled(_compiled) => todo!(),
                            UiInstruction::Parsed(parsed) => match parsed {
                                UiParsedInstruction::Parsed(value1) => {
                                    if value1.program != "vote" {
                                        filter_vote_program.push(tx1.clone());
                                    }
                                }
                                UiParsedInstruction::PartiallyDecoded(_value2) => {
                                    filter_vote_program.push(tx1.clone());
                                }
                            },
                        },
                        UiMessage::Raw(_) => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            }

            info!(
                "filter_vote_program after length: {}",
                filter_vote_program.len()
            );
            let len = filter_vote_program.len();
            result.transactions = Some(filter_vote_program);

            if len != 0 {
                table.insert(solt, &*bincode::serialize(&result)?)?;
            }
        }
        write_txn.commit()?;

        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        println!("{:?}", table);

        Ok(())
    }
}
