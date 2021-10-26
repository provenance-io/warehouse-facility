pub fn vec_contains<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    let matching = a.iter().filter(|&ae| b.iter().any(|be| be == ae)).count();
    matching == b.len()
}

pub fn vec_has_any<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    let matching = a.iter().filter(|&ae| b.iter().any(|be| be == ae)).count();
    matching > 0
}

use std::convert::TryInto;
use uuid::Uuid;
use bech32::{ self, FromBase32, ToBase32, Variant };
use sha2::{ Digest, Sha256 };

const PREFIX_SCOPE: &str = "scope";
const PREFIX_SESSION: &str  = "session";
const PREFIX_RECORD: &str  = "record";
const PREFIX_SCOPE_SPECIFICATION: &str  = "scopespec";
const PREFIX_CONTRACT_SPECIFICATION: &str  = "contractspec";
const PREFIX_RECORD_SPECIFICATION: &str  = "recspec";

const KEY_SCOPE: u8 = 0x00;
const KEY_SESSION: u8 = 0x01;
const KEY_RECORD: u8 = 0x02;
const KEY_SCOPE_SPECIFICATION: u8 = 0x04; // Note that this is not in numerical order.
const KEY_CONTRACT_SPECIFICATION: u8 = 0x03;
const KEY_RECORD_SPECIFICATION: u8 = 0x05;

pub struct MetadataAddress {
    bytes: Vec<u8>
}

impl MetadataAddress {

    pub fn for_scope(scope_uuid: Uuid) -> Self {
        let mut data: Vec<u8> = Vec::new();
        data.push(KEY_SCOPE);
        data.extend(MetadataAddress::uuid_as_byte_array(scope_uuid));
        MetadataAddress {
            bytes: data
        }
    }

    pub fn for_session(scope_uuid: Uuid, session_uuid: Uuid) -> Self {
        let mut data: Vec<u8> = Vec::new();
        data.push(KEY_SESSION);
        data.extend(MetadataAddress::uuid_as_byte_array(scope_uuid));
        data.extend(MetadataAddress::uuid_as_byte_array(session_uuid));
        MetadataAddress {
            bytes: data
        }
    }

    pub fn for_record(scope_uuid: Uuid, record_name: String) -> Self {
        /* TODO
        if (recordName.isBlank()) {
            throw IllegalArgumentException("Invalid recordName: cannot be empty or blank.")
        }
        */
        let mut data: Vec<u8> = Vec::new();
        data.push(KEY_RECORD);
        data.extend(MetadataAddress::uuid_as_byte_array(scope_uuid));
        data.extend(MetadataAddress::as_hashed_bytes(record_name));
        MetadataAddress {
            bytes: data
        }
    }

    pub fn for_scope_specification(scope_spec_uuid: Uuid) -> Self {
        let mut data: Vec<u8> = Vec::new();
        data.push(KEY_SCOPE_SPECIFICATION);
        data.extend(MetadataAddress::uuid_as_byte_array(scope_spec_uuid));
        MetadataAddress {
            bytes: data
        }
    }

    pub fn for_contract_specification(contract_spec_uuid: Uuid) -> Self {
        let mut data: Vec<u8> = Vec::new();
        data.push(KEY_CONTRACT_SPECIFICATION);
        data.extend(MetadataAddress::uuid_as_byte_array(contract_spec_uuid));
        MetadataAddress {
            bytes: data
        }
    }

    pub fn for_record_specification(contract_spec_uuid: Uuid, record_spec_name: String) -> Self {
        /* TODO
        if (recordSpecName.isBlank()) {
            throw IllegalArgumentException("Invalid recordSpecName: cannot be empty or blank.")
        }
        */
        let mut data: Vec<u8> = Vec::new();
        data.push(KEY_RECORD_SPECIFICATION);
        data.extend(MetadataAddress::uuid_as_byte_array(contract_spec_uuid));
        data.extend(MetadataAddress::as_hashed_bytes(record_spec_name));
        MetadataAddress {
            bytes: data
        }
    }

    pub fn from_bech32(bech32_value: String) -> Self {
        let (hrp, data5, _variant) = bech32::decode(&*bech32_value).unwrap();
        let data = Vec::<u8>::from_base32(&data5).unwrap();
        MetadataAddress::validate_bytes(&data);
        let prefix = MetadataAddress::get_prefix_from_key(data[0]);
        if hrp != prefix {
            /* TODO
            throw IllegalArgumentException("Incorrect HRP: Expected ${prefix}, Actual: ${hrp}.")
            */
        }
        MetadataAddress {
            bytes: data
        }
    }

    fn uuid_as_byte_array(uuid: Uuid) -> Vec<u8> {
        uuid.as_bytes().to_vec()
    }

    fn byte_array_as_uuid(data: Vec<u8>) -> Uuid {
        Uuid::from_bytes(data.try_into().unwrap())
        // TODO: .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
    }

    fn as_hashed_bytes(string: String) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(string.to_lowercase().as_bytes().to_vec());
        let mut hashed_bytes = hasher.finalize().to_vec();
        hashed_bytes.truncate(16);
        hashed_bytes
    }

    fn get_prefix_from_key(key: u8) -> String {
        match key {
            KEY_SCOPE => PREFIX_SCOPE.to_string(),
            KEY_SESSION => PREFIX_SESSION.to_string(),
            KEY_RECORD => PREFIX_RECORD.to_string(),
            KEY_SCOPE_SPECIFICATION => PREFIX_SCOPE_SPECIFICATION.to_string(),
            KEY_CONTRACT_SPECIFICATION => PREFIX_CONTRACT_SPECIFICATION.to_string(),
            KEY_RECORD_SPECIFICATION => PREFIX_RECORD_SPECIFICATION.to_string(),
            _ => {
                /* TODO
                throw IllegalArgumentException("Invalid key: $key")
                */
                "".to_string()
            }
        }
    }

    fn validate_bytes(bytes: &Vec<u8>) {
        let expected_length = match bytes[0] {
            KEY_SCOPE => 17,
            KEY_SESSION => 33,
            KEY_RECORD => 33,
            KEY_SCOPE_SPECIFICATION => 17,
            KEY_CONTRACT_SPECIFICATION => 17,
            KEY_RECORD_SPECIFICATION => 33,
            _ => {
                /* TODO
                throw IllegalArgumentException("Invalid key: ${bytes[0]}")
                */
                0
            }
        };

        if expected_length != bytes.len() {
            /* TODO
            throw IllegalArgumentException("Incorrect data length for type ${getPrefixFromKey(bytes[0])}: Expected ${expectedLength}, Actual: ${bytes.size}.")
            */
        }
    }

    pub fn get_key(&self) -> u8 {
        self.bytes[0]
    }

    pub fn get_prefix(&self) -> String {
        MetadataAddress::get_prefix_from_key(self.get_key())
    }

    pub fn get_primary_uuid(&self) -> Uuid {
        MetadataAddress::byte_array_as_uuid(self.bytes.get(1..17).unwrap().to_vec())
    }

    pub fn get_secondary_bytes(&self) -> Vec<u8> {
        if self.bytes.len() <= 17 {
            vec![]
        } else {
            self.bytes.get(17..self.bytes.len()).unwrap().to_vec()
        }
    }

}

impl ToString for MetadataAddress {

    fn to_string(&self) -> String {
        bech32::encode(&*self.get_prefix(), self.bytes.to_base32(), Variant::Bech32).unwrap()
    }

}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use crate::utils::MetadataAddress;
    use crate::utils::vec_contains;
    use uuid::Uuid;

    const SCOPE_UUID: &str = "d1f0a3a5-c1c2-4f8e-a8c1-416e102d0520";
    const SCOPE_BECH32: &str = "scope1qrglpga9c8pylr4gc9qkuypdq5sqph649l";

    const SESSION_UUID: &str = "73b477e1-dfeb-4709-88cf-b0eb80830d3c";
    const SESSION_BECH32: &str = "session1q8glpga9c8pylr4gc9qkuypdq5s88drhu807k3cf3r8mp6uqsvxnckjeje7";

    const RECORD_UUID: &str = "518a6d56-01e4-4b81-9b80-848b42b97735";
    const RECORD_NAME: &str = "TestRecordName";
    const RECORD_BECH32: &str = "record1qfgc5m2kq8jyhqvmszzgks4ewu6edl6jlsvuseqr2lxusdwfutjggpy33s7";
    const RECORD_NAME_SHA256: [u8; 16] = [0x96, 0xff, 0x52, 0xfc, 0x19, 0xc8, 0x64, 0x03, 0x57, 0xcd, 0xc8, 0x35, 0xc9, 0xe2, 0xe4, 0x84];

    const SCOPE_SPEC_UUID: &str = "2e0222fc-901d-458a-aa21-604c14872e53";
    const SCOPE_SPEC_BECH32: &str = "scopespec1qshqyghujqw5tz42y9syc9y89efs4rmd74";

    const CONTRACT_SPEC_UUID: &str = "dd53a634-002b-40f8-926c-ac888d20f881";
    const CONTRACT_SPEC_BECH32: &str = "contractspec1q0w48f35qq45p7yjdjkg3rfqlzqs0q4jgj";

    const RECORD_SPEC_UUID: &str = "535b5c1c-bc5f-4d22-bfdd-1158828a3383";
    const RECORD_SPEC_NAME: &str = "TestRecordSpecName";
    const RECORD_SPEC_BECH32: &str = "recspec1q4f4khquh3056g4lm5g43q52xwps3at0tsx4rey0vds0663kl72uwkq2vex";
    const RECORD_SPEC_NAME_SHA256: [u8; 16] = [0x08, 0xf5, 0x6f, 0x5c, 0x0d, 0x51, 0xe4, 0x8f, 0x63, 0x60, 0xfd, 0x6a, 0x36, 0xff, 0x95, 0xc7];

    #[test]
    pub fn metadata_address_for_scope() {
        let scope_addr = MetadataAddress::for_scope(Uuid::parse_str(SCOPE_UUID).unwrap());
        let result = scope_addr.to_string();
        match &*result {
            SCOPE_BECH32 => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", SCOPE_BECH32, result),
        }
    }

    #[test]
    pub fn metadata_address_for_session() {
        let session_addr = MetadataAddress::for_session(Uuid::parse_str(SCOPE_UUID).unwrap(), Uuid::parse_str(SESSION_UUID).unwrap());
        let result = session_addr.to_string();
        match &*result {
            SESSION_BECH32 => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", SESSION_BECH32, result),
        }
    }

    #[test]
    pub fn metadata_address_for_record() {
        let record_addr = MetadataAddress::for_record(Uuid::parse_str(RECORD_UUID).unwrap(), RECORD_NAME.to_string());
        let result = record_addr.to_string();
        match &*result {
            RECORD_BECH32 => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", RECORD_BECH32, result),
        }
    }

    #[test]
    pub fn metadata_address_for_scope_specification() {
        let scope_spec_addr = MetadataAddress::for_scope_specification(Uuid::parse_str(SCOPE_SPEC_UUID).unwrap());
        let result = scope_spec_addr.to_string();
        match &*result {
            SCOPE_SPEC_BECH32 => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", SCOPE_SPEC_BECH32, result),
        }
    }

    #[test]
    pub fn metadata_address_for_contract_specification() {
        let contract_spec_addr = MetadataAddress::for_contract_specification(Uuid::parse_str(CONTRACT_SPEC_UUID).unwrap());
        let result = contract_spec_addr.to_string();
        match &*result {
            CONTRACT_SPEC_BECH32 => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", CONTRACT_SPEC_BECH32, result),
        }
    }

    #[test]
    pub fn metadata_address_for_record_specification() {
        let record_spec_addr = MetadataAddress::for_record_specification(Uuid::parse_str(RECORD_SPEC_UUID).unwrap(), RECORD_SPEC_NAME.to_string());
        let result = record_spec_addr.to_string();
        match &*result {
            RECORD_SPEC_BECH32 => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", RECORD_SPEC_BECH32, result),
        }
    }

    #[test]
    pub fn metadata_address_for_scope_from_bech32() {
        let scope_addr = MetadataAddress::from_bech32(SCOPE_BECH32.to_string());
        let scope_uuid = scope_addr.get_primary_uuid().to_hyphenated().to_string().to_lowercase();
        match &*scope_uuid {
            SCOPE_UUID => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", SCOPE_UUID, scope_uuid),
        }
    }

    #[test]
    pub fn metadata_address_for_session_from_bech32() {
        let session_addr = MetadataAddress::from_bech32(SESSION_BECH32.to_string());
        let scope_uuid = session_addr.get_primary_uuid().to_hyphenated().to_string().to_lowercase();
        match &*scope_uuid {
            SCOPE_UUID => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", SCOPE_UUID, scope_uuid),
        }
        let session_uuid = Uuid::from_bytes(session_addr.get_secondary_bytes().try_into().unwrap()).to_hyphenated().to_string().to_lowercase();
        match &*session_uuid {
            SESSION_UUID => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", SESSION_UUID, session_uuid),
        }
    }

    #[test]
    pub fn metadata_address_for_record_from_bech32() {
        let record_addr = MetadataAddress::from_bech32(RECORD_BECH32.to_string());
        let record_uuid = record_addr.get_primary_uuid().to_hyphenated().to_string().to_lowercase();
        match &*record_uuid {
            RECORD_UUID => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", RECORD_UUID, record_uuid),
        }
        let record_name_sha256 = record_addr.get_secondary_bytes();
        if !vec_contains(&*record_name_sha256, &RECORD_NAME_SHA256) {
            panic!("unexpected error: expected {:?} got {:?}", RECORD_NAME_SHA256, record_name_sha256)
        }
    }

    #[test]
    pub fn metadata_address_for_scope_specification_from_bech32() {
        let scope_spec_addr = MetadataAddress::from_bech32(SCOPE_SPEC_BECH32.to_string());
        let scope_spec_uuid = scope_spec_addr.get_primary_uuid().to_hyphenated().to_string().to_lowercase();
        match &*scope_spec_uuid {
            SCOPE_SPEC_UUID => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", SCOPE_SPEC_UUID, scope_spec_uuid),
        }
    }

    #[test]
    pub fn metadata_address_for_contract_specification_from_bech32() {
        let contract_spec_addr = MetadataAddress::from_bech32(CONTRACT_SPEC_BECH32.to_string());
        let contract_spec_uuid = contract_spec_addr.get_primary_uuid().to_hyphenated().to_string().to_lowercase();
        match &*contract_spec_uuid {
            CONTRACT_SPEC_UUID => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", CONTRACT_SPEC_UUID, contract_spec_uuid),
        }
    }

    #[test]
    pub fn metadata_address_for_record_specification_from_bech32() {
        let record_spec_addr = MetadataAddress::from_bech32(RECORD_SPEC_BECH32.to_string());
        let record_spec_uuid = record_spec_addr.get_primary_uuid().to_hyphenated().to_string().to_lowercase();
        match &*record_spec_uuid {
            RECORD_SPEC_UUID => {}
            _ => panic!("unexpected error: expected {:?} got {:?}", RECORD_SPEC_UUID, record_spec_uuid),
        }
        let record_spec_name_sha256 = record_spec_addr.get_secondary_bytes();
        if !vec_contains(&*record_spec_name_sha256, &RECORD_SPEC_NAME_SHA256) {
            panic!("unexpected error: expected {:?} got {:?}", RECORD_SPEC_NAME_SHA256, record_spec_name_sha256)
        }
    }
}
