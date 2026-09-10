// Portable little-endian table format. The build script produces this asset;
// browsers only decode it, never construct pruning tables.
fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x100000001b3))
}

#[allow(dead_code)] // Used by the build script and reproducibility test.
pub fn encode() -> Vec<u8> {
    let mt = MoveTable::get();
    let pt = PruningTable::get();
    let mut data = Vec::new();
    macro_rules! moves { ($($field:ident),*) => { $(
        for row in mt.$field.iter() { for n in row { data.extend(n.to_le_bytes()); } }
    )* }; }
    moves!(twist, flip, ud_slice, cp, ep8, slice_p);
    macro_rules! bytes { ($($field:ident),*) => { $(
        data.extend((pt.$field.len() as u32).to_le_bytes());
        data.extend(pt.$field.iter().map(|v| *v as u8));
    )* }; }
    bytes!(twist_slice, flip_slice, cp_slice, ep8_slice);
    macro_rules! shorts { ($field:ident) => { for n in pt.$field.iter() { data.extend(n.to_le_bytes()); } }; }
    shorts!(twist_class);
    bytes!(twist_sym, twist_self_sym);
    shorts!(flip_class);
    bytes!(flip_sym, flip_self_sym);
    shorts!(ud_slice_x2);
    let mut output = b"CUBE0001".to_vec();
    output.extend(checksum(&data).to_le_bytes());
    output.extend(data);
    output
}

struct Reader<'a> { data: &'a [u8], at: usize }
impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        assert_eq!(&data[..8], b"CUBE0001", "table version mismatch");
        assert_eq!(u64::from_le_bytes(data[8..16].try_into().unwrap()), checksum(&data[16..]), "table checksum mismatch");
        Self { data, at: 16 }
    }
    fn short(&mut self) -> u16 {
        let n = u16::from_le_bytes(self.data[self.at..self.at+2].try_into().unwrap());
        self.at += 2;
        n
    }
    fn moves<const N: usize>(&mut self) -> Box<[[u16;18];N]> {
        (0..N).map(|_| std::array::from_fn(|_| self.short())).collect::<Vec<_>>()
            .into_boxed_slice().try_into().unwrap()
    }
    fn bytes(&mut self) -> Box<[u8]> {
        let len = u32::from_le_bytes(self.data[self.at..self.at+4].try_into().unwrap()) as usize;
        self.at += 4;
        let result = self.data[self.at..self.at+len].into();
        self.at += len;
        result
    }
    fn shorts<const N: usize>(&mut self) -> Box<[u16;N]> {
        (0..N).map(|_| self.short()).collect::<Vec<_>>().into_boxed_slice().try_into().unwrap()
    }
    fn bools<const N: usize>(&mut self) -> Box<[bool;N]> {
        self.bytes().iter().map(|v| *v != 0).collect::<Vec<_>>().into_boxed_slice().try_into().unwrap()
    }
}
fn decode_moves(bytes: &[u8]) -> MoveTable {
    let mut r = Reader::new(bytes);
    MoveTable { twist: r.moves(), flip: r.moves(), ud_slice: r.moves(), cp: r.moves(), ep8: r.moves(), slice_p: r.moves() }
}
fn decode_pruning(bytes: &[u8]) -> PruningTable {
    let mut r = Reader::new(bytes);
    r.at += (2187+2048+495+40320+40320+24)*18*2;
    let result = PruningTable {
        twist_slice:r.bytes(), flip_slice:r.bytes(), cp_slice:r.bytes(), ep8_slice:r.bytes(),
        twist_class:r.shorts(), twist_sym:r.bools(), twist_self_sym:r.bools(),
        flip_class:r.shorts(), flip_sym:r.bools(), flip_self_sym:r.bools(), ud_slice_x2:r.shorts()
    };
    assert_eq!(r.at, bytes.len());
    result
}
