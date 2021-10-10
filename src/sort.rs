use std::cmp::{Eq, Reverse};
use std::collections::HashMap;
use std::hash::Hash;

pub fn sort<T>(mut history: Vec<T>) -> Vec<T>
where
    T: Clone + Eq + Hash,
{
    let freq_map = frequency_map(&history);
    let pos_map = position_map(&history);
    history.sort_by_key(|c| Reverse(pos_map.get(c).unwrap()));
    history.dedup();
    history.sort_by_key(|c| Reverse(freq_map.get(c).unwrap()));
    history
}

fn frequency_map<T>(history: &[T]) -> HashMap<T, usize>
where
    T: Clone + Eq + Hash,
{
    let mut map = HashMap::new();
    history.iter().for_each(|cmd| {
        *map.entry(cmd.to_owned()).or_insert(0) += 1;
    });
    map
}

fn position_map<T>(history: &[T]) -> HashMap<T, usize>
where
    T: Clone + Eq + Hash,
{
    let mut map = HashMap::new();
    history.iter().enumerate().for_each(|(pos, cmd)| {
        map.insert(cmd.to_owned(), pos);
    });
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
