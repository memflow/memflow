//! Remapping layer for a memory view.
use super::*;

/// Remapped memory view.
///
/// This structure allows to build a new memory view as a subset of an existing view.
///
/// This is useful for nested VM introspection, or analyzing emulators and custom memory
/// structures.
#[derive(Clone)]
pub struct RemapView<T: MemoryView> {
    mem: T,
    mem_map: MemoryMap<(Address, umem)>,
}

impl<T: MemoryView> RemapView<T> {
    pub fn new(mem: T, mem_map: MemoryMap<(Address, umem)>) -> Self {
        Self { mem, mem_map }
    }
}

impl<T: MemoryView> MemoryView for RemapView<T> {
    fn read_raw_iter<'a>(
        &mut self,
        data: CIterator<ReadData<'a>>,
        out_fail: &mut ReadFailCallback<'_, 'a>,
    ) -> Result<()> {
        let out_fail = std::cell::RefCell::new(out_fail);

        let mut void = |(addr, buf): (Address, _)| out_fail.borrow_mut().call(MemData(addr, buf));
        let mut out_fail = |data| out_fail.borrow_mut().call(data);

        let mut iter = self
            .mem_map
            .map_base_iter(data.map(<_>::into), &mut void)
            .map(|((a, _), b)| MemData(a, b));

        self.mem
            .read_raw_iter((&mut iter).into(), &mut (&mut out_fail).into())
    }

    fn write_raw_iter<'a>(
        &mut self,
        data: CIterator<WriteData<'a>>,
        out_fail: &mut WriteFailCallback<'_, 'a>,
    ) -> Result<()> {
        let out_fail = std::cell::RefCell::new(out_fail);

        let mut void = |(addr, buf): (Address, _)| out_fail.borrow_mut().call(MemData(addr, buf));
        let mut out_fail = |data| out_fail.borrow_mut().call(data);

        let mut iter = self
            .mem_map
            .map_base_iter(data.map(<_>::into), &mut void)
            .map(|((a, _), b)| MemData(a, b));

        self.mem
            .write_raw_iter((&mut iter).into(), &mut (&mut out_fail).into())
    }

    fn metadata(&self) -> MemoryViewMetadata {
        MemoryViewMetadata {
            max_address: self.mem_map.max_address(),
            real_size: self.mem_map.real_size(),
            ..self.mem.metadata()
        }
    }
}
