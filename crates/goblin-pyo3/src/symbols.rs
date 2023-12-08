use goblin::mach::symbols::Nlist;
use pyo3::prelude::*;

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


