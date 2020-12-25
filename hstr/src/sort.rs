use std::cmp::{Eq, Reverse};
use std::collections::HashMap;
use std::hash::Hash;

pub fn sort<T>(mut commands: Vec<T>) -> Vec<T>
where
    T: Clone + Eq + Hash,
{
    let freq_map = frequency_map(&commands);
    let pos_map = position_map(&commands);
    commands.sort_by_key(|c| Reverse(pos_map.get(c).unwrap()));
    commands.dedup();
    commands.sort_by_key(|c| Reverse(freq_map.get(c).unwrap()));
    commands
}

fn frequency_map<T>(commands: &[T]) -> HashMap<T, usize>
where
    T: Clone + Eq + Hash,
{
    let mut map = HashMap::new();
    for cmd in commands.iter() {
        *map.entry(cmd.clone()).or_insert(0) += 1;
    }
    map
}

fn position_map<T>(commands: &[T]) -> HashMap<T, usize>
where
    T: Clone + Eq + Hash,
{
    let mut map = HashMap::new();
    for (pos, cmd) in commands.iter().enumerate() {
        map.insert(cmd.clone(), pos);
    }
    map
}

#[cfg(test)]
mod tests {
    #[test]
    fn sort() {
        let vec = vec![3, 2, 4, 6, 2, 4, 3, 3, 4, 5, 6, 3, 2, 4, 5, 5, 3];
        let sorted_vec = super::sort(vec);
        assert_eq!(sorted_vec, [3, 4, 5, 2, 6]);
    }
}
