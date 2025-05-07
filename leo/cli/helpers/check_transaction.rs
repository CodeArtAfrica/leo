// Copyright (C) 2019-2025 Provable Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use leo_errors::Result;
use leo_package::NetworkName;

use anyhow::anyhow;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub enum TransactionStatus {
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "aborted")]
    Aborted,
    #[serde(rename = "rejected")]
    Rejected,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    id: String,
}

#[derive(Debug, Deserialize)]
struct TransactionResult {
    status: TransactionStatus,

    transaction: Transaction,
}

#[derive(Debug, Deserialize)]
struct Block {
    transactions: Vec<TransactionResult>,
    aborted_transaction_ids: Vec<String>,
}

pub fn current_height(endpoint: &str, network: NetworkName) -> Result<usize> {
    let height_url = format!("{endpoint}/{network}/block/height/latest");
    let height_str = leo_package::fetch_from_network_plain(&height_url)?;
    let height: usize = height_str.parse().map_err(|e| anyhow!("error parsing height: {e}"))?;
    Ok(height)
}

fn status_at_height(
    id: &str,
    endpoint: &str,
    network: NetworkName,
    height: usize,
) -> Result<Option<TransactionStatus>> {
    const MAX_WAIT: usize = 8;

    // Wait until the block at `height` exists.
    for i in 0usize.. {
        if current_height(endpoint, network)? >= height {
            break;
        } else if i >= MAX_WAIT {
            // We've waited too long; give up.
            return Ok(None);
        } else {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    let block_url = format!("{endpoint}/{network}/block/{height}");
    let block_str = leo_package::fetch_from_network_plain(&block_url)?;
    let block: Block = serde_json::from_str(&block_str).expect("Failed to deserialize");
    let maybe_this_transaction =
        block.transactions.iter().find(|transaction_result| transaction_result.transaction.id == id);

    if let Some(transaction_result) = maybe_this_transaction {
        // We found it.
        return Ok(Some(transaction_result.status));
    }

    if block.aborted_transaction_ids.iter().any(|aborted_id| aborted_id == id) {
        // It was aborted.
        return Ok(Some(TransactionStatus::Aborted));
    }

    for rejected in &block.transactions {
        if rejected.status != TransactionStatus::Rejected {
            continue;
        }

        // The necessary call isn't yet implemented in snarkOS, so let's just print this for now.
        println!("Some rejected transaction found {}", rejected.transaction.id);

        // let url = format!("{endpoint}/{network}/unconfirmed/{}", rejected.transaction.id);
        // let other_id = leo_package::fetch_from_network_plain(&url)?;
        // if id == other_id {
        //     // It was rejected.
        //     return Ok(Some(TransactionStatus::Rejected));
        // }
    }

    Ok(None)
}

const BLOCKS_TO_CHECK: usize = 12;

struct CheckedTransaction {
    blocks_checked: usize,
    status: Option<TransactionStatus>,
}

fn check_transaction(
    id: &str,
    endpoint: &str,
    network: NetworkName,
    start_height: usize,
) -> Result<CheckedTransaction> {
    for use_height in start_height..start_height + BLOCKS_TO_CHECK {
        let status = status_at_height(id, endpoint, network, use_height)?;
        if status.is_some() {
            return Ok(CheckedTransaction { blocks_checked: use_height - start_height + 1, status });
        }

        // Avoid rate limits.
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(CheckedTransaction { blocks_checked: BLOCKS_TO_CHECK, status: None })
}

/// Check to find the transaction id among new blocks, printing its status (if found)
/// to the user. Returns `true` if and only if the transaction was found.
pub fn check_transaction_with_message(
    id: &str,
    endpoint: &str,
    network: NetworkName,
    start_height: usize,
) -> Result<Option<TransactionStatus>> {
    println!("Searching up to {BLOCKS_TO_CHECK} blocks to find transaction (this may take several seconds)...");
    let checked = crate::cli::check_transaction::check_transaction(id, endpoint, network, start_height)?;
    println!("Explored {} blocks.", checked.blocks_checked);
    match checked.status {
        Some(TransactionStatus::Accepted) => println!("Transaction accepted"),
        Some(TransactionStatus::Rejected) => println!("Transaction rejected"),
        Some(TransactionStatus::Aborted) => println!("Transaction aborted"),
        None => println!("Couldn't find the transaction after searching through several blocks"),
    }
    Ok(checked.status)
}
