/*!
Example:
  cargo run --release --example ps_inventory -- /tmp/mem.img
*/

use log::info;
use memflow::prelude::v1::*;
use std::env::args;

fn main() {
    env_logger::init();
    let connector_args: ConnectorArgs = args()
        .nth(1)
        .map(|a| str::parse(a.as_ref()).expect("args parse"))
        .expect("need rawmem::<path>");
    let mut conn = memflow_rawmem::create_connector(&connector_args).expect("init");
    let mut buf = [0u8; 16];
    conn.phys_view()
        .read_raw_into(Address::from(0x1000), &mut buf)
        .unwrap();
    info!("bytes: {:02x?}", buf);
}
