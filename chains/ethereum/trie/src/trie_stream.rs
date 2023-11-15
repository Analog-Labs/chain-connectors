use crate::rstd::vec::Vec;
use core::{iter::Iterator, option::Option};
use rlp::RlpStream;
use trie_root::{Hasher, TrieStream, Value};

/// Concrete `TrieStream` impl for the ethereum trie.
#[derive(Default)]
pub struct Hash256RlpTrieStream {
    stream: rlp::RlpStream,
}

impl TrieStream for Hash256RlpTrieStream {
    fn new() -> Self {
        Self { stream: RlpStream::new() }
    }

    fn append_empty_data(&mut self) {
        self.stream.append_empty_data();
    }

    fn begin_branch(
        &mut self,
        _maybe_key: Option<&[u8]>,
        _maybe_value: Option<Value>,
        _has_children: impl Iterator<Item = bool>,
    ) {
        // an item for every possible nibble/suffix
        // + 1 for data
        self.stream.begin_list(17);
    }

    fn append_empty_child(&mut self) {
        self.stream.append_empty_data();
    }

    fn end_branch(&mut self, value: Option<Value>) {
        match value {
            Some(value) => match value {
                Value::Inline(value) => self.stream.append(&value),
                Value::Node(value) => self.stream.append(&value),
            },
            None => self.stream.append_empty_data(),
        };
    }

    fn append_leaf(&mut self, key: &[u8], value: Value) {
        self.stream.begin_list(2);
        self.stream.append_iter(hex_prefix_encode(key, true));
        match value {
            Value::Inline(value) => self.stream.append(&value),
            Value::Node(value) => self.stream.append(&value),
        };
    }

    fn append_extension(&mut self, key: &[u8]) {
        self.stream.begin_list(2);
        self.stream.append_iter(hex_prefix_encode(key, false));
    }

    fn append_substream<H: Hasher>(&mut self, other: Self) {
        let out = other.out();
        match out.len() {
            0..=31 => self.stream.append_raw(&out, 1),
            _ => self.stream.append(&H::hash(&out).as_ref()),
        };
    }

    fn out(self) -> Vec<u8> {
        self.stream.out().freeze().into()
    }
}

// Copy from `triehash` crate.
/// Hex-prefix Notation. First nibble has flags: oddness = 2^0 & termination = 2^1.
///
/// The "termination marker" and "leaf-node" specifier are completely equivalent.
///
/// Input values are in range `[0, 0xf]`.
///
/// ```markdown
///  [0,0,1,2,3,4,5]   0x10012345 // 7 > 4
///  [0,1,2,3,4,5]     0x00012345 // 6 > 4
///  [1,2,3,4,5]       0x112345   // 5 > 3
///  [0,0,1,2,3,4]     0x00001234 // 6 > 3
///  [0,1,2,3,4]       0x101234   // 5 > 3
///  [1,2,3,4]         0x001234   // 4 > 3
///  [0,0,1,2,3,4,5,T] 0x30012345 // 7 > 4
///  [0,0,1,2,3,4,T]   0x20001234 // 6 > 4
///  [0,1,2,3,4,5,T]   0x20012345 // 6 > 4
///  [1,2,3,4,5,T]     0x312345   // 5 > 3
///  [1,2,3,4,T]       0x201234   // 4 > 3
/// ```
fn hex_prefix_encode(nibbles: &[u8], leaf: bool) -> impl Iterator<Item = u8> + '_ {
    let inlen = nibbles.len();
    let oddness_factor = inlen % 2;

    let first_byte = {
        #[allow(clippy::cast_possible_truncation)]
        let mut bits = ((inlen as u8 & 1) + (2 * u8::from(leaf))) << 4;
        if oddness_factor == 1 {
            bits += nibbles[0];
        }
        bits
    };
    core::iter::once(first_byte)
        .chain(nibbles[oddness_factor..].chunks(2).map(|ch| ch[0] << 4 | ch[1]))
}
