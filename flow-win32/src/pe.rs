use crate::error::Result;

use log::info;

use address::{Address, Length};
use mem::{PhysicalRead, VirtualRead};

use pretty_hex::*;

use crate::kernel::StartBlock;

// TODO: move this in a seperate crate as a elf/pe/macho helper for pa/va

// TODO: we need both a physical and virtual reader, our use case is va though
/*
pub struct VirtualScrollReader<'a, T: PhysicalRead + VirtualRead> {
    mem: &'a mut T,
    dtb: DTB,
    base: Address,
}

impl<'a, T: PhysicalRead + VirtualRead> VirtualScrollReader<'a, T> {
    pub fn new(mem: &'a mut T, dtb: DTB, base: Address) -> Self {
        VirtualScrollReader {
            mem: mem,
            dtb: dtb,
            base: base,
        }
    }
}
*/

/*
impl<'a, T: PhysicalRead + VirtualRead> Index<usize> for VirtualScrollReader<'a, T> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        info!("VirtualScrollReader(): reading byte at {:x}", idx);
        let buf = self.mem.virt_read(self.dtb.arch, self.dtb.dtb, self.base + Length::from(idx), Length::from_b(1)).unwrap();
        &buf[0]
    }
}

impl<'a, T: PhysicalRead + VirtualRead> Index<RangeFrom<usize>> for VirtualScrollReader<'a, T> {
    type Output = [u8];

    fn index(&self, range: RangeFrom<usize>) -> &[u8] {
        info!("VirtualScrollReader(): reading range from {:x}", range.start);
        let buf = self.mem.virt_read(self.dtb.arch, self.dtb.dtb, self.base + Length::from(range.start), Length::from_mb(2)).unwrap();
        &buf
    }
}

impl<'a, T: PhysicalRead + VirtualRead, Ctx> MeasureWith<Ctx> for VirtualScrollReader<'a, T> {
    type Units = usize;

    #[inline]
    fn measure_with(&self, _ctx: &Ctx) -> Self::Units {
        // TODO: return a somewhat senseful length here based on ram limits?
        //println!("measuring results in len {}", self.buf.len());
        Length::from_gb(16).as_usize()
    }
}
*/

////////////////////
///
///
pub fn test_read_pe<T: PhysicalRead + VirtualRead>(
    mem: &mut T,
    start_block: StartBlock,
    base: Address,
) -> Result<()> {
    let header_buf = mem.virt_read(start_block.arch, start_block.dtb, base, Length::from_kb(4))?;
    info!("{:?}", header_buf.hex_dump());

    /*
    let header_buf = mem.virt_read(dtb.arch, dtb.dtb, base, Length::from_mb(32))?;
    let header = PE::parse(&header_buf).unwrap(); // pe::header::Header::parse(&header_buf).unwrap(); // TODO: ?;
    println!("header: {:?}", header);

    header.sections.iter().for_each(|s| println!("section found: {}", String::from_utf8(s.name.to_vec()).unwrap_or_default()));
    header.exports.iter().for_each(|e| println!("export found: {:?}", e));
    header.export_data.iter().for_each(|e| println!("export_data found: {:?}", e));
    header.libraries.iter().for_each(|l| println!("library found: {}", l));
    */
    /*
        let sections_offset = &mut (header.dos_header.pe_pointer as usize + pe::header::SIZEOF_PE_MAGIC + pe::header::SIZEOF_COFF_HEADER + header.coff_header.size_of_optional_header as usize);
        let sections = header.coff_header.sections(&header_buf, sections_offset).unwrap();
        println!("sections: {:?}", sections);

        if let Some(optional_header) = header.optional_header {
            println!("optional_header: {:?}", optional_header);

            let entry = optional_header.standard_fields.address_of_entry_point as usize;
            let image_base = optional_header.windows_fields.image_base as usize;
            let file_alignment = optional_header.windows_fields.file_alignment;
            println!("entry {:#x} image_base {:#x} file_alignment {:#x}", entry, image_base, file_alignment);

            if let Some(export_table) = *optional_header.data_directories.get_export_table() {
                println!("export_table: {:?}", export_table);
                let export_rva = export_table.virtual_address as usize;
                println!("export_rva: {:x}", export_rva);
                // base + export_rva ...
                let export_offset = pe::utils::find_offset_or(export_rva, &sections, file_alignment, &format!("cannot map export_rva ({:#x}) into offset", export_rva)).unwrap();
                println!("export_offset: {:x}", export_offset);

                //let export_buf =

                // TODO: ExportDirectoryTable::parse(bytes, export_offset)
                if let Ok(ed) = pe::export::ExportData::parse(&header_buf, export_table, &sections, file_alignment) {
                    println!("export data {:#?}", ed);
                    /*
                    exports = export::Export::parse(bytes, &ed, &sections, file_alignment)?;
                    name = ed.name;
                    debug!("name: {:#?}", name);
                    export_data = Some(ed);
                    */
                }

            }
        }
    */
    /*
    println!("pe header parsed! length={:x}", p.size);
    println!("{:?}", p);
    println!("name: {}", p.name.unwrap_or_default());
    p.sections.iter().for_each(|s| println!("section found: {}", String::from_utf8(s.name.to_vec()).unwrap_or_default()));
    p.exports.iter().for_each(|e| println!("export found: {:?}", e));
    p.export_data.iter().for_each(|e| println!("export_data found: {:?}", e));
    p.libraries.iter().for_each(|l| println!("library found: {}", l));
    //p.header.optional_header.unwrap().windows_fields.
    let optional_header = p.header.optional_header.expect("No optional header");
    let exps = optional_header.data_directories.get_export_table().unwrap();
    println!("export table size: {}", exps.size);
    */

    Ok(())
}
