// Portable little-endian 4-bit packed pruning table format (CUBE0002).
// The build script produces this asset; browsers zero-copy borrow it from
// static WASM memory without allocating or copying pruning tables.
const TABLE_MAGIC: &[u8; 8] = b"CUBE0002";

fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}

#[allow(dead_code)] // Used by the build script and reproducibility test.
pub fn encode() -> Vec<u8> {
    let pt = PruningTable::get();
    let total_data_len = 4 * 4
        + pt.twist_slice.len()
        + pt.flip_slice.len()
        + pt.cp_slice.len()
        + pt.ep8_slice.len();
    let mut data = Vec::with_capacity(total_data_len);
    for slice in [
        pt.twist_slice.as_ref(),
        pt.flip_slice.as_ref(),
        pt.cp_slice.as_ref(),
        pt.ep8_slice.as_ref(),
    ] {
        data.extend((slice.len() as u32).to_le_bytes());
        data.extend(slice);
    }
    let mut output = Vec::with_capacity(16 + data.len());
    output.extend_from_slice(TABLE_MAGIC);
    output.extend(checksum(&data).to_le_bytes());
    output.extend(data);
    output
}

#[allow(dead_code)]
pub fn verify_table_data(data: &[u8]) -> Result<(), &'static str> {
    if data.len() < 16 {
        return Err("table data too short");
    }
    if &data[..8] != TABLE_MAGIC {
        return Err("table version mismatch");
    }
    let expected_checksum = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let actual_checksum = checksum(&data[16..]);
    if expected_checksum != actual_checksum {
        return Err("table checksum mismatch");
    }
    Ok(())
}

fn ensure_table_verified(data: &[u8]) {
    static VERIFIED_PTR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let ptr = data.as_ptr() as usize;
    if VERIFIED_PTR.load(std::sync::atomic::Ordering::Acquire) != ptr {
        verify_table_data(data).expect("table validation failed");
        VERIFIED_PTR.store(ptr, std::sync::atomic::Ordering::Release);
    }
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        ensure_table_verified(data);
        Self { data, at: 16 }
    }
    fn slice(&mut self, expected_len: usize) -> &'a [u8] {
        let len = u32::from_le_bytes(self.data[self.at..self.at + 4].try_into().unwrap()) as usize;
        assert_eq!(len, expected_len, "table slice length mismatch");
        self.at += 4;
        let result = &self.data[self.at..self.at + len];
        self.at += len;
        result
    }
}

fn decode_pruning(bytes: &'static [u8]) -> PruningTable {
    let mut r = Reader::new(bytes);
    let result = PruningTable {
        twist_slice: Cow::Borrowed(r.slice(TWIST_SLICE_PACKED_LEN)),
        flip_slice: Cow::Borrowed(r.slice(FLIP_SLICE_PACKED_LEN)),
        cp_slice: Cow::Borrowed(r.slice(CP_SLICE_PACKED_LEN)),
        ep8_slice: Cow::Borrowed(r.slice(EP8_SLICE_PACKED_LEN)),
    };
    assert_eq!(r.at, bytes.len());
    result
}
