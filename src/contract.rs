use crate::contract_info::{get_contract_info, set_contract_info, ContractInfo};
use crate::error::ContractError;
use crate::msg::{Authorize, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, Validate};
use crate::state::{
    get_pledge_ids, get_pledges, load_pledge, save_pledge, Facility, Pledge, PledgeState,
};
use cosmwasm_std::{
    attr, coins, entry_point, to_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Storage,
};
use provwasm_std::{
    activate_marker, bind_name, cancel_marker, create_marker, destroy_marker, finalize_marker,
    grant_marker_access, transfer_marker_coins, withdraw_coins, MarkerAccess, MarkerType,
    NameBinding, ProvenanceMsg, ProvenanceQuerier,
};
use rust_decimal::prelude::{FromStr, ToPrimitive};
use rust_decimal::Decimal;
use std::ops::{Div, Mul};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// smart contract initialization entrypoint
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // validate the message
    msg.validate()?;

    let advance_rate = Decimal::from_str(&msg.facility.advance_rate).map_err(|_| {
        ContractError::InvalidFields {
            fields: vec![String::from("facility.advance_rate")],
        }
    })?;

    let facility = msg.facility.clone();
    let contract_addr = env.contract.address.clone();

    // calculate the total supply and distribution of facility marker
    let facility_marker_supply: u128 = 10u128.pow(advance_rate.scale() + 2);
    let facility_marker_to_warehouse: u128 = advance_rate
        .div(Decimal::from(100))
        .mul(Decimal::from(facility_marker_supply))
        .to_u128()
        .unwrap();
    let facility_marker_to_originator: u128 = facility_marker_supply - facility_marker_to_warehouse;

    // save contract info
    let contract_info = ContractInfo::new(
        info.sender,
        msg.bind_name,
        msg.contract_name,
        CONTRACT_VERSION.into(),
        msg.facility,
    );
    set_contract_info(deps.storage, &contract_info)?;

    // messages to include in transaction
    let mut messages = Vec::new();

    // create name binding
    messages.push(bind_name(
        contract_info.bind_name,
        env.contract.address,
        NameBinding::Restricted,
    )?);

    // create facility marker
    messages.push(create_marker(
        facility_marker_supply,
        facility.marker_denom.clone(),
        MarkerType::Restricted,
    )?);

    // set privileges on the facility marker
    messages.push(grant_marker_access(
        facility.marker_denom.clone(),
        contract_addr,
        vec![
            MarkerAccess::Admin,
            MarkerAccess::Delete,
            MarkerAccess::Deposit,
            MarkerAccess::Transfer,
            MarkerAccess::Withdraw,
        ],
    )?);

    // finalize the facility marker
    messages.push(finalize_marker(facility.marker_denom.clone())?);

    // activate the facility marker
    messages.push(activate_marker(facility.marker_denom.clone())?);

    // withdraw the facility marker to the warehouse address
    messages.push(withdraw_coins(
        facility.marker_denom.clone(),
        facility_marker_to_warehouse,
        facility.marker_denom.clone(),
        Addr::unchecked(facility.warehouse),
    )?);

    // withdraw the facility marker to the originator address
    messages.push(withdraw_coins(
        facility.marker_denom.clone(),
        facility_marker_to_originator,
        facility.marker_denom.clone(),
        Addr::unchecked(facility.originator),
    )?);

    // build response
    Ok(Response {
        submessages: vec![],
        messages,
        attributes: vec![
            attr(
                "contract_info",
                format!("{:?}", get_contract_info(deps.storage)?),
            ),
            attr("action", "init"),
        ],
        data: None,
    })
}

// smart contract execute entrypoint
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // validate the message
    msg.validate()?;

    // authorize the sender
    let contract_info = get_contract_info(deps.storage)?;
    msg.authorize(contract_info.clone(), info.sender.clone())?;

    match msg {
        ExecuteMsg::ProposePledge {
            id,
            assets,
            total_advance,
            asset_marker_denom,
        } => propose_pledge(
            deps,
            env,
            info,
            contract_info,
            id,
            assets,
            total_advance,
            asset_marker_denom,
        ),
        ExecuteMsg::AcceptPledge { id } => accept_pledge(deps, env, info, contract_info, id),
        ExecuteMsg::CancelPledge { id } => cancel_pledge(deps, env, info, contract_info, id),
        ExecuteMsg::ExecutePledge { id } => execute_pledge(deps, env, info, contract_info, id),
    }
}

#[allow(clippy::too_many_arguments)]
fn propose_pledge(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    contract_info: ContractInfo,
    id: String,
    assets: Vec<String>,
    total_advance: u64,
    asset_marker_denom: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // ensure that a pledge with the specified id doesn't already exist
    let pledge = load_pledge(deps.storage, id.as_bytes());
    if let Ok(v) = pledge {
        return Err(ContractError::PledgeAlreadyExists { id: v.id });
    }

    // create the pledge
    let pledge = Pledge {
        id,
        assets,
        total_advance,
        asset_marker_denom: asset_marker_denom.clone(),
        state: PledgeState::Proposed,
    };

    // save the pledge
    save_pledge(deps.storage, &pledge.id.as_bytes(), &pledge)?;

    // messages to include in transaction
    let messages = vec![
        // create asset pool marker
        create_marker(1, asset_marker_denom.clone(), MarkerType::Restricted)?,
        // set privileges on the asset pool marker
        grant_marker_access(
            asset_marker_denom.clone(),
            env.contract.address,
            vec![
                MarkerAccess::Admin,
                MarkerAccess::Burn,
                MarkerAccess::Delete,
                MarkerAccess::Deposit,
                MarkerAccess::Mint,
                MarkerAccess::Transfer,
                MarkerAccess::Withdraw,
            ],
        )?,
        // finalize the asset pool marker
        finalize_marker(asset_marker_denom.clone())?,
        // activate the asset pool marker
        activate_marker(asset_marker_denom.clone())?,
        // withdraw the asset pool marker to the originator address
        withdraw_coins(
            asset_marker_denom.clone(),
            1,
            asset_marker_denom,
            Addr::unchecked(contract_info.facility.originator),
        )?,
    ];

    Ok(Response {
        submessages: vec![],
        messages,
        attributes: vec![attr("action", "propose_pledge")],
        data: Some(to_binary(&pledge)?),
    })
}

fn accept_pledge(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_info: ContractInfo,
    id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // locate the pledge
    let mut pledge = load_pledge(deps.storage, id.as_bytes())?;

    // only pledges that are in the "PROPOSED" state can be accepted
    if pledge.state != PledgeState::Proposed {
        return Err(ContractError::StateError {
            error: "Unable to accept pledge: Pledge is not in the 'proposed' state.".into(),
        });
    }

    // make sure that the warehouse sent the appropriate stablecoin
    let advance_funds = info
        .funds
        .get(0)
        .ok_or(ContractError::MissingPledgeAdvance {})?;
    if (advance_funds.denom != contract_info.facility.stablecoin_denom)
        || (advance_funds.amount != pledge.total_advance.into())
    {
        return Err(ContractError::InsufficientPledgeAdvance {
            need: pledge.total_advance.to_u128().unwrap(),
            need_denom: contract_info.facility.stablecoin_denom,
            received: advance_funds.amount.u128(),
            received_denom: advance_funds.denom.clone(),
        });
    }

    // update the pledge
    pledge.state = PledgeState::Accepted;
    save_pledge(deps.storage, &pledge.id.as_bytes(), &pledge)?;

    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "accept_pledge")],
        data: Some(to_binary(&pledge)?),
    })
}

fn cancel_pledge(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    contract_info: ContractInfo,
    id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // locate the pledge
    let mut pledge = load_pledge(deps.storage, id.as_bytes())?;

    // only pledges that are in the "PROPOSED" or "ACCEPTED" states can be cancelled
    let remove_assets_from_escrow = true;
    let mut remove_advance_from_escrow = false;
    match pledge.state {
        PledgeState::Proposed => {}
        PledgeState::Accepted => {
            remove_advance_from_escrow = true;
        }
        _ => {
            return Err(ContractError::StateError {
                error:
                    "Unable to cancel pledge: Pledge is not in the 'proposed' or 'accepted' state."
                        .into(),
            })
        }
    }

    // messages to include in transaction
    let mut messages = Vec::new();

    // remove the advance from escrow back to the warehouse account
    if remove_advance_from_escrow {
        messages.push(
            BankMsg::Send {
                to_address: contract_info.facility.warehouse.to_string(),
                amount: coins(
                    pledge.total_advance.into(),
                    contract_info.facility.stablecoin_denom,
                ),
            }
            .into(),
        );
    }

    // remove the assets (asset marker) from escrow
    if remove_assets_from_escrow {
        let querier = ProvenanceQuerier::new(&deps.querier);
        let asset_marker = querier.get_marker_by_denom(pledge.asset_marker_denom.clone())?;

        // transfer the asset marker back to the marker supply
        messages.push(transfer_marker_coins(
            1,
            pledge.asset_marker_denom.clone(),
            asset_marker.address,
            contract_info.facility.originator,
        )?);

        // cancel the asset marker
        messages.push(cancel_marker(pledge.asset_marker_denom.clone())?);

        // destroy the asset marker
        messages.push(destroy_marker(pledge.asset_marker_denom.clone())?);
    }

    // update the pledge
    pledge.state = PledgeState::Cancelled;
    save_pledge(deps.storage, &pledge.id.as_bytes(), &pledge)?;

    Ok(Response {
        submessages: vec![],
        messages,
        attributes: vec![attr("action", "cancel_pledge")],
        data: Some(to_binary(&pledge)?),
    })
}

fn execute_pledge(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    contract_info: ContractInfo,
    id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // locate the pledge
    let mut pledge = load_pledge(deps.storage, id.as_bytes())?;

    // only pledges that are in the "ACCEPTED" state can be executed
    if pledge.state != PledgeState::Accepted {
        return Err(ContractError::StateError {
            error: "Unable to execute pledge: Pledge is not in the 'accepted' state.".into(),
        });
    }

    // messages to include in transaction
    let messages = vec![
        // transfer stablecoin from escrow to the originator
        BankMsg::Send {
            to_address: contract_info.facility.originator.to_string(),
            amount: coins(
                pledge.total_advance.into(),
                contract_info.facility.stablecoin_denom,
            ),
        }
        .into(),
    ];

    /*
    // transfer stablecoin from escrow to the originator
    messages.push(
        BankMsg::Send {
            to_address: contract_info.facility.originator.to_string(),
            amount: coins(
                pledge.total_advance.into(),
                contract_info.facility.stablecoin_denom,
            ),
        }
        .into(),
    );
    */

    // update the pledge
    pledge.state = PledgeState::Executed;
    save_pledge(deps.storage, &pledge.id.as_bytes(), &pledge)?;

    Ok(Response {
        submessages: vec![],
        messages,
        attributes: vec![attr("action", "execute_pledge")],
        data: None,
    })
}

fn get_facility_info(store: &dyn Storage) -> StdResult<Facility> {
    let contract_info = get_contract_info(store)?;
    Ok(contract_info.facility)
}

fn get_pledge(store: &dyn Storage, id: String) -> StdResult<Pledge> {
    load_pledge(store, id.as_bytes())
}

fn list_pledge_ids(store: &dyn Storage) -> StdResult<Vec<String>> {
    get_pledge_ids(store, None, None)
}

fn list_pledges(store: &dyn Storage) -> StdResult<Vec<Pledge>> {
    get_pledges(store, None, None)
}

// smart contract query entrypoint
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetContractInfo {} => to_binary(&get_contract_info(deps.storage)?),
        QueryMsg::GetFacilityInfo {} => to_binary(&get_facility_info(deps.storage)?),
        QueryMsg::GetPledge { id } => to_binary(&get_pledge(deps.storage, id)?),
        QueryMsg::ListPledgeIds {} => to_binary(&list_pledge_ids(deps.storage)?),
        QueryMsg::ListPledges {} => to_binary(&list_pledges(deps.storage)?),
    }
}

// smart contract migrate/upgrade entrypoint
#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    // always update version info
    let mut contract_info = get_contract_info(deps.storage)?;
    contract_info.version = CONTRACT_VERSION.into();
    set_contract_info(deps.storage, &contract_info)?;

    Ok(Response::default())
}
