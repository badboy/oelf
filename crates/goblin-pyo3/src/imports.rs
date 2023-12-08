use pyo3::prelude::*;

#[derive(Debug)]
#[pyclass]
pub struct Import {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    dylib: String,
    #[pyo3(get)]
    is_lazy: bool,
    #[pyo3(get)]
    offset: u64,
    #[pyo3(get)]
    size: usize,
    #[pyo3(get)]
    address: u64,
    #[pyo3(get)]
    addend: i64,
    #[pyo3(get)]
    is_weak: bool,
    #[pyo3(get)]
    start_of_sequence_offset: u64,
}

#[pymethods]
impl Import {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl From<goblin::mach::imports::Import<'_>> for Import {
    fn from(import: goblin::mach::imports::Import<'_>) -> Self {
        Self {
            name: import.name.to_string(),
            dylib: import.dylib.to_string(),
            is_lazy: import.is_lazy,
            offset: import.offset,
            size: import.size,
            address: import.address,
            addend: import.addend,
            is_weak: import.is_weak,
            start_of_sequence_offset: import.start_of_sequence_offset,
        }
    }
}
