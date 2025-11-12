use memflow::prelude::v1::*;

#[os(name = "rawmem", return_wrapped = true)]
pub fn create_os(args: &OsArgs, lib: LibArc) -> Result<OsInstanceArcBox<'static>> {
    todo!()
}
