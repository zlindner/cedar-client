/*unsafe {
    let data = mmap.as_ptr();
    let header = data as *const Header;

    if (*header).magic != 0x34474B50 {
        return Err(NxError::InvalidMagicBytes);
    }

    println!("{:?}", *header);

    let nodetable = data.offset((*header).nodeoffset as isize) as *const Node;
    let string_table = data.offset((*header).stringoffset as isize) as *const u64;
    println!("string_table: {:?}", *string_table);

    let off = *string_table.offset((*nodetable).name as isize);
    println!("off: {:?}", off);
    let ptr = data.offset(off as isize);
    let size = ptr as *const u16;
    let str = str::from_utf8_unchecked(from_raw_parts(ptr.offset(2), (*size) as usize));

    println!("{:?}", *nodetable);
}*/

/*let name = u32::from_le_bytes(node_table.get(0..4).unwrap().try_into().unwrap());
let children = u32::from_le_bytes(node_table.get(4..8).unwrap().try_into().unwrap());
let count = u16::from_le_bytes(node_table.get(8..10).unwrap().try_into().unwrap());
let data_type = u16::from_le_bytes(node_table.get(10..12).unwrap().try_into().unwrap());
let data = u64::from_le_bytes(node_table.get(12..20).unwrap().try_into().unwrap());

let off = string_table + name as u64;

let str_len = u16::from_le_bytes(
    mmap.get(off as usize..off as usize + 2)
        .unwrap()
        .try_into()
        .unwrap(),
);

let str_val = str::from_utf8(
    mmap.get(off as usize..off as usize + str_len as usize)
        .unwrap(),
)
.unwrap();*/
