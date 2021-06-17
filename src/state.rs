use cosmwasm_std::{Addr, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Facility {
    // The address of the originator.
    pub originator: Addr,

    // The address of the warehouse provider.
    pub warehouse: Addr,

    // The new marker denom to create representing fractional
    // ownership of assets in this facility.
    pub marker_denom: String,

    // The stablecoin denom used for the advance from the warehouse.
    pub stablecoin_denom: String,

    // The advance rate of the facility agreement with the warehouse
    // as a percentage (for example: "75.125" = 75.125%).
    pub advance_rate: String,

    // The paydown rate of the facility agreement with the warehouse
    // as a percentage (for example: "102.25" = 102.25%).
    pub paydown_rate: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PledgeState {
    // The originator has proposed the pledge to the facility.
    Proposed,

    // The warehouse has accepted the proposal.
    Accepted,

    // The originator has cancelled the pledge proposal.
    Cancelled,

    // The originator has executed the pledge.
    Executed,

    // The originator has payed-down the assets in the pledge.
    Closed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Pledge {
    pub id: String,
    pub assets: Vec<String>,
    pub total_advance: u64,
    pub asset_marker_denom: String,
    pub state: PledgeState,
}

pub const NAMESPACE_PLEDGES: &str = "pledges";
const PLEDGES: Map<&[u8], Pledge> = Map::new(NAMESPACE_PLEDGES);

pub fn load_pledge(storage: &dyn Storage, key: &[u8]) -> StdResult<Pledge> {
    PLEDGES.load(storage, key)
}

pub fn save_pledge(storage: &mut dyn Storage, key: &[u8], pledge: &Pledge) -> StdResult<()> {
    PLEDGES.save(storage, key, pledge)
}

pub fn get_pledge_ids(
    storage: &dyn Storage,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<String>> {
    Ok(PLEDGES
        .keys(storage, min, max, Order::Ascending)
        .map(|id| String::from_utf8(id).unwrap())
        .collect::<Vec<String>>())
}

pub fn get_pledges(
    storage: &dyn Storage,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<Pledge>> {
    Ok(get_pledge_ids(storage, min, max)?
        .iter()
        .map(|id| load_pledge(storage, id.as_bytes()).unwrap())
        .collect::<Vec<Pledge>>())
}
