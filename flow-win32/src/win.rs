use std::cell::RefCell;
use std::rc::Rc;

use flow_core::address::Address;
use flow_core::mem::VirtualRead;

use crate::kernel::StartBlock;

pub mod types;
pub mod unicode_string;

pub mod module;
pub mod process;

use process::ProcessIterator;

// TODO: cache processes somewhat?
pub struct Windows<T: VirtualRead> {
    pub mem: Rc<RefCell<T>>,

    pub start_block: StartBlock,
    pub kernel_base: Address,
    pub eprocess_base: Address,

    pub kernel_pdb: Option<Rc<RefCell<types::PDB>>>,
}

impl<T: VirtualRead> Clone for Windows<T>
where
    Rc<RefCell<T>>: Clone,
    StartBlock: Clone,
    Address: Clone,
    Option<types::PDB>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            mem: self.mem.clone(),
            start_block: self.start_block,
            kernel_base: self.kernel_base,
            eprocess_base: self.eprocess_base,
            kernel_pdb: self.kernel_pdb.clone(),
        }
    }
}

impl<T: VirtualRead> Windows<T> {
    pub fn process_iter(&self) -> ProcessIterator<T> {
        let rc = Rc::new(RefCell::new(self.clone()));
        ProcessIterator::new(rc)
    }
}
