use crate::state::ContractParty;
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

    #[error(
        "Cannot propose pledge: One or more assets has already been pledged or is in the inventory"
    )]
    AssetsAlreadyPledged {},

    #[error("Facility contract missing grants on escrow marker")]
    MissingEscrowMarkerGrant {},

    #[error("Cannot accept pledge: Missing pledge advance funds")]
    MissingPledgeAdvanceFunds {},

    #[error("Cannot accept pledge: Insufficient funds: need {need:?} {need_denom:?}, received {received:?} {received_denom:?}")]
    InsufficientPledgeAdvanceFunds {
        need: u128,
        need_denom: String,
        received: u128,
        received_denom: String,
    },

    #[error("Cannot propose paydown: Missing paydown funds")]
    MissingPaydownFunds {},

    #[error("Cannot propose paydown: Insufficient funds: need {need:?} {need_denom:?}, received {received:?} {received_denom:?}")]
    InsufficientPaydownFunds {
        need: u128,
        need_denom: String,
        received: u128,
        received_denom: String,
    },

    #[error("Paydown already exists: {id:?}")]
    PaydownAlreadyExists { id: String },

    #[error("Cannot propose paydown: Assets not in inventory")]
    AssetsNotInInventory {},

    #[error("Cannot accept paydown: Party {party:?} already accepted")]
    PaydownPartyAlreadyAccepted { party: ContractParty },

    #[error("Cannot accept paydown: Missing purchase funds")]
    MissingPurchaseFunds {},

    #[error("Cannot accept paydown: Insufficient purchase funds: need {need:?} {need_denom:?}, received {received:?} {received_denom:?}")]
    InsufficientPurchaseFunds {
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
