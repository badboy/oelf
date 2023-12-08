use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass]
pub struct Header {
    #[pyo3(get)]
    magic: u32,
    #[pyo3(get)]
    cputype: u32,
    #[pyo3(get)]
    cpusubtype: u32,
    #[pyo3(get)]
    filetype: u32,
    #[pyo3(get)]
    ncmds: usize,
    #[pyo3(get)]
    sizeofcmds: u32,
    #[pyo3(get)]
    flags: u32,
    #[pyo3(get)]
    reserved: u32,
}

impl From<goblin::mach::header::Header> for Header {
    fn from(other: goblin::mach::header::Header) -> Self {
        Header {
            magic: other.magic,
            cputype: other.cputype,
            cpusubtype: other.cpusubtype,
            filetype: other.filetype,
            ncmds: other.ncmds,
            sizeofcmds: other.sizeofcmds,
            flags: other.flags,
            reserved: other.reserved,
        }
    }
}

#[pymethods]
impl Header {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
