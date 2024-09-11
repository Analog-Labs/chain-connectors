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
}
