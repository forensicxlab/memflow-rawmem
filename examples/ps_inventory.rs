/*!
Example:
  cargo run --release --example ps_inventory -- /tmp/mem.img
*/
use log::info;
use memflow::prelude::v1::*;
use std::env::args;

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let connector_args = args()
        .nth(1)
        .map(|arg| str::parse(arg.as_ref()).expect("unable to parse connector args"));

    let mut inventory = Inventory::scan();
    let connector = inventory
        .instantiate_connector("rawmem", None, connector_args.as_ref())
        .expect("unable to create rawmem connector");
    let mut os = inventory
        .instantiate_os("win32", Some(connector), None)
        .expect("unable to create win32 instance with rawmem connector");
    let process_list = os.process_info_list().expect("unable to read process list");

    info!(
        "{:>5} {:>10} {:>10} {:<}",
        "PID", "SYS ARCH", "PROC ARCH", "NAME"
    );
    for p in process_list {
        info!(
            "{:>5} {:^10} {:^10} {}",
            p.pid, p.sys_arch, p.proc_arch, p.name
        );
    }
}
