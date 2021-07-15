use crate::contract_info::ContractInfo;
use crate::error::ContractError;
use crate::state::Facility;
use cosmwasm_std::Addr;
use rust_decimal::prelude::FromStr;
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait Validate {
    fn validate(&self) -> Result<(), ContractError>;
}

pub trait Authorize {
    fn authorize(&self, contract_info: ContractInfo, sender: Addr) -> Result<(), ContractError>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub bind_name: String,
    pub contract_name: String,
    pub facility: Facility,
}

/// Simple validation of InstantiateMsg data
///
/// ### Example
///
/// ```rust
/// use warehouse_facility::msg::{InstantiateMsg, Validate};
/// pub fn instantiate(msg: InstantiateMsg){
///     let result = msg.validate();
///     todo!()
/// }
/// ```
impl Validate for InstantiateMsg {
    fn validate(&self) -> Result<(), ContractError> {
        let mut invalid_fields: Vec<&str> = vec![];

        // validate the bind name
        if self.bind_name.is_empty() {
            invalid_fields.push("bind_name");
        }

        // validate the contract name
        if self.contract_name.is_empty() {
            invalid_fields.push("contract_name");
        }

        // validate the facility originator address
        if self.facility.originator.as_str().is_empty() {
            invalid_fields.push("facility.originator");
        }

        // validate the facility warehouse address
        if self.facility.warehouse.as_str().is_empty() {
            invalid_fields.push("facility.warehouse");
        }

        // validate the facility escrow marker address
        if self.facility.escrow_marker.as_str().is_empty() {
            invalid_fields.push("facility.escrow_marker");
        }

        // validate the facility marker denom
        if self.facility.marker_denom.is_empty() {
            invalid_fields.push("facility.marker_denom");
        }

        // validate the stablecoin denom
        if self.facility.stablecoin_denom.is_empty() {
            invalid_fields.push("facility.stablecoin_denom");
        }

        // validate the advance rate
        let advance_rate = Decimal::from_str(&self.facility.advance_rate)
            .map_err(|_| invalid_fields.push("facility.advance_rate"))
            .unwrap();
        if advance_rate <= Decimal::from(0) || advance_rate > Decimal::from(100) {
            invalid_fields.push("facility.advance_rate");
        }

        // validate the paydown rate
        let paydown_rate = Decimal::from_str(&self.facility.paydown_rate)
            .map_err(|_| invalid_fields.push("facility.paydown_rate"))
            .unwrap();
        if paydown_rate <= Decimal::from(0) {
            invalid_fields.push("facility.paydown_rate");
        }

        match invalid_fields.len() {
            0 => Ok(()),
            _ => Err(ContractError::InvalidFields {
                fields: invalid_fields.into_iter().map(|item| item.into()).collect(),
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Propose pledging assets to the warehouse facility (originator)
    ProposePledge {
        // A unique identifier for this pledge.
        id: String,

        // A list of assets to include in the pledge.
        assets: Vec<String>,

        // The total requested advance for the pledged assets.
        total_advance: u64,

        // The marker denom to create representing the encumbered
        // pool of pledged assets.
        asset_marker_denom: String,
    },

    // Accept a proposal to pledge assets to the warehouse facility (warehouse)
    AcceptPledge {
        // The unique identifier of the pledge.
        id: String,
    },

    // Cancel a proposal to pledge assets to the warehouse facility (originator)
    CancelPledge {
        // The unique identifier of the pledge.
        id: String,
    },

    // Executes a proposal to pledge assets to the warehouse facility (originator)
    ExecutePledge {
        // The unique identifier of the pledge.
        id: String,
    },

    // Propose a paydown of a pledge to the warehouse facility (originator)
    ProposePaydown {
        // The unique identifier of the paydown.
        id: String,

        // A list of assets to include in the pledge.
        assets: Vec<String>,

        // The total proposed paydown for the pledged assets.
        total_paydown: u64,
    },

    // Accept a proposal to paydown assets in the warehouse facility (warehouse)
    AcceptPaydown {
        // The unique identifier of the paydown.
        id: String,
    },

    // Cancel a proposal to paydown assets in the warehouse facility (originator)
    CancelPaydown {
        // The unique identifier of the paydown.
        id: String,
    },

    // Executes a proposal to paydown assets int the warehouse facility (originator)
    ExecutePaydown {
        // The unique identifier of the paydown.
        id: String,
    },
}

/// Simple validation of ExecuteMsg data
///
/// ### Example
///
/// ```rust
/// use warehouse_facility::msg::{ExecuteMsg, Validate};
/// pub fn execute(msg: ExecuteMsg){
///     let result = msg.validate();
///     todo!()
/// }
/// ```
impl Validate for ExecuteMsg {
    fn validate(&self) -> Result<(), ContractError> {
        let mut invalid_fields: Vec<&str> = vec![];

        match self {
            ExecuteMsg::ProposePledge {
                id,
                assets,
                total_advance: _,
                asset_marker_denom,
            } => {
                // validate the pledge id
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }

                // validate the assets
                if assets.is_empty() {
                    invalid_fields.push("assets");
                }
                for asset in assets {
                    if Uuid::parse_str(&asset).is_err() {
                        invalid_fields.push("asset");
                    }
                }

                // validate the marker denom
                if asset_marker_denom.is_empty() {
                    invalid_fields.push("asset_marker_denom");
                }
            }

            ExecuteMsg::AcceptPledge { id } => {
                // validate the pledge id
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }
            }

            ExecuteMsg::CancelPledge { id } => {
                // validate the pledge id
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }
            }

            ExecuteMsg::ExecutePledge { id } => {
                // validate the pledge id
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }
            }

            ExecuteMsg::ProposePaydown {
                id,
                assets,
                total_paydown: _,
            } => {
                // validate the paydown id
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }

                // validate the assets
                if assets.is_empty() {
                    invalid_fields.push("assets");
                }
                for asset in assets {
                    if Uuid::parse_str(&asset).is_err() {
                        invalid_fields.push("asset");
                    }
                }
            }

            ExecuteMsg::AcceptPaydown { id } => {
                // validate the paydown id
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }
            }

            ExecuteMsg::CancelPaydown { id } => {
                // validate the paydown id
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }
            }

            ExecuteMsg::ExecutePaydown { id } => {
                // validate the paydown id
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }
            }
        }

        match invalid_fields.len() {
            0 => Ok(()),
            _ => Err(ContractError::InvalidFields {
                fields: invalid_fields.into_iter().map(|item| item.into()).collect(),
            }),
        }
    }
}

impl Authorize for ExecuteMsg {
    fn authorize(&self, contract_info: ContractInfo, sender: Addr) -> Result<(), ContractError> {
        let mut authorized: bool = true;

        match self {
            ExecuteMsg::ProposePledge {
                id: _,
                assets: _,
                total_advance: _,
                asset_marker_denom: _,
            } => {
                // only the originator in this facility can propose a pledge
                if contract_info.facility.originator != sender {
                    authorized = false;
                }
            }

            ExecuteMsg::AcceptPledge { id: _ } => {
                // only the warehouse in this facility can accept a pledge
                if contract_info.facility.warehouse != sender {
                    authorized = false;
                }
            }

            ExecuteMsg::CancelPledge { id: _ } => {
                // only the originator in this facility can cancel a pledge
                if contract_info.facility.originator != sender {
                    authorized = false;
                }
            }

            ExecuteMsg::ExecutePledge { id: _ } => {
                // only the originator in this facility can execute a pledge
                if contract_info.facility.originator != sender {
                    authorized = false;
                }
            }

            ExecuteMsg::ProposePaydown {
                id: _,
                assets: _,
                total_paydown: _,
            } => {
                // only the originator in this facility can propose a paydown
                if contract_info.facility.originator != sender {
                    authorized = false;
                }
            }

            ExecuteMsg::AcceptPaydown { id: _ } => {
                // only the warehouse in this facility can accept a paydown
                if contract_info.facility.warehouse != sender {
                    authorized = false;
                }
            }

            ExecuteMsg::CancelPaydown { id: _ } => {
                // only the originator in this facility can cancel a paydown
                if contract_info.facility.originator != sender {
                    authorized = false;
                }
            }

            ExecuteMsg::ExecutePaydown { id: _ } => {
                // only the originator in this facility can execute a paydown
                if contract_info.facility.originator != sender {
                    authorized = false;
                }
            }
        }

        match authorized {
            true => Ok(()),
            false => Err(ContractError::Unauthorized {}),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get the contract info.
    GetContractInfo {},

    // Get the facility info.
    GetFacilityInfo {},

    // Get info about a pledge in the facility.
    GetPledge { id: String },

    // List the ids of all pledges in the facility.
    ListPledgeIds {},

    // List info about all pledges in the facility.
    ListPledges {},

    // List info about all open pledge proposals in the facility.
    ListPledgeProposals {},

    // List the ids of all paydowns in the facility.
    ListPaydownIds {},

    // List info about all paydowns in the facility.
    ListPaydowns {},

    // List info about all open paydown proposals in the facility.
    ListPaydownProposals {},

    // Get info about a paydown in the facility.
    GetPaydown { id: String },

    // List the assets currently involved in the facility (whether
    // proposed for pledge/paydown or currently in the inventory).
    ListAssets {},

    // List the assets currently in the facility inventory.
    ListInventory {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {
    Migrate {},
}
