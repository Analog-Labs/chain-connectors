#![allow(dead_code)]
use hashbrown::{HashMap, HashSet};
use rosetta_config_ethereum::ext::types::{Address, Bloom, BloomInput, H256};
use std::iter::Iterator;

pub struct LogFilter {
    filter: HashMap<Address, HashSet<H256>>,
}

impl LogFilter {
    pub fn new() -> Self {
        Self { filter: HashMap::new() }
    }

    pub fn add<T: Iterator<Item = H256>>(&mut self, address: Address, topics: T) -> bool {
        match self.filter.entry(address) {
            hashbrown::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().extend(topics);
                false
            },
            hashbrown::hash_map::Entry::Vacant(entry) => {
                entry.insert(topics.collect());
                true
            },
        }
    }

    pub fn remove(&mut self, address: &Address) -> Option<HashSet<H256>> {
        self.filter.remove(address)
    }

    pub fn is_empty(&self) -> bool {
        self.filter.is_empty()
    }

    /// Returns an iterator of topics that match the given bloom filter
    pub fn topics_from_bloom(
        &self,
        bloom: Bloom,
    ) -> impl Iterator<Item = (Address, impl Iterator<Item = H256> + '_)> + '_ {
        self.filter.iter().filter_map(move |(address, topics)| {
            if !bloom.contains_input(BloomInput::Raw(address.as_bytes())) {
                return None;
            }
            let topics = topics
                .iter()
                .copied()
                .filter(move |topic| bloom.contains_input(BloomInput::Raw(topic.as_bytes())));
            Some((*address, topics))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use rosetta_config_ethereum::ext::types::Bloom;

    #[test]
    fn add_remove_works() {
        let mut filter = LogFilter::new();
        let address = Address::from([0; 20]);
        let topics = [H256::from([0; 32]), H256::from([1; 32])];

        assert!(filter.is_empty());
        assert!(filter.add(address, topics.into_iter()));
        assert!(!filter.is_empty());
        assert!(!filter.add(address, topics.into_iter()));
        assert!(filter.remove(&address).is_some());
        assert!(filter.remove(&address).is_none());
        assert!(filter.is_empty());
    }

    #[test]
    fn filter_topics_works() {
        let mut filter = LogFilter::new();
        let logs_bloom = Bloom::from(hex!("00000000000000000000000000000000000000000000020200040000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000002000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000200000000000000000"));

        // Empty filter
        let mut logs = filter.topics_from_bloom(logs_bloom);
        assert!(logs.next().is_none());
        drop(logs);

        let expect_address = Address::from(hex!("97be939b2eb5a462c634414c8134b09ebad04d83"));
        let expect_topics = [
            H256(hex!("b7dbf4f78c37528484cb9761beaca968c613f3c6c534b25b1988b912413c68bc")),
            H256(hex!("fca76ae197bb7f913a92bd1f31cb362d0fdbf27b2cc56d8b9bc22d0d76c58dc8")),
        ];
        filter.add(expect_address, expect_topics.into_iter());

        let mut logs = filter.topics_from_bloom(logs_bloom);
        let (address, mut topics) = logs.next().unwrap();
        assert_eq!(address, expect_address);
        assert_eq!(topics.next().unwrap(), expect_topics[0]);
        assert_eq!(topics.next().unwrap(), expect_topics[1]);
        assert!(logs.next().is_none());
        assert!(topics.next().is_none());
    }
}
