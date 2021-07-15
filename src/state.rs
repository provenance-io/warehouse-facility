use crate::utils::vec_has_any;
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

    // The address of the escrow marker.
    pub escrow_marker: Addr,

    // The new marker denom to create representing fractional
    // ownership of assets in this facility.
    pub marker_denom: String,

    // The stablecoin denom used for the advance from the warehouse.
    pub stablecoin_denom: String,

    // The advance rate of the facility agreement with the warehouse
    // as a percentage (for example: "75.125" = 75.125%).
    pub advance_rate: String,

    // The paydown rate of the facility agreement with the warehouse
    // as a percentage of the UPB (for example: "77.25" = 77.25%).
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
    state: Option<PledgeState>,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<String>> {
    Ok(PLEDGES
        .keys(storage, min, max, Order::Ascending)
        .filter(|id| {
            if state.is_none() {
                true
            } else {
                return &load_pledge(storage, id).unwrap().state == state.as_ref().unwrap();
            }
        })
        .map(|id| String::from_utf8(id).unwrap())
        .collect::<Vec<String>>())
}

pub fn get_pledges(
    storage: &dyn Storage,
    state: Option<PledgeState>,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<Pledge>> {
    Ok(get_pledge_ids(storage, state, min, max)?
        .iter()
        .map(|id| load_pledge(storage, id.as_bytes()).unwrap())
        .collect::<Vec<Pledge>>())
}

pub fn find_pledge_ids_with_assets(
    storage: &dyn Storage,
    assets: Vec<String>,
    state: Option<PledgeState>,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<String>> {
    Ok(PLEDGES
        .keys(storage, min, max, Order::Ascending)
        .filter(|id| {
            let pledge = load_pledge(storage, id).unwrap();
            if state.is_none() || &pledge.state == state.as_ref().unwrap() {
                vec_has_any(&pledge.assets, &assets)
            } else {
                false
            }
        })
        .map(|id| String::from_utf8(id).unwrap())
        .collect::<Vec<String>>())
}

pub fn find_pledges_with_assets(
    storage: &dyn Storage,
    assets: Vec<String>,
    state: Option<PledgeState>,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<Pledge>> {
    Ok(
        find_pledge_ids_with_assets(storage, assets, state, min, max)?
            .iter()
            .map(|id| load_pledge(storage, id.as_bytes()).unwrap())
            .collect::<Vec<Pledge>>(),
    )
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetState {
    // A pledge proposal exists for this asset.
    PledgeProposed,

    // The asset is part of the facility inventory.
    Inventory,

    // A paydown proposal exists for this asset.
    PaydownProposed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Asset {
    pub id: String,
    pub state: AssetState,
}

pub const NAMESPACE_ASSETS: &str = "assets";
const ASSETS: Map<&[u8], Asset> = Map::new(NAMESPACE_ASSETS);

pub fn load_asset(storage: &dyn Storage, key: &[u8]) -> StdResult<Asset> {
    ASSETS.load(storage, key)
}

pub fn save_asset(storage: &mut dyn Storage, key: &[u8], asset: &Asset) -> StdResult<()> {
    ASSETS.save(storage, key, asset)
}

pub fn remove_asset(storage: &mut dyn Storage, key: &[u8]) -> StdResult<()> {
    ASSETS.remove(storage, key);
    Ok(())
}

// Set the assets to the specified state in the inventory.
pub fn set_assets_state(
    storage: &mut dyn Storage,
    state: AssetState,
    ids: &[String],
) -> StdResult<()> {
    for id in ids {
        save_asset(
            storage,
            id.as_bytes(),
            &Asset {
                id: id.to_string(),
                state: state.clone(),
            },
        )?;
    }
    Ok(())
}

// Remove assets from the inventory.
pub fn remove_assets(storage: &mut dyn Storage, ids: &[String]) -> StdResult<()> {
    for id in ids {
        remove_asset(storage, id.as_bytes())?;
    }
    Ok(())
}

pub fn get_asset_ids(
    storage: &dyn Storage,
    state: Option<AssetState>,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<String>> {
    Ok(ASSETS
        .keys(storage, min, max, Order::Ascending)
        .filter(|id| {
            if state.is_none() {
                true
            } else {
                return &load_asset(storage, id).unwrap().state == state.as_ref().unwrap();
            }
        })
        .map(|id| String::from_utf8(id).unwrap())
        .collect::<Vec<String>>())
}

pub fn get_asset_ids_by_filter(
    storage: &dyn Storage,
    filter: Vec<AssetState>,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<String>> {
    Ok(ASSETS
        .keys(storage, min, max, Order::Ascending)
        .filter(|id| filter.contains(&load_asset(storage, id).unwrap().state))
        .map(|id| String::from_utf8(id).unwrap())
        .collect::<Vec<String>>())
}

pub fn get_assets(
    storage: &dyn Storage,
    state: Option<AssetState>,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<Asset>> {
    Ok(get_asset_ids(storage, state, min, max)?
        .iter()
        .map(|id| load_asset(storage, id.as_bytes()).unwrap())
        .collect::<Vec<Asset>>())
}

pub fn get_assets_by_filter(
    storage: &dyn Storage,
    filter: Vec<AssetState>,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<Asset>> {
    Ok(get_asset_ids_by_filter(storage, filter, min, max)?
        .iter()
        .map(|id| load_asset(storage, id.as_bytes()).unwrap())
        .collect::<Vec<Asset>>())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PaydownState {
    // The originator has proposed the paydown to the facility.
    Proposed,

    // The warehouse has accepted the proposal.
    Accepted,

    // The originator has cancelled the paydown proposal.
    Cancelled,

    // The originator has executed the paydown.
    Executed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Paydown {
    pub id: String,
    pub assets: Vec<String>,
    pub total_paydown: u64,
    pub state: PaydownState,
}

pub const NAMESPACE_PAYDOWNS: &str = "paydowns";
const PAYDOWNS: Map<&[u8], Paydown> = Map::new(NAMESPACE_PAYDOWNS);

pub fn load_paydown(storage: &dyn Storage, key: &[u8]) -> StdResult<Paydown> {
    PAYDOWNS.load(storage, key)
}

pub fn save_paydown(storage: &mut dyn Storage, key: &[u8], paydown: &Paydown) -> StdResult<()> {
    PAYDOWNS.save(storage, key, paydown)
}

pub fn get_paydown_ids(
    storage: &dyn Storage,
    state: Option<PaydownState>,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<String>> {
    Ok(PAYDOWNS
        .keys(storage, min, max, Order::Ascending)
        .filter(|id| {
            if state.is_none() {
                true
            } else {
                return &load_paydown(storage, id).unwrap().state == state.as_ref().unwrap();
            }
        })
        .map(|id| String::from_utf8(id).unwrap())
        .collect::<Vec<String>>())
}

pub fn get_paydowns(
    storage: &dyn Storage,
    state: Option<PaydownState>,
    min: Option<Bound>,
    max: Option<Bound>,
) -> StdResult<Vec<Paydown>> {
    Ok(get_paydown_ids(storage, state, min, max)?
        .iter()
        .map(|id| load_paydown(storage, id.as_bytes()).unwrap())
        .collect::<Vec<Paydown>>())
}
