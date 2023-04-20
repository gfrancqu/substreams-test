mod pb;
use borsh::BorshDeserialize;
use bs58;
use pb::sf::substreams::sink::files::v1::Lines;
use serde::Serialize;
use serde_json::json;
use substreams::log;
use substreams_solana::pb::sol::{self};

// Used to model our specific instruction arguments
#[derive(BorshDeserialize)]
struct RainLoanInstructionPayload {
    collection_id: u32,
    duration: u64,
    interest: u64,
    amount: u64,
    slippage: u16,
}

// An enum used for error handling
enum RainError {
    NotARainInstruction,
    DeserializeError,
}

//We can add more instructions in this enum
enum RainInstruction {
    TakeLoan(RainLoanInstructionPayload),
}

impl RainInstruction {
    // Instruction come as u8 Vector, this function unpack a an instruction and return a RainInstruction
    //result if the instruction match a specific discriminator
    pub fn unpack(input: &Vec<u8>) -> Result<Self, RainError> {
        if input.len() < 8 {
            return Err(RainError::NotARainInstruction);
        }
        let (variant, rest) = input.split_at(8);
        match variant {
            // Discriminator for TakeLoan instruction
            [153, 53, 51, 59, 222, 102, 52, 131] => {
                let payload = RainLoanInstructionPayload::try_from_slice(rest);
                if payload.is_err() {
                    return Err(RainError::DeserializeError);
                }
                Ok(Self::TakeLoan(payload.unwrap()))
            }
            _ => return Err(RainError::NotARainInstruction),
        }
    }
}

// this is the model we want to output, it serialize to json
// Note: you could define a protobuf data models and export this models
// and use another module to output to json
#[derive(Serialize)]
pub struct RainLoan {
    pool: String,
    loaner: String,
    collection: u32,
    amount: u64,
    duration: u64,
    interest: u64,
    slippage: u32,
    timestamp: u64,
}

// Program address we want to watch
const RAIN_CONTRACT: &str = "RainEraPU5yDoJmTrHdYynK9739GkEfDsE4ffqce2BR";

// Our module handler take a solana block as input, and produce `Lines`
#[substreams::handlers::map]
fn map_block(block: sol::v1::Block) -> Result<Lines, substreams::errors::Error> {
    return Ok(Lines {
        lines: vec![String::from("test")],
    });
    /*let rain_contract: Vec<u8> = bs58::decode(RAIN_CONTRACT).into_vec().unwrap();

    let Some(timestamp) = block.block_time else {
        return Ok(Lines { lines: vec![]});
    };

    let loans: Vec<RainLoan> = block
        .transactions
        .into_iter()
        .filter_map(|confirmed_tx| {
            let Some(tx) = confirmed_tx.transaction else { return None};
            let Some(msg) = tx.message else { return None };

            //check if there are any instructions for our specific contract in the account_keys
            if !msg.account_keys.contains(&rain_contract) {
                return None;
            }

            //filter only the instruction that concern our specific contract
            Some(msg.instructions.into_iter().filter_map(move |ix| {
                let Ok(instruction) = RainInstruction::unpack(&ix.data) else { return None };
                match instruction {
                    RainInstruction::TakeLoan(loan_ix) => {
                        Some(
                            RainLoan {
                                pool: bs58::encode(
                                    msg.account_keys.get(ix.accounts[8] as usize).unwrap(),
                                )
                                .into_string(),
                                loaner: bs58::encode(
                                    msg.account_keys.get(ix.accounts[0] as usize).unwrap(),
                                )
                                .into_string(),
                                collection: loan_ix.collection_id,
                                amount: loan_ix.amount,
                                duration: loan_ix.duration,
                                interest: loan_ix.interest,
                                slippage: loan_ix.slippage as u32,
                                timestamp: timestamp.timestamp as u64,
                            }
                        )
                    }
                }
            }))
        })
        .flatten()
        .collect();

    //output every loans to json
    Ok(Lines {
        lines: loans
            .into_iter()
            .map(|l| {
                return json!({
                    "pool": l.pool,
                    "loaner": l.loaner,
                    "collection": l.collection,
                    "amount": l.amount,
                    "duration": l.duration,
                    "interest": l.interest,
                    "slippage": l.slippage,
                    "timestamp": l.timestamp
                })
                .to_string();
            })
            .collect(),
    })*/
}
