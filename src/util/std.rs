use std::cmp::Ordering;
use std::collections::BTreeMap;

pub trait EqNoneAsEmpty<T>
where
    T: Default + PartialEq,
{
    fn eq_none_as_default(&self, other: &Option<T>) -> bool;
}

impl<T> EqNoneAsEmpty<T> for Option<T>
where
    T: Default + PartialEq,
{
    fn eq_none_as_default(&self, other: &Option<T>) -> bool {
        return match (self, other) {
            (Some(s), Some(o)) => s == o,
            (None, Some(o)) => &T::default() == o,
            (Some(s), None) => s == &T::default(),
            (None, None) => true,
        };
    }
}

pub trait TreeMapExtension<K, V>
where
    K: std::cmp::Ord,
    V: std::cmp::Ord,
{
    fn contains(&self, other: &BTreeMap<K, V>) -> bool;
    fn contains_entry(&self, key: &K, value: &V) -> bool;
}

impl<K, V> TreeMapExtension<K, V> for BTreeMap<K, V>
where
    K: std::cmp::Ord,
    V: std::cmp::Ord,
{
    fn contains(&self, other: &BTreeMap<K, V>) -> bool {
        return other.iter().all(|(k, v)| self.contains_entry(k, v));
    }

    fn contains_entry(&self, key: &K, value: &V) -> bool {
        return self
            .iter()
            .any(|(k, v)| k.cmp(key) == Ordering::Equal && v.cmp(value) == Ordering::Equal);
    }
}

pub trait TreeMapOptExtension<K, V>
where
    K: std::cmp::Ord,
    V: std::cmp::Ord,
{
    fn contains_opt(&self, other: &Option<BTreeMap<K, V>>) -> bool;
}

impl<K, V> TreeMapOptExtension<K, V> for Option<BTreeMap<K, V>>
where
    K: std::cmp::Ord,
    V: std::cmp::Ord,
{
    fn contains_opt(&self, other: &Option<BTreeMap<K, V>>) -> bool {
        return match (&self, other) {
            (Some(m1), Some(m2)) => m1.contains(m2),
            (Some(_), None) => true,
            (None, Some(m2)) => m2.is_empty(),
            (None, None) => true,
        };
    }
}

#[cfg(test)]
mod test {
    use crate::util::std::{TreeMapExtension, TreeMapOptExtension};
    use std::collections::BTreeMap;

    #[test]
    fn tree_map_fully_contains_other() {
        // Arrange
        let mut m1 = BTreeMap::new();
        m1.insert("h1", "v1");
        m1.insert("h2", "v2");

        let mut m2 = BTreeMap::new();
        m2.insert("h1", "v1");
        m2.insert("h2", "v2");

        // Act
        let result = m1.contains(&m2);

        // Assert
        assert_eq!(true, result);
    }

    #[test]
    fn tree_map_contains_subset() {
        // Arrange
        let mut m1 = BTreeMap::new();
        m1.insert("h1", "v1");
        m1.insert("h2", "v2");

        let mut m2 = BTreeMap::new();
        m2.insert("h1", "v1");

        // Act
        let result = m1.contains(&m2);

        // Assert
        assert_eq!(true, result);
    }

    #[test]
    fn tree_map_does_not_contain_other() {
        // Arrange
        let mut m1 = BTreeMap::new();
        m1.insert("h1", "v1");

        let mut m2 = BTreeMap::new();
        m2.insert("h1", "v1");
        m2.insert("h2", "v2");

        // Act
        let result = m1.contains(&m2);

        // Assert
        assert_eq!(false, result);
    }

    #[test]
    fn tree_map_contains_all_keys_no_values() {
        // Arrange
        let mut m1 = BTreeMap::new();
        m1.insert("h1", "v1");
        m1.insert("h2", "v2");

        let mut m2 = BTreeMap::new();
        m2.insert("h1", "v3");
        m2.insert("h2", "v4");

        // Act
        let result = m1.contains(&m2);

        // Assert
        assert_eq!(false, result);
    }

    #[test]
    fn tree_map_contains_all_keys_some_values() {
        // Arrange
        let mut m1 = BTreeMap::new();
        m1.insert("h1", "v1");
        m1.insert("h2", "v2");

        let mut m2 = BTreeMap::new();
        m2.insert("h1", "v1");
        m2.insert("h2", "v2");
        m2.insert("h3", "v3");

        // Act
        let result = m1.contains(&m2);

        // Assert
        assert_eq!(false, result);
    }

    #[test]
    fn tree_map_contains_all_keys_some_values_equal_length() {
        // Arrange
        let mut m1 = BTreeMap::new();
        m1.insert("h1", "v1");
        m1.insert("h2", "v2");

        let mut m2 = BTreeMap::new();
        m2.insert("h1", "v1");
        m2.insert("h2", "v3");

        // Act
        let result = m1.contains(&m2);

        // Assert
        assert_eq!(false, result);
    }

    #[test]
    fn tree_map_contains_opt_both_some() {
        // Arrange
        let mut m1 = BTreeMap::new();
        m1.insert("h1", "v1");

        let mut m2 = BTreeMap::new();
        m2.insert("h1", "v1");

        // Act
        let result = Some(m1).contains_opt(&Some(m2));

        // Assert
        assert_eq!(true, result);
    }

    #[test]
    fn tree_map_contains_opt_first_some_second_none() {
        // Arrange
        let mut m = BTreeMap::new();
        m.insert("h1", "v1");

        // Act
        let result = Some(m).contains_opt(&None);

        // Assert
        assert_eq!(true, result);
    }

    #[test]
    fn tree_map_contains_opt_first_none_second_some() {
        // Arrange
        let mut m = BTreeMap::new();
        m.insert("h1", "v1");

        // Act
        let result = None.contains_opt(&Some(m));

        // Assert
        assert_eq!(false, result);
    }

    #[test]
    fn tree_map_contains_opt_first_none_second_some_but_empty() {
        // Arrange
        let m: BTreeMap<String, String> = BTreeMap::new();

        // Act
        let result = None.contains_opt(&Some(m));

        // Assert
        assert_eq!(true, result);
    }

    #[test]
    fn tree_map_contains_opt_both_none() {
        // Arrange
        let m: Option<BTreeMap<String, String>> = None;

        // Act
        let result = m.contains_opt(&None);

        // Assert
        assert_eq!(true, result);
    }
}
