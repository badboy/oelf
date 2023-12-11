use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass]
pub struct LoadCommand {
    #[pyo3(get)]
    offset: usize,
    #[pyo3(get)]
    command: String,
}

#[pymethods]
impl LoadCommand {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl From<&goblin::mach::load_command::LoadCommand> for LoadCommand {
    fn from(lcmd: &goblin::mach::load_command::LoadCommand) -> Self {
        LoadCommand {
            offset: lcmd.offset,
            command: format!("{:?}", lcmd.command),
        }
    }
}
