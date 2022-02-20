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
    fn read_raw_iter(&mut self, MemOps { inp, out_fail, out }: ReadRawMemOps) -> Result<()> {
        let out_fail = out_fail.map(std::cell::RefCell::new);

        let mut out_fail1 = out_fail
            .as_ref()
            .map(|of| move |data| of.borrow_mut().call(data));
        let mut out_fail2 = out_fail
            .as_ref()
            .map(|of| move |data| of.borrow_mut().call(data));
        let mut out_fail2 = out_fail2.as_mut().map(<_>::into);
        let out_fail2 = out_fail2.as_mut();

        let mut out = out.map(|o| move |data| o.call(data));
        let mut out = out.as_mut().map(<_>::into);
        let out = out.as_mut();

        let mem_map = &mut self.mem_map;
        let mem = &mut self.mem;

        let iter = mem_map
            .map_base_iter(inp, out_fail1.as_mut())
            .map(|CTup3((a, _), m, b)| CTup3(a, m, b));

        MemOps::with_raw(iter, out, out_fail2, |data| mem.read_raw_iter(data))
    }

    fn write_raw_iter(&mut self, MemOps { inp, out_fail, out }: WriteRawMemOps) -> Result<()> {
        let out_fail = out_fail.map(std::cell::RefCell::new);

        let mut out_fail1 = out_fail
            .as_ref()
            .map(|of| move |data| of.borrow_mut().call(data));
        let mut out_fail2 = out_fail
            .as_ref()
            .map(|of| move |data| of.borrow_mut().call(data));
        let mut out_fail2 = out_fail2.as_mut().map(<_>::into);
        let out_fail2 = out_fail2.as_mut();

        let mut out = out.map(|o| move |data| o.call(data));
        let mut out = out.as_mut().map(<_>::into);
        let out = out.as_mut();

        let mem_map = &mut self.mem_map;
        let mem = &mut self.mem;

        let iter = mem_map
            .map_base_iter(inp, out_fail1.as_mut())
            .map(|CTup3((a, _), m, b)| CTup3(a, m, b));

        MemOps::with_raw(iter, out, out_fail2, |data| mem.write_raw_iter(data))
    }

    fn metadata(&self) -> MemoryViewMetadata {
        MemoryViewMetadata {
            max_address: self.mem_map.max_address(),
            real_size: self.mem_map.real_size(),
            ..self.mem.metadata()
        }
    }
}
