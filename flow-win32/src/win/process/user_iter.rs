use super::Windows;

use flow_core::address::Address;
use flow_core::mem::*;

use std::cell::RefCell;
use std::rc::Rc;

use super::UserProcess;

pub struct ProcessIterator<T: VirtualRead> {
    win: Rc<RefCell<Windows<T>>>,
    first_eprocess: Address,
    eprocess: Address,
}

impl<T: VirtualRead> ProcessIterator<T> {
    pub fn new(win: Rc<RefCell<Windows<T>>>) -> Self {
        let eprocess = win.borrow().eprocess_base;
        Self {
            win,
            first_eprocess: eprocess,
            eprocess,
        }
    }
}

impl<T: VirtualRead> Iterator for ProcessIterator<T> {
    type Item = UserProcess<T>;

    fn next(&mut self) -> Option<UserProcess<T>> {
        // is eprocess null (first iter, read err, sysproc)?
        if self.eprocess.is_null() {
            return None;
        }

        // copy memory for the lifetime of this function
        let win = self.win.borrow();
        let start_block = win.start_block;
        let kernel_pdb = &mut win.kernel_pdb.as_ref()?.borrow_mut();

        let memory = &mut win.mem.borrow_mut();

        // resolve offsets
        let _eprocess = kernel_pdb.find_struct("_EPROCESS")?;
        let _eprocess_links = _eprocess.find_field("ActiveProcessLinks")?.offset;

        let _list_entry = kernel_pdb.find_struct("_LIST_ENTRY")?;
        let _list_entry_blink = _list_entry.find_field("Blink")?.offset;

        // read next eprocess entry
        let mut next = VirtualReader::with(&mut **memory, start_block.arch, start_block.dtb)
            .virt_read_addr(self.eprocess + _eprocess_links + _list_entry_blink)
            .unwrap(); // TODO: convert to Option
        if !next.is_null() {
            next -= _eprocess_links;
        }

        // if next process is 'system' again just null it
        if next == self.first_eprocess {
            next = Address::null();
        }

        // return the previous process and set 'next' for next iter
        let cur = self.eprocess;
        self.eprocess = next;

        Some(UserProcess::with(self.win.clone(), cur))
    }
}
