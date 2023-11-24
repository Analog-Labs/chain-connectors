//! `NodeCodec` implementation for Rlp

use crate::rstd::{borrow::Borrow, iter, marker::PhantomData, ops::Range, vec::Vec};
use hash_db::Hasher;
use hex_literal::hex;
use primitive_types::H256;
use rlp::{DecoderError, Prototype, Rlp, RlpStream};
use trie_db::{
    node::{NibbleSlicePlan, NodeHandlePlan, NodePlan, Value, ValuePlan},
    ChildReference, NodeCodec,
};

/// Concrete implementation of a `NodeCodec` with Rlp encoding, generic over the `Hasher`
#[derive(Default, Clone)]
pub struct RlpNodeCodec<H: Hasher> {
    mark: PhantomData<H>,
}

/// The hashed null node for the ethereum trie, the same as keccak256(0x80)
const HASHED_NULL_NODE_BYTES: [u8; 32] =
    hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421");
pub const HASHED_NULL_NODE: H256 = H256(HASHED_NULL_NODE_BYTES);

/// Encode a partial value with an iterator as input.
fn encode_partial_from_iterator_iter<'a>(
    mut partial: impl Iterator<Item = u8> + 'a,
    odd: bool,
    is_leaf: bool,
) -> impl Iterator<Item = u8> + 'a {
    let first = if odd { partial.next().unwrap_or(0) } else { 0 };
    encode_partial_inner_iter(first, partial, odd, is_leaf)
}

/// Encode a partial value with an iterator as input.
fn encode_partial_inner_iter<'a>(
    first_byte: u8,
    partial_remaining: impl Iterator<Item = u8> + 'a,
    odd: bool,
    is_leaf: bool,
) -> impl Iterator<Item = u8> + 'a {
    let encoded_type = if is_leaf { 0x20 } else { 0 };
    let first = if odd { 0x10 + encoded_type + first_byte } else { encoded_type };
    iter::once(first).chain(partial_remaining)
}

fn decode_value_range(rlp: &Rlp, mut offset: usize) -> Result<Range<usize>, DecoderError> {
    let payload = rlp.payload_info()?;
    offset += payload.header_len;
    Ok(offset..(offset + payload.value_len))
}

fn decode_child_handle_plan<H: Hasher>(
    child_rlp: &Rlp,
    mut offset: usize,
) -> Result<NodeHandlePlan, DecoderError> {
    Ok(if child_rlp.is_data() && child_rlp.size() == H::LENGTH {
        let payload = child_rlp.payload_info()?;
        offset += payload.header_len;
        NodeHandlePlan::Hash(offset..(offset + payload.value_len))
    } else {
        NodeHandlePlan::Inline(offset..(offset + child_rlp.as_raw().len()))
    })
}

impl<H> NodeCodec for RlpNodeCodec<H>
where
    H: Hasher,
{
    type Error = DecoderError;
    type HashOut = H::Out;

    fn hashed_null_node() -> H::Out {
        H::hash(<Self as NodeCodec>::empty_node())
    }

    fn decode_plan(data: &[u8]) -> Result<NodePlan, Self::Error> {
        let r = Rlp::new(data);
        match r.prototype()? {
            // either leaf or extension - decode first item with NibbleSlice::???
            // and use is_leaf return to figure out which.
            // if leaf, second item is a value (is_data())
            // if extension, second item is a node (either SHA3 to be looked up and
            // fed back into this function or inline RLP which can be fed back into this function).
            Prototype::List(2) => {
                let (partial_rlp, mut partial_offset) = r.at_with_offset(0)?;
                let partial_payload = partial_rlp.payload_info()?;
                partial_offset += partial_payload.header_len;

                let (partial, is_leaf) = if partial_rlp.is_empty() {
                    (NibbleSlicePlan::new(partial_offset..partial_offset, 0), false)
                } else {
                    let partial_header = partial_rlp.data()?[0];
                    // check leaf bit from header.
                    let is_leaf = partial_header & 32 == 32;
                    // Check the header bit to see if we're dealing with an odd partial (only a
                    // nibble of header info) or an even partial (skip a full
                    // byte).
                    let (start, byte_offset) =
                        if partial_header & 16 == 16 { (0, 1) } else { (1, 0) };
                    let range =
                        (partial_offset + start)..(partial_offset + partial_payload.value_len);
                    (NibbleSlicePlan::new(range, byte_offset), is_leaf)
                };

                let (value_rlp, value_offset) = r.at_with_offset(1)?;
                Ok(if is_leaf {
                    let value = decode_value_range(&value_rlp, value_offset)?;
                    let value = ValuePlan::Inline(value); // TODO: check if this is correct
                    NodePlan::Leaf { partial, value }
                } else {
                    let child = decode_child_handle_plan::<H>(&value_rlp, value_offset)?;
                    NodePlan::Extension { partial, child }
                })
            },
            // branch - first 16 are nodes, 17th is a value (or empty).
            Prototype::List(17) => {
                let mut children = [
                    None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None,
                ];
                for (i, child) in children.iter_mut().enumerate() {
                    let (child_rlp, child_offset) = r.at_with_offset(i)?;
                    if !child_rlp.is_empty() {
                        *child = Some(decode_child_handle_plan::<H>(&child_rlp, child_offset)?);
                    }
                }
                let (value_rlp, value_offset) = r.at_with_offset(16)?;
                let value = if value_rlp.is_empty() {
                    None
                } else {
                    Some(decode_value_range(&value_rlp, value_offset)?)
                };
                let value = value.map(ValuePlan::Inline); // TODO: check if this is correct
                Ok(NodePlan::Branch { value, children })
            },
            // an empty branch index.
            Prototype::Data(0) => Ok(NodePlan::Empty),
            // something went wrong.
            _ => Err(DecoderError::Custom("Rlp is not valid.")),
        }
    }

    fn is_empty_node(data: &[u8]) -> bool {
        Rlp::new(data).is_empty()
    }

    fn empty_node() -> &'static [u8] {
        &[0x80]
    }

    fn leaf_node(partial: impl Iterator<Item = u8>, number_nibble: usize, value: Value) -> Vec<u8> {
        let mut stream = RlpStream::new_list(2);
        stream.append_iter(encode_partial_from_iterator_iter(partial, number_nibble % 2 > 0, true));
        stream.append(&match value {
            Value::Inline(bytes) => bytes,
            Value::Node(_) => unimplemented!("unsupported"),
        });
        stream.out().freeze().to_vec()
    }

    fn extension_node(
        partial: impl Iterator<Item = u8>,
        number_nibble: usize,
        child_ref: ChildReference<H::Out>,
    ) -> Vec<u8> {
        let mut stream = RlpStream::new_list(2);
        stream.append_iter(encode_partial_from_iterator_iter(
            partial,
            number_nibble % 2 > 0,
            false,
        ));
        match child_ref {
            ChildReference::Hash(hash) => stream.append(&hash.as_ref()),
            ChildReference::Inline(inline_data, length) => {
                let bytes = &AsRef::<[u8]>::as_ref(&inline_data)[..length];
                stream.append_raw(bytes, 1)
            },
        };
        stream.out().freeze().to_vec()
    }

    fn branch_node(
        children: impl Iterator<Item = impl Borrow<Option<ChildReference<H::Out>>>>,
        maybe_value: Option<Value>,
    ) -> Vec<u8> {
        let mut stream = RlpStream::new_list(17);
        for child_ref in children {
            match child_ref.borrow() {
                Some(c) => match c {
                    ChildReference::Hash(h) => stream.append(&h.as_ref()),
                    ChildReference::Inline(inline_data, length) => {
                        let bytes = &AsRef::<[u8]>::as_ref(inline_data)[..*length];
                        stream.append_raw(bytes, 1)
                    },
                },
                None => stream.append_empty_data(),
            };
        }
        if let Some(value) = maybe_value {
            match value {
                Value::Inline(bytes) => stream.append(&bytes),
                Value::Node(_) => unimplemented!("unsupported"),
            };
        } else {
            stream.append_empty_data();
        }
        stream.out().freeze().to_vec()
    }

    fn branch_node_nibbled(
        _partial: impl Iterator<Item = u8>,
        _number_nibble: usize,
        _children: impl Iterator<Item = impl Borrow<Option<ChildReference<H::Out>>>>,
        _maybe_value: Option<Value>,
    ) -> Vec<u8> {
        unreachable!("This codec is only used with a trie Layout that uses extension node.")
    }
}
