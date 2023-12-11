use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass]
pub struct Segment {
    #[pyo3(get)]
    cmd: u32,
    #[pyo3(get)]
    cmdsize: u32,
    #[pyo3(get)]
    name: Option<String>,
    #[pyo3(get)]
    vmaddr: u64,
    #[pyo3(get)]
    vmsize: u64,
    #[pyo3(get)]
    fileoff: u64,
    #[pyo3(get)]
    filesize: u64,
    #[pyo3(get)]
    maxprot: u32,
    #[pyo3(get)]
    initprot: u32,
    #[pyo3(get)]
    nsects: u32,
    #[pyo3(get)]
    flags: u32,
}

#[pymethods]
impl Segment {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl From<&goblin::mach::segment::Segment<'_>> for Segment {
    fn from(segm: &goblin::mach::segment::Segment) -> Self {
        let segname = segm.name().ok().map(|s| s.to_string());
        Segment {
            cmd: segm.cmd,
            cmdsize: segm.cmdsize,
            name: segname,
            vmaddr: segm.vmaddr,
            vmsize: segm.vmsize,
            fileoff: segm.fileoff,
            filesize: segm.filesize,
            maxprot: segm.maxprot,
            initprot: segm.initprot,
            nsects: segm.nsects,
            flags: segm.flags,
        }
    }
}
