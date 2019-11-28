use crate::util::Result;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::fmt::{Error, Formatter};

/// Stores time intervals in ms in close-open semantic [start, end)
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct TimeInterval {
    start: i64,
    end: i64,
}

#[derive(Debug, Snafu)]
enum TimeIntervalError {
    #[snafu(display("Start `{}` must be before end `{}`", start, end))]
    EndBeforeStart { start: i64, end: i64 },

    #[snafu(display(
        "{} cannot be unioned with {} since the intervals are neither intersecting nor contiguous",
        i1,
        i2
    ))]
    UnmatchedIntervals { i1: TimeInterval, i2: TimeInterval },
}

impl TimeInterval {
    /// Create a new time interval and check bounds
    ///
    /// # Examples
    ///
    /// ```
    /// use geoengine_datatypes::primitives::TimeInterval;
    ///
    /// TimeInterval::new(0, 0).unwrap();
    /// TimeInterval::new(0, 1).unwrap();
    ///
    /// TimeInterval::new(1, 0).unwrap_err();
    /// ```
    ///
    pub fn new(start: i64, end: i64) -> Result<Self> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(TimeIntervalError::EndBeforeStart { start, end }.into())
        }
    }

    /// Create a new time interval without bound checks
    ///
    /// # Examples
    ///
    /// ```
    /// use geoengine_datatypes::primitives::TimeInterval;
    ///
    /// let t_unsafe = unsafe { TimeInterval::new_unchecked(0, 1) };
    ///
    /// assert_eq!(t_unsafe, TimeInterval::new(0, 1).unwrap());
    /// ```
    ///
    pub unsafe fn new_unchecked(start: i64, end: i64) -> Self {
        Self { start, end }
    }

    /// Returns whether the other TimeInterval is contained (smaller or equal) within this interval
    ///
    /// # Examples
    ///
    /// ```
    /// use geoengine_datatypes::primitives::TimeInterval;
    ///
    /// let valid_pairs = vec![
    ///     ((0, 1), (0, 1)),
    ///     ((0, 3), (1, 2)),
    ///     ((0, 2), (0, 1)),
    ///     ((0, 2), (1, 2)),
    /// ];
    ///
    /// for ((t1, t2), (t3, t4)) in valid_pairs {
    ///     let i1 = TimeInterval::new(t1, t2).unwrap();
    ///     let i2 = TimeInterval::new(t3, t4).unwrap();
    ///     assert!(i1.contains(&i2), "{:?} should contain {:?}", i1, i2);
    /// }
    ///
    /// let invalid_pairs = vec![((0, 1), (-1, 2))];
    ///
    /// for ((t1, t2), (t3, t4)) in invalid_pairs {
    ///     let i1 = TimeInterval::new(t1, t2).unwrap();
    ///     let i2 = TimeInterval::new(t3, t4).unwrap();
    ///     assert!(!i1.contains(&i2), "{:?} should not contain {:?}", i1, i2);
    /// }
    /// ```
    ///
    pub fn contains(&self, other: &Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    /// Returns whether the given interval intersects this interval
    ///
    /// # Examples
    ///
    /// ```
    /// use geoengine_datatypes::primitives::TimeInterval;
    ///
    /// let valid_pairs = vec![
    ///     ((0, 1), (0, 1)),
    ///     ((0, 3), (1, 2)),
    ///     ((0, 2), (1, 3)),
    ///     ((0, 1), (0, 2)),
    ///     ((0, 2), (-2, 1)),
    /// ];
    ///
    /// for ((t1, t2), (t3, t4)) in valid_pairs {
    ///     let i1 = TimeInterval::new(t1, t2).unwrap();
    ///     let i2 = TimeInterval::new(t3, t4).unwrap();
    ///     assert!(i1.intersects(&i2), "{:?} should intersect {:?}", i1, i2);
    /// }
    ///
    /// let invalid_pairs = vec![
    ///     ((0, 1), (-1, 0)), //
    ///     ((0, 1), (1, 2)),
    ///     ((0, 1), (2, 3)),
    /// ];
    ///
    /// for ((t1, t2), (t3, t4)) in invalid_pairs {
    ///     let i1 = TimeInterval::new(t1, t2).unwrap();
    ///     let i2 = TimeInterval::new(t3, t4).unwrap();
    ///     assert!(
    ///         !i1.intersects(&i2),
    ///         "{:?} should not intersect {:?}",
    ///         i1,
    ///         i2
    ///     );
    /// }
    /// ```
    ///
    pub fn intersects(&self, other: &Self) -> bool {
        self.start < other.end && self.end > other.start
    }

    /// Unites this interval with another one.
    ///
    /// # Examples
    ///
    /// ```
    /// use geoengine_datatypes::primitives::TimeInterval;
    ///
    /// let i1 = TimeInterval::new(0, 2).unwrap();
    /// let i2 = TimeInterval::new(1, 3).unwrap();
    /// let i3 = TimeInterval::new(2, 4).unwrap();
    /// let i4 = TimeInterval::new(3, 5).unwrap();
    ///
    /// assert_eq!(i1.union(&i2).unwrap(), TimeInterval::new(0, 3).unwrap());
    /// assert_eq!(i1.union(&i3).unwrap(), TimeInterval::new(0, 4).unwrap());
    /// i1.union(&i4).unwrap_err();
    /// ```
    ///
    pub fn union(&self, other: &Self) -> Result<Self> {
        if self.intersects(other) || self.start == other.end || self.end == other.start {
            Ok(Self {
                start: i64::min(self.start, other.start),
                end: i64::max(self.end, other.end),
            })
        } else {
            Err(TimeIntervalError::UnmatchedIntervals {
                i1: *self,
                i2: *other,
            }
            .into())
        }
    }
}

impl Debug for TimeInterval {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "TimeInterval [{}, {})", self.start, self.end)
    }
}

impl Display for TimeInterval {
    /// Display the interval in its close-open form
    ///
    /// # Examples
    ///
    /// ```
    /// use geoengine_datatypes::primitives::TimeInterval;
    ///
    /// assert_eq!(format!("{}", TimeInterval::new(0, 1).unwrap()), "[0, 1)");
    /// ```
    ///
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

impl PartialOrd for TimeInterval {
    /// Order intervals whether they are completely before, equal or after each other or in-between (unordered)
    ///
    /// # Examples
    ///
    /// ```
    /// use geoengine_datatypes::primitives::TimeInterval;
    ///
    /// assert_eq!(
    ///     TimeInterval::new(0, 1).unwrap(),
    ///     TimeInterval::new(0, 1).unwrap()
    /// );
    /// assert_ne!(
    ///     TimeInterval::new(0, 1).unwrap(),
    ///     TimeInterval::new(1, 2).unwrap()
    /// );
    ///
    /// assert!(TimeInterval::new(0, 1).unwrap() <= TimeInterval::new(0, 1).unwrap());
    /// assert!(TimeInterval::new(0, 1).unwrap() <= TimeInterval::new(1, 2).unwrap());
    /// assert!(TimeInterval::new(0, 1).unwrap() < TimeInterval::new(1, 2).unwrap());
    ///
    /// assert!(TimeInterval::new(0, 1).unwrap() >= TimeInterval::new(0, 1).unwrap());
    /// assert!(TimeInterval::new(1, 2).unwrap() >= TimeInterval::new(0, 1).unwrap());
    /// assert!(TimeInterval::new(1, 2).unwrap() > TimeInterval::new(0, 1).unwrap());
    ///
    /// assert!(TimeInterval::new(0, 2)
    ///     .unwrap()
    ///     .partial_cmp(&TimeInterval::new(1, 3).unwrap())
    ///     .is_none());
    ///
    /// assert!(TimeInterval::new(0, 1)
    ///     .unwrap()
    ///     .partial_cmp(&TimeInterval::new(0, 2).unwrap())
    ///     .is_none());
    /// ```
    ///
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.eq(other) {
            Some(Ordering::Equal)
        } else if self.end <= other.start {
            Some(Ordering::Less)
        } else if self.start >= other.end {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}
