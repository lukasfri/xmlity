// Define a struct that holds the item to be cloned
pub struct RepeatClone<T> {
    item: T,
}

// Implement the Iterator trait for the RepeatClone struct
impl<T> Iterator for RepeatClone<T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // Clone the item and return it
        Some(self.item.clone())
    }
}

// A function to create a RepeatClone iterator
pub fn repeat_clone<T>(item: T) -> RepeatClone<T>
where
    T: Clone,
{
    RepeatClone { item }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeat_clone() {
        let mut iter = repeat_clone(42);
        assert_eq!(iter.next(), Some(42));
        assert_eq!(iter.next(), Some(42));
        assert_eq!(iter.next(), Some(42));
    }
}
