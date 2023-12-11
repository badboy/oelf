use std::{
    fs::{self, File},
    io::Read,
};

use goblin::mach::{Mach, SingleArch};
use pyo3::{exceptions::PyTypeError, prelude::*};

mod exports;
mod header;
mod imports;
mod load_commands;
mod sections;
mod segments;
mod symbols;

use exports::Export;
use header::Header;
use imports::Import;
use load_commands::LoadCommand;
use sections::{Section, Sections};
use segments::Segment;
use symbols::Symbols;

#[pyclass]
struct Object {
    len: usize,
    ptr: *mut u8,
    #[pyo3(get)]
    path: String,
    inner: Option<goblin::mach::MachO<'static>>,
}

// SAFETY: We only use `ptr` in `drop` to reconstruct a `Vec`
unsafe impl Send for Object {}

impl Object {
    fn macho(&self) -> &goblin::mach::MachO {
        self.inner.as_ref().unwrap()
    }
}

#[pymethods]
impl Object {
    #[new]
    fn new(path: String) -> PyResult<Self> {
        let path = fs::canonicalize(path).unwrap();
        let mut file = File::open(&path).unwrap();
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

        let macho = match object {
            goblin::Object::Mach(Mach::Binary(macho)) => macho,
            goblin::Object::Mach(Mach::Fat(march)) => {
                let mut macho = None;
                for arch in &march {
                    let arch = arch.map_err(|_| {
                        PyErr::new::<PyTypeError, _>("cannot parse single arch from Mach-O file")
                    })?;

                    if let SingleArch::MachO(m) = arch {
                        macho = Some(m);
                        break;
                    }
                }

                match macho {
                    Some(macho) => macho,
                    None => return Err(PyErr::new::<PyTypeError, _>("not a macho file")),
                }
            }
            _ => return Err(PyErr::new::<PyTypeError, _>("not a macho file")),
        };

        Ok(Self {
            len,
            ptr,
            path: path.display().to_string(),
            inner: Some(macho),
        })
    }

    #[getter]
    fn header(&self) -> Header {
        self.macho().header.into()
    }

    #[getter]
    fn name(&self) -> Option<&str> {
        self.macho().name
    }

    fn symbols(&self) -> Symbols {
        Symbols::from(self.macho().symbols())
    }

    fn segments(&self) -> Vec<Segment> {
        self.macho()
            .segments
            .into_iter()
            .map(|seg| seg.into())
            .collect()
    }

    fn sections(&self) -> Sections {
        let macho = self.macho();
        let mut sections = vec![];
        let mut idx = 0;
        for sect_iter in macho.segments.sections() {
            sections.extend(sect_iter.map(|section| {
                idx += 1;
                let (sect, _data) = section.unwrap();
                Section::from((idx, sect))
            }));
        }
        Sections { sections }
    }

    #[getter]
    fn libs(&self) -> Vec<&str> {
        self.macho().libs.clone()
    }

    #[getter]
    fn rpaths(&self) -> Vec<&str> {
        self.macho().rpaths.clone()
    }

    fn exports(&self) -> Result<Vec<Export>, PyErr> {
        let macho = self.macho();
        let exports = macho
            .exports()
            .map_err(|_| PyErr::new::<PyTypeError, _>("failed"))?;
        Ok(exports.into_iter().map(|exp| exp.into()).collect())
    }

    fn imports(&self) -> Result<Vec<Import>, PyErr> {
        let macho = self.macho();
        let imports = macho
            .imports()
            .map_err(|_| PyErr::new::<PyTypeError, _>("failed"))?;
        Ok(imports.into_iter().map(|exp| exp.into()).collect())
    }

    fn load_commands(&self) -> Vec<LoadCommand> {
        self.macho()
            .load_commands
            .iter()
            .map(|cmd| cmd.into())
            .collect()
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        let obj = self.inner.take();
        drop(obj);

        // SAFETY:
        // We took `ptr` and `len` from the vec earlier (and ensured `len` == `cap`),
        // then leaked it to get a static reference to it
        // which was only held within `self.inner`, which has been dropped above.
        unsafe {
            let vec = Vec::from_raw_parts(self.ptr, self.len, self.len);
            drop(vec);
        }
    }
}
#[pymodule]
#[pyo3(name = "oelf")]
fn py_goblin(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Object>()?;
    Ok(())
}
