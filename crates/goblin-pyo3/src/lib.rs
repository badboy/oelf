use std::{
    fs::File,
    io::Read,
};

use goblin::mach::Mach;
use pyo3::{prelude::*, exceptions::PyTypeError};

mod exports;
mod header;
mod imports;
mod symbols;

use exports::Export;
use header::Header;
use imports::Import;
use symbols::Symbols;

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
#[pymodule]
#[pyo3(name = "goblin")]
fn py_goblin(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Object>()?;
    Ok(())
}
