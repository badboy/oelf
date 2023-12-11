use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass]
pub struct Section {
    #[pyo3(get)]
    index: usize,
    #[pyo3(get)]
    name: Option<String>,
    #[pyo3(get)]
    segment: Option<String>,
    #[pyo3(get)]
    addr: u64,
    #[pyo3(get)]
    size: u64,
    #[pyo3(get)]
    offset: u32,
    #[pyo3(get)]
    align: u32,
    #[pyo3(get)]
    reloff: u32,
    #[pyo3(get)]
    nreloc: u32,
    #[pyo3(get)]
    flags: u32,
}

#[pymethods]
impl Section {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl From<(usize, goblin::mach::segment::Section)> for Section {
    fn from((index, section): (usize, goblin::mach::segment::Section)) -> Self {
        Section {
            index,
            name: section.name().ok().map(|s| s.to_string()),
            segment: section.segname().ok().map(|s| s.to_string()),
            addr: section.addr,
            size: section.size,
            offset: section.offset,
            align: section.align,
            reloff: section.reloff,
            nreloc: section.nreloc,
            flags: section.flags,
        }
    }
}

#[derive(Clone)]
#[pyclass]
pub struct Sections {
    pub(crate) sections: Vec<Section>,
}

#[pymethods]
impl Sections {
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<SectionIter>> {
        let iter = SectionIter {
            inner: slf.sections.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }
}

#[pyclass]
struct SectionIter {
    inner: std::vec::IntoIter<Section>,
}

#[pymethods]
impl SectionIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Section> {
        slf.inner.next()
    }
}
