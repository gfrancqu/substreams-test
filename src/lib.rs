mod pb;

use borsh::BorshDeserialize;

use bs58;
use pb::sf::substreams::sink::files::v1::Lines;
use serde::Serialize;
use serde_json::json;
use substreams::log;
use substreams_solana::pb::sol::{self};

enum RainInstruction {
    TakeLoan(RainInstructionPayload),
}

#[derive(BorshDeserialize)]
struct RainInstructionPayload {
    collection_id: u32,
    duration: u64,
    interest: u64,
    amount: u64,
    slippage: u16,
}

enum RainError {
    DeserializeError,
}

impl RainInstruction {
    pub fn unpack(input: &Vec<u8>) -> Result<Self, RainError> {
        if input.len() < 8 {
            return Err(RainError::DeserializeError);
        }
        let (variant, rest) = input.split_at(8);
        match variant {
            [153, 53, 51, 59, 222, 102, 52, 131] => {
                let payload = RainInstructionPayload::try_from_slice(rest).unwrap();
                Ok(Self::TakeLoan(payload))
            }
            _ => return Err(RainError::DeserializeError),
        }
    }
}

#[derive(Serialize)]
pub struct RainLoan {
    pool: String,
    loaner: String,
    collection: u32,
    amount: u64,
    duration: u64,
    floor_price: u64,
    interest: u64,
    slippage: u32,
}

const RAIN_CONTRACT: &str = "RainEraPU5yDoJmTrHdYynK9739GkEfDsE4ffqce2BR";

#[substreams::handlers::map]
fn map_block(block: sol::v1::Block) -> Result<Lines, substreams::errors::Error> {
    let rain_contract: Vec<u8> = bs58::decode(RAIN_CONTRACT).into_vec().unwrap();

    let loans: Vec<RainLoan> = block
        .transactions
        .into_iter()
        .filter_map(move |tx| {
            if let Some(t) = &tx.transaction {
                if let Some(msg) = &t.message {
                    if let Some(account_idx) = msg
                        .account_keys
                        .iter()
                        .position(|acc| acc == &rain_contract)
                    {
                        //log::info!("rain account is at {:?}", account_idx);
                        return Some(msg.instructions.clone().into_iter().filter_map(move |ix| {
                            let instruction = RainInstruction::unpack(&ix.data);
                            // Match the returned data struct to what you expect
                            if let Ok(payload) = instruction {
                                /*log::info!(
                                    "There is a loan ! accounts:{:?}",
                                    tx.clone()
                                        .transaction
                                        .unwrap()
                                        .message
                                        .unwrap()
                                        .account_keys
                                );*/
                                match payload {
                                    RainInstruction::TakeLoan(loan_ix) => {
                                        return Some(RainLoan {
                                            pool: bs58::encode(
                                                tx.clone()
                                                    .transaction
                                                    .unwrap()
                                                    .message
                                                    .unwrap()
                                                    .account_keys
                                                    .get(ix.accounts[8] as usize)
                                                    .unwrap(),
                                            )
                                            .into_string(),
                                            loaner: bs58::encode(
                                                tx.clone()
                                                    .transaction
                                                    .unwrap()
                                                    .message
                                                    .unwrap()
                                                    .account_keys
                                                    .get(ix.accounts[0] as usize)
                                                    .unwrap(),
                                            )
                                            .into_string(),
                                            collection: loan_ix.collection_id,
                                            amount: loan_ix.amount,
                                            duration: loan_ix.duration,
                                            floor_price: 0,
                                            interest: loan_ix.interest,
                                            slippage: loan_ix.slippage as u32,
                                        });
                                    }
                                }
                            } else {
                                return None;
                            }
                        }));
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        })
        .flatten()
        .collect();

    let loans_it = loans.into_iter();
    Ok(Lines {
        lines: loans_it
            .map(|l| {
                return json!({
                    "pool": l.pool,
                    "loaner": l.loaner,
                    "collection": l.collection,
                    "amount": l.amount,
                    "duration": l.duration,
                    "floor_price": l.floor_price,
                    "interest": l.interest,
                    "slippage": l.slippage,
                })
                .to_string();
            })
            .collect(),
    })
}
