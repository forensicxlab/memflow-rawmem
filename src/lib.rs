use std::fs::OpenOptions;
use std::path::Path;

use log::info;
use memflow::prelude::v1::*;

/// identity map: file offset 0 -> [base .. base + file_len)
fn map_entire_file_with_base(
    file: &std::fs::File,
    base: Address,
) -> Result<MemoryMap<(Address, umem)>> {
    let len = file
        .metadata()
        .map_err(|_| Error(ErrorOrigin::Connector, ErrorKind::Unknown).log_error("stat failed"))?
        .len();
    let mut map = MemoryMap::new();
    map.push_remap(base, len as umem, Address::from(0));
    Ok(map)
}

fn parse_base_arg(s: &str) -> Option<u64> {
    let t = s.trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        t.parse::<u64>().ok()
    }
}

#[cfg(feature = "filemap")]
pub type MemRawRo<'a> = ReadMappedFilePhysicalMemory<'a>;
#[cfg(not(feature = "filemap"))]
pub type MemRawRo<'a> = FileIoMemory<CloneFile>;

#[connector(name = "rawmem", help_fn = "help", no_default_cache = true)]
pub fn create_connector<'a>(args: &ConnectorArgs) -> Result<MemRawRo<'a>> {
    let target = args.target.as_deref().ok_or_else(|| {
        Error(ErrorOrigin::Connector, ErrorKind::Unknown).log_error("`target` missing")
    })?;

    let base = args
        .extra_args
        .get("base")
        .and_then(|s| parse_base_arg(s))
        .map(Address::from)
        .unwrap_or(Address::from(0));

    let file = OpenOptions::new()
        .read(true)
        .open(Path::new(target))
        .map_err(|_| Error(ErrorOrigin::Connector, ErrorKind::Unknown).log_error("open failed"))?;

    let map = map_entire_file_with_base(&file, base)?;

    #[cfg(feature = "filemap")]
    {
        let conn = MmapInfo::try_with_filemap(file, map)?.into_connector(); // ReadMappedFilePhysicalMemory
        info!("rawmem: '{}' (RO mmap) base={:#x}", target, base.to_umem());
        Ok(conn)
    }
    #[cfg(not(feature = "filemap"))]
    {
        let conn = MemRawRo::with_mem_map(file.into(), map)?;
        info!("rawmem: '{}' (RO stdio) base={:#x}", target, base.to_umem());
        Ok(conn)
    }
}

pub fn help() -> String {
    "\
The rawmem connector exposes a raw memory image as physical memory (read-only).

Args:
  target  - path to the raw image (required)
  base    - optional physical base (hex like 0x100000000 or decimal)

Examples:
  rawmem::/path/to/mem.img
  rawmem::/path/to/mem.img:base=0x100000000
"
    .to_owned()
}

/// API: RO/RW via a Rust flag
/// Could be usefull for users who import the crate directly (not via plugin).
/// Props: @segfault

enum Inner<'a> {
    Ro(ReadMappedFilePhysicalMemory<'a>),
    Rw(FileIoMemory<CloneFile>),
}

pub struct MemRaw<'a> {
    inner: Inner<'a>,
}

impl<'a> MemRaw<'a> {
    pub fn open<P: AsRef<Path>>(path: P, base: Address, writable: bool) -> Result<Self> {
        let path = path.as_ref();
        let len = std::fs::metadata(path)
            .map_err(|_| {
                Error(ErrorOrigin::Connector, ErrorKind::Unknown).log_error("stat failed")
            })?
            .len();
        let mut map = MemoryMap::new();
        map.push_remap(base, len as umem, Address::from(0));

        if writable {
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map_err(|_| {
                    Error(ErrorOrigin::Connector, ErrorKind::Unknown).log_error("open (rw) failed")
                })?;
            let io = FileIoMemory::with_mem_map(f.into(), map)?;
            Ok(Self {
                inner: Inner::Rw(io),
            })
        } else {
            let f = OpenOptions::new().read(true).open(path).map_err(|_| {
                Error(ErrorOrigin::Connector, ErrorKind::Unknown).log_error("open (ro) failed")
            })?;
            let ro = MmapInfo::try_with_filemap(f, map)?.into_connector();
            Ok(Self {
                inner: Inner::Ro(ro),
            })
        }
    }
}

impl<'a> PhysicalMemory for MemRaw<'a> {
    fn phys_read_raw_iter(
        &mut self,
        ops: memflow::mem::mem_data::PhysicalReadMemOps,
    ) -> Result<()> {
        match &mut self.inner {
            Inner::Ro(m) => m.phys_read_raw_iter(ops),
            Inner::Rw(m) => m.phys_read_raw_iter(ops),
        }
    }
    fn phys_write_raw_iter(
        &mut self,
        ops: memflow::mem::mem_data::PhysicalWriteMemOps,
    ) -> Result<()> {
        match &mut self.inner {
            Inner::Ro(m) => m.phys_write_raw_iter(ops),
            Inner::Rw(m) => m.phys_write_raw_iter(ops),
        }
    }
    fn metadata(&self) -> memflow::mem::phys_mem::PhysicalMemoryMetadata {
        match &self.inner {
            Inner::Ro(m) => m.metadata(),
            Inner::Rw(m) => m.metadata(),
        }
    }
}
