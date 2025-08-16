use std::{collections::HashMap, hash::Hash};

pub trait MultiMap<K, V> {
	/// returns the new length of the array
	fn multimap_insert(&mut self, key: K, value: V) -> usize;

	fn multimap_drain<'a>(
		&'a mut self,
	) -> impl Iterator<Item = (&'a K, impl Iterator<Item = V>)> + 'a
	where
		K: 'a;
	fn multimap_drain_total<'a>(&'a mut self) -> impl Iterator<Item = (&'a K, V)> + 'a
	where
		K: 'a,
	{
		self.multimap_drain()
			.map(|(k, iter)| iter.map(move |v| (k, v)))
			.flatten()
	}
}
impl<K: PartialEq + Eq + Hash, V> MultiMap<K, V> for HashMap<K, Vec<V>> {
	fn multimap_insert(&mut self, key: K, value: V) -> usize {
		match self.get_mut(&key) {
			Some(a) => {
				a.push(value);
				a.len()
			}
			None => {
				self.insert(key, vec![value]);
				1
			}
		}
	}

	fn multimap_drain<'a>(&'a mut self) -> impl Iterator<Item = (&'a K, impl Iterator<Item = V>)>
	where
		K: 'a,
	{
		self.iter_mut().map(|(key, value)| (key, value.drain(..)))
	}
}
