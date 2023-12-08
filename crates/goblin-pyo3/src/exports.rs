use pyo3::prelude::*;

#[derive(Debug, Default, Clone)]
#[pyclass]
enum ExportTyp {
    #[default]
    Regular,
    Reexport,
    Stub,
}

#[pymethods]
impl ExportTyp {
    fn __str__(&self) -> &'static str {
        match self {
            ExportTyp::Regular => "regular",
            ExportTyp::Reexport => "reexport",
            ExportTyp::Stub => "stub",
        }
    }
}

#[derive(Debug, Default, Clone)]
#[pyclass]
struct ExportInfo {
    #[pyo3(get)]
    typ: ExportTyp,
    #[pyo3(get)]
    address: u64,
    #[pyo3(get)]
    flags: u64,
    #[pyo3(get)]
    lib: Option<String>,
    #[pyo3(get)]
    lib_symbol_name: Option<String>,
}

impl From<goblin::mach::exports::ExportInfo<'_>> for ExportInfo {
    fn from(info: goblin::mach::exports::ExportInfo) -> Self {
        use goblin::mach::exports::ExportInfo::*;
        match info {
            Regular { address, flags } => Self {
                typ: ExportTyp::Regular,
                address,
                flags,
                ..Default::default()
            },
            Reexport {
                lib,
                lib_symbol_name,
                flags,
            } => Self {
                typ: ExportTyp::Reexport,
                lib: Some(lib.to_string()),
                lib_symbol_name: lib_symbol_name.map(|s| s.to_string()),
                flags,
                ..Default::default()
            },
            Stub { flags, .. } => Self {
                typ: ExportTyp::Stub,
                flags,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug)]
#[pyclass]
pub struct Export {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    info: ExportInfo,
    #[pyo3(get)]
    size: usize,
    #[pyo3(get)]
    offset: u64,
}

#[pymethods]
impl Export {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl From<goblin::mach::exports::Export<'_>> for Export {
    fn from(export: goblin::mach::exports::Export) -> Self {
        Self {
            name: export.name,
            info: export.info.into(),
            size: export.size,
            offset: export.offset,
        }
    }
}
