use web3;

use std::str::FromStr;
use web3::types::*;
use web3::contract::Contract;
use web3::futures::Future;
use tiny_keccak::Keccak;
use std::convert::From;

pub struct RegistryInstance<T: web3::Transport> {
    instance: Contract<T>
}

impl<T: web3::Transport> RegistryInstance<T> {
    //
    pub fn new(web3: & web3::Web3<T>) -> RegistryInstance<T> {
        const REGISTRY_ADDR: &str = "0x8009a230dc908e71befafba36e09efef2513640d";

        let instance = Contract::from_json(
            web3.eth(),
            H160::from_str(REGISTRY_ADDR).unwrap(), //TODO:Make static
            include_bytes!("../Registry.json")).unwrap();

        RegistryInstance {
            instance
        }
    }
    
    //returns true if the domain passed in is in the adchain registry
    pub fn is_in_registry(&self, domain: &str) -> bool {
        let my_account: Address = "0x494b26d0fea32296d5b1d011b2c1f95cb8e1d175".parse().unwrap();

        let mut sha3 = Keccak::new_keccak256();
        let data: Vec<u8> = From::from(domain);

        sha3.update(&data);
        let mut array: [u8; 32] = [0; 32];
        sha3.finalize(&mut array);
    
        let hash = H256(array);

        // let domain = String::from(domain).into_token();

        let result: bool = match self.instance
            .query("isWhitelisted", 
            (hash, ), 
            Some(my_account),
            web3::contract::Options::default(), 
            Some(BlockNumber::Latest)).wait() {
            Ok(result) => result,
            Err(err) => panic!("Network was unreachable! {:?}", err),
        };
        result
    }
}
