use crypto::digest::Digest;
use crypto::sha2::Sha512;
use log::info;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io::Cursor;

use cbor::encoder::GenericEncoder;
use cbor::value::Key;
use cbor::value::Text;
use cbor::value::Value;

use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::handler::TransactionContext;
use sawtooth_sdk::processor::handler::TransactionHandler;

pub fn prefix() -> String {
    let mut hasher = Sha512::new();
    hasher.input_str("restroom");
    hasher.result_str()[..6].to_string()
}

struct HashPayload {
    value: String,
    address: String,
}

impl HashPayload {
    pub fn new(payload_data: &[u8]) -> Result<HashPayload, ApplyError> {
        let input = Cursor::new(payload_data);

        let mut decoder = cbor::GenericDecoder::new(cbor::Config::default(), input);
        let decoder_value = decoder
            .value()
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;

        let c = cbor::value::Cursor::new(&decoder_value);
        let value: String = match c.field("value").text_plain() {
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Value must be a string",
                )));
            }
            Some(value) => value.clone(),
        };

        let address: String = match c.field("address").text_plain() {
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Address must be a string",
                )));
            }
            Some(address) => address.clone(),
        };

        Ok(HashPayload { value, address })
    }

    pub fn get_value(&self) -> &String {
        &self.value
    }

    pub fn get_address(&self) -> &String {
        &self.address
    }
}

pub struct State<'a> {
    context: &'a mut dyn TransactionContext,
    get_cache: HashMap<String, BTreeMap<Key, Value>>,
}

impl<'a> State<'a> {
    pub fn new(context: &'a mut dyn TransactionContext) -> State {
        State {
            context,
            get_cache: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: &str, value: &str) -> Result<(), ApplyError> {
        let mut map: BTreeMap<Key, Value> = match self.get_cache.get_mut(name) {
            Some(m) => m.clone(),
            None => BTreeMap::new(),
        };
        map.insert(
            Key::Text(Text::Text(String::from(name))),
            Value::Text(Text::Text(String::from(value))),
        );

        let mut e = GenericEncoder::new(Cursor::new(Vec::new()));
        e.value(&Value::Map(map))
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;

        let packed = e.into_inner().into_writer().into_inner();
        self.context
            .set_state_entry(String::from(name), packed)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;

        Ok(())
    }
}

pub struct StateTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl StateTransactionHandler {
    pub fn new() -> StateTransactionHandler {
        StateTransactionHandler {
            family_name: String::from("restroom"),
            family_versions: vec![String::from("1.0")],
            namespaces: vec![prefix()],
        }
    }
}

impl TransactionHandler for StateTransactionHandler {
    fn family_name(&self) -> String {
        self.family_name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family_versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
        self.namespaces.clone()
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut dyn TransactionContext,
    ) -> Result<(), ApplyError> {
        let payload = HashPayload::new(request.get_payload());
        let payload = match payload {
            Err(err) => return Err(err),
            Ok(payload) => payload,
        };

        let mut state = State::new(context);
        info!(
            "payload: {} {} {}",
            payload.get_value(),
            request.get_header().get_inputs()[0],
            request.get_header().get_outputs()[0]
        );

        state.set(payload.get_address(), payload.get_value())
    }
}
