use crate::util::Result;
use snafu::Snafu;

/// This trait defines common features of all featurecollections
pub trait FeatureCollection {
    /// Returns the number of features
    fn len(&self) -> usize;

    /// Returns whether the feature collection contains no features
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns whether this feature collection is simple, i.e., contains no multi-types
    fn is_simple(&self) -> bool;

    /// Removes the last feature from the collection
    fn remove_last_feature(&mut self) -> Result<()>;
}

#[derive(Debug, Snafu)]
pub enum FeatureCollectionError {
    #[snafu(display("Feature indices do not match"))]
    UnmatchedFeatureIndices,
    #[snafu(display("Unable to delete features from empty collection"))]
    DeleteFromEmpty,
}

#[cfg(test)]
mod test {
    use super::*;

    struct Dummy(Vec<u16>);

    impl FeatureCollection for Dummy {
        fn len(&self) -> usize {
            self.0.len()
        }
        fn is_simple(&self) -> bool {
            unimplemented!()
        }
        fn remove_last_feature(&mut self) -> Result<()> {
            unimplemented!()
        }
    }

    #[test]
    fn is_empty() {
        assert!(Dummy(Vec::new()).is_empty());
        assert!(!Dummy(vec![1, 2, 3]).is_empty());
    }
}