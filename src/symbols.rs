use goblin::mach::symbols::{n_type_to_str, N_EXT, N_STAB, N_TYPE, N_UNDF, N_WEAK_DEF, N_WEAK_REF};
use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass]
struct Nlist {
    #[pyo3(get)]
    n_strx: usize,
    #[pyo3(get)]
    n_type: u8,
    #[pyo3(get)]
    n_sect: usize,
    #[pyo3(get)]
    n_desc: u16,
    #[pyo3(get)]
    n_value: u64,
}

#[pymethods]
impl Nlist {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl Nlist {
    /// Gets this symbol's type in bits 0xe
    pub fn get_type(&self) -> u8 {
        self.n_type & N_TYPE
    }
    /// Gets the str representation of the type of this symbol
    pub fn type_str(&self) -> &'static str {
        n_type_to_str(self.get_type())
    }
    /// Whether this symbol is global or not
    pub fn is_global(&self) -> bool {
        self.n_type & N_EXT != 0
    }
    /// Whether this symbol is weak or not
    pub fn is_weak(&self) -> bool {
        self.n_desc & (N_WEAK_REF | N_WEAK_DEF) != 0
    }
    /// Whether this symbol is undefined or not
    pub fn is_undefined(&self) -> bool {
        self.n_sect == 0 && self.n_type & N_TYPE == N_UNDF
    }
    /// Whether this symbol is a symbolic debugging entry
    pub fn is_stab(&self) -> bool {
        self.n_type & N_STAB != 0
    }
}

impl From<goblin::mach::symbols::Nlist> for Nlist {
    fn from(list: goblin::mach::symbols::Nlist) -> Self {
        Self {
            n_strx: list.n_strx,
            n_type: list.n_type,
            n_sect: list.n_sect,
            n_desc: list.n_desc,
            n_value: list.n_value,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass]
struct Symbol {
    #[pyo3(get)]
    name: String,

    #[pyo3(get)]
    meta: Nlist,
}

#[pymethods]
impl Symbol {
    #[getter]
    fn typ(&self) -> &'static str {
        self.meta.type_str()
    }

    #[getter]
    fn is_global(&self) -> bool {
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

    #[getter]
    fn section(&self) -> usize {
        self.meta.n_sect
    }

    fn __repr__(&self) -> String {
        format!(
            "Symbol {{ name: {}, type: {}, global: {}, weak: {}, undefined: {}, stab: {}, meta: {:?} }}",
            self.name,
            self.typ(),
            self.is_global(),
            self.weak(),
            self.undefined(),
            self.stab(),
            self.meta,
        )
    }
}

#[pyclass]
pub struct Symbols {
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
                    meta: meta.into(),
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
