use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::state::Facility;

const NAMESPACE_CONTRACT_INFO: &str = "contract_info";
pub const CONTRACT_TYPE: &str = "figure:smart-contracts.warehouse-facility";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const CONTRACT_INFO: Item<ContractInfo> = Item::new(NAMESPACE_CONTRACT_INFO);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfo {
    pub admin: Addr,
    pub bind_name: String,
    pub contract_name: String,
    pub version: String,
    pub contract_type: String,
    pub contract_version: String,
    pub facility: Facility,
}

impl ContractInfo {
    pub fn new(
        admin: Addr,
        bind_name: String,
        contract_name: String,
        version: String,
        facility: Facility,
    ) -> ContractInfo {
        ContractInfo {
            admin,
            bind_name,
            contract_name,
            version,
            contract_type: CONTRACT_TYPE.into(),
            contract_version: CONTRACT_VERSION.into(),
            facility,
        }
    }
}

pub fn set_contract_info(
    store: &mut dyn Storage,
    contract_info: &ContractInfo,
) -> Result<(), ContractError> {
    let result = CONTRACT_INFO.save(store, &contract_info);
    result.map_err(ContractError::Std)
}

pub fn get_contract_info(store: &dyn Storage) -> StdResult<ContractInfo> {
    CONTRACT_INFO.load(store)
}

#[cfg(test)]
mod tests {
    use provwasm_mocks::mock_dependencies;

    use crate::contract_info::{
        get_contract_info, set_contract_info, ContractInfo, CONTRACT_TYPE, CONTRACT_VERSION,
    };
    use crate::state::Facility;
    use cosmwasm_std::Addr;

    #[test]
    pub fn set_contract_info_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        let result = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
                "ver".to_string(),
                Facility {
                    originator: Addr::unchecked("contract_originator"),
                    warehouse: Addr::unchecked("contract_warehouse"),
                    marker_denom: "test.denom.wf1".into(),
                    stablecoin_denom: "test.denom.stable".into(),
                    advance_rate: "75.125".into(),
                    paydown_rate: "102.25".into(),
                },
            ),
        );
        match result {
            Ok(()) => {}
            result => panic!("unexpected error: {:?}", result),
        }

        let contract_info = get_contract_info(&deps.storage);
        match contract_info {
            Ok(contract_info) => {
                assert_eq!(contract_info.admin, Addr::unchecked("contract_admin"));
                assert_eq!(contract_info.bind_name, "contract_bind_name");
                assert_eq!(contract_info.contract_name, "contract_name");
                assert_eq!(contract_info.version, "ver");
                assert_eq!(contract_info.contract_type, CONTRACT_TYPE);
                assert_eq!(contract_info.contract_version, CONTRACT_VERSION);
                assert_eq!(
                    contract_info.facility.originator,
                    Addr::unchecked("contract_originator")
                );
                assert_eq!(
                    contract_info.facility.warehouse,
                    Addr::unchecked("contract_warehouse")
                );
                assert_eq!(contract_info.facility.marker_denom, "test.denom.wf1");
                assert_eq!(contract_info.facility.stablecoin_denom, "test.denom.stable");
                assert_eq!(contract_info.facility.advance_rate, "75.125");
                assert_eq!(contract_info.facility.paydown_rate, "102.25");
            }
            result => panic!("unexpected error: {:?}", result),
        }
    }
}
