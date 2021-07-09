use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid fields: {fields:?}")]
    InvalidFields { fields: Vec<String> },

    #[error("State error: {error:?}")]
    StateError { error: String },

    #[error("Pledge already exists: {id:?}")]
    PledgeAlreadyExists { id: String },

    #[error("Facility contract missing grants on escrow marker")]
    MissingEscrowMarkerGrant {},

    #[error("Cannot accept pledge: Missing pledge advance")]
    MissingPledgeAdvance {},

    #[error("Cannot accept pledge: Insufficient funds: need {need:?} {need_denom:?}, received {received:?} {received_denom:?}")]
    InsufficientPledgeAdvance {
        need: u128,
        need_denom: String,
        received: u128,
        received_denom: String,
    },
}

impl From<ContractError> for StdError {
    fn from(error: ContractError) -> Self {
        StdError::GenericErr {
            msg: error.to_string(),
        }
    }
}
