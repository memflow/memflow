use crate::error::Result;

use std::rc::Rc;
use std::cell::RefCell;
use log::{debug, trace};

use address::{Address, Length};
use mem::{VirtualRead};

use crate::kernel::StartBlock;

pub mod types;
pub mod process;

use process::{Process, ProcessIterator};

// TODO: cache processes somewhat?
#[derive(Clone)]
pub struct Windows<T: VirtualRead> {
    pub mem: Rc<RefCell<T>>,

    pub start_block: StartBlock,
    pub kernel_base: Address,
    pub eprocess_base: Address,

    // TODO: refcell + shared access?
    pub kernel_pdb: Option<types::PDB>,
}

impl<T: VirtualRead> Windows<T> {
    pub fn process_iter(&mut self) -> ProcessIterator<'_, T> {
        ProcessIterator::new(self)
    }
}
