use rlp::Encodable;

use bytes::BytesMut;

/// Adjust the index of an item for rlp encoding.
pub const fn adjust_index_for_rlp(i: usize, len: usize) -> usize {
    if i > 0x7f {
        i
    } else if i == 0x7f || i + 1 == len {
        0
    } else {
        i + 1
    }
}

/// Calculate a transaction root.
///
/// `(rlp(index), encoded(tx))` pairs.
pub fn calculate_transaction_root<T>(transactions: &[T]) -> B256
where
    T: AsRef<TransactionSigned>,
{
    ordered_trie_root_with_encoder(transactions, |tx: &T, buf| tx.as_ref().encode_inner(buf, false))
}

/// Compute a trie root of the collection of items with a custom encoder.
pub fn ordered_trie_root_with_encoder<T, F>(items: &[T], mut encode: F) -> B256
where
    F: FnMut(&T, &mut BytesMut),
{
    let mut index_buffer = BytesMut::new();
    let mut value_buffer = BytesMut::new();

    let mut hb = HashBuilder::default();
    let items_len = items.len();
    for i in 0..items_len {
        let index = adjust_index_for_rlp(i, items_len);

        index_buffer.clear();
        index.encode(&mut index_buffer);

        value_buffer.clear();
        encode(&items[index], &mut value_buffer);

        hb.add_leaf(Nibbles::unpack(&index_buffer), &value_buffer);
    }

    hb.root()
}
