use std::{
    fs::File,
    io::Read,
};

use goblin::{
    mach::{symbols::Nlist, Mach},
};
use pyo3::{prelude::*, exceptions::PyTypeError};

#[pyclass]
struct Object {
    len: usize,
    ptr: *mut u8,
    inner: Option<goblin::Object<'static>>,
}

// SAFETY: We only use `ptr` in `drop` to reconstruct a `Vec`
unsafe impl Send for Object {}

#[pymethods]
impl Object {
    #[new]
    fn new(path: String) -> Self {
        let mut file = File::open(path).unwrap();
        let size = file.metadata().map(|m| m.len() as usize).ok();
        let mut vec = Vec::with_capacity(size.unwrap_or(0));
        file.read_to_end(&mut vec).unwrap();

        vec.shrink_to_fit();
        let len = vec.len();
        let cap = vec.capacity();
        let ptr = vec.as_mut_ptr();
        assert!(len == cap);

        let obj = vec.leak();
        let object = goblin::Object::parse(obj).unwrap();

        Self {
            len,
            ptr,
            inner: Some(object),
        }
    }

    #[getter]
    fn header(&self) -> Header {
        match self.inner.as_ref().unwrap() {
            goblin::Object::Mach(Mach::Binary(macho)) => Header::from(macho.header),
            _ => unimplemented!(),
        }
    }

    #[getter]
    fn name(&self) -> Option<&str> {
        match self.inner.as_ref().unwrap() {
            goblin::Object::Mach(Mach::Binary(macho)) => macho.name.clone(),
            _ => unimplemented!(),
        }
    }

    fn symbols(&self) -> Symbols {
        match self.inner.as_ref().unwrap() {
            goblin::Object::Mach(Mach::Binary(macho)) => Symbols::from(macho.symbols()),
            _ => unimplemented!(),
        }
    }

    #[getter]
    fn libs(&self) -> Vec<&str> {
        match self.inner.as_ref().unwrap() {
            goblin::Object::Mach(Mach::Binary(macho)) => macho.libs.clone(),
            _ => unimplemented!(),
        }
    }

    #[getter]
    fn rpaths(&self) -> Vec<&str> {
        match self.inner.as_ref().unwrap() {
            goblin::Object::Mach(Mach::Binary(macho)) => macho.rpaths.clone(),
            _ => unimplemented!(),
        }
    }

    fn exports(&self) -> Result<Vec<Export>, PyErr> {
        match self.inner.as_ref().unwrap() {
            goblin::Object::Mach(Mach::Binary(macho)) => {
                let exports = macho.exports().map_err(|_| PyErr::new::<PyTypeError, _>("failed"))?;
                Ok(exports.into_iter().map(|exp| exp.into()).collect())
            },
            _ => unimplemented!(),
        }
    }

    fn imports(&self) -> Result<Vec<Import>, PyErr> {
        match self.inner.as_ref().unwrap() {
            goblin::Object::Mach(Mach::Binary(macho)) => {
                let imports = macho.imports().map_err(|_| PyErr::new::<PyTypeError, _>("failed"))?;
                Ok(imports.into_iter().map(|exp| exp.into()).collect())
            },
            _ => unimplemented!(),
        }
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        let obj = self.inner.take();
        drop(obj);

        // SAFETY:
        // We took `ptr` and `len` from the vec earlier,
        // then leaked it to get a stic reference to it which was only held within `self.inner`,
        // which has been dropped above.
        unsafe {
            let vec = Vec::from_raw_parts(self.ptr, self.len, self.len);
            drop(vec);
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass]
struct Header {
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

#[derive(Debug, Clone)]
#[pyclass]
struct Symbol {
    #[pyo3(get)]
    name: String,

    meta: Nlist,
}

#[pymethods]
impl Symbol {
    #[getter]
    fn typ(&self) -> &'static str {
        self.meta.type_str()
    }

    #[getter]
    fn global(&self) -> bool {
        self.meta.is_global()
    }

    #[getter]
    fn weak(&self) -> bool {
        self.meta.is_weak()
    }

    #[getter]
    fn undefined(&self) -> bool {
        self.meta.is_undefined()
    }

    #[getter]
    fn stab(&self) -> bool {
        self.meta.is_stab()
    }

    fn __repr__(&self) -> String {
        format!(
            "Symbol {{ name: {}, global: {}, weak: {}, undefined: {}, stab: {} }}",
            self.name,
            self.global(),
            self.weak(),
            self.undefined(),
            self.stab()
        )
    }
}

#[pyclass]
struct Symbols {
    symbols: Vec<Symbol>,
}

#[pymethods]
impl Symbols {
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<SymbolIter>> {
        let iter = SymbolIter {
            inner: slf.symbols.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }
}

impl From<goblin::mach::symbols::SymbolIterator<'_>> for Symbols {
    fn from(other: goblin::mach::symbols::SymbolIterator) -> Self {
        let symbols = other
            .map(|sym| {
                let (symname, meta) = sym.unwrap();
                Symbol {
                    name: symname.to_string(),
                    meta,
                }
            })
            .collect();

        Symbols { symbols }
    }
}

#[pyclass]
struct SymbolIter {
    inner: std::vec::IntoIter<Symbol>,
}

#[pymethods]
impl SymbolIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Symbol> {
        slf.inner.next()
    }
}

#[derive(Debug, Default, Clone)]
#[pyclass]
enum ExportTyp {
    #[default]
    Regular,
    Reexport,
    Stub
}

#[derive(Debug, Default, Clone)]
#[pyclass]
pub struct ExportInfo {
    #[pyo3(get)]
    typ: ExportTyp,
    #[pyo3(get)]
    address: u64,
    #[pyo3(get)]
    flags: u64,
    #[pyo3(get)]
    lib: String,
    #[pyo3(get)]
    lib_symbol_name: Option<String>,
}

impl From<goblin::mach::exports::ExportInfo<'_>> for ExportInfo {
    fn from(info: goblin::mach::exports::ExportInfo) -> Self {
        use goblin::mach::exports::ExportInfo::*;
        match info {
            Regular { address, flags } => {
                Self {
                    typ: ExportTyp::Regular,
                    address,
                    flags,
                    .. Default::default()
                }
            }
            Reexport { lib, lib_symbol_name, flags } => {
                Self {
                    typ: ExportTyp::Reexport,
                    lib: lib.to_string(),
                    lib_symbol_name: lib_symbol_name.map(|s| s.to_string()),
                    flags,
                    .. Default::default()
                }
            }
            Stub {
                flags, ..
            } => {
                Self {
                    typ: ExportTyp::Stub,
                    flags,
                    .. Default::default()
                }
            }
        }
    }
}

#[derive(Debug)]
#[pyclass]
struct Export {
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

#[derive(Debug)]
#[pyclass]
struct Import {
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

#[pymodule]
#[pyo3(name = "goblin")]
fn py_goblin(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Object>()?;
    Ok(())
}
