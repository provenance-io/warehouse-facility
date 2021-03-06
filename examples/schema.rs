use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use warehouse_facility::contract_info::ContractInfo;
use warehouse_facility::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use warehouse_facility::state::{Facility, Pledge};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(ContractInfo), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(Facility), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    export_schema(&schema_for!(Pledge), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
}
