pub mod cashu;
pub mod vault;

pub static BOARD_SIZE: u8 = 4;

use configparser::ini::Ini;

pub fn load_conf() -> (String, String, String, String) {
    let mut config = Ini::new();
    let _map = config
        .load("./config.cfg")
        .expect("config.cfg does not exist! please copy config_example.cfg");

    let address = config.get("lnd", "address").expect("address provided");
    let cert = config.get("lnd", "certfile").expect("cert provided");
    let macaroon = config
        .get("lnd", "macaroonfile")
        .expect("macaroon provided");
    let label = config
        .get("lnd", "defaultlabel")
        .expect("default label provided");
    (address, cert, macaroon, label)
}
