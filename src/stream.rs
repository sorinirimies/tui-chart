//! Fixed-capacity streaming helpers for live charts.

use crate::line::Series;
use std::collections::VecDeque;

/// Fixed-capacity FIFO ring buffer; pushing past capacity drops the oldest item.
#[derive(Debug, Clone, PartialEq)]
pub struct RingBuffer<T> {
    buf: VecDeque<T>,
    cap: usize,
}

impl<T> RingBuffer<T> {
    /// Create with a capacity (minimum 1).
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        Self {
            buf: VecDeque::with_capacity(cap.min(4096)),
            cap,
        }
    }

    /// Append an item, returning the evicted oldest item if full.
    pub fn push(&mut self, item: T) -> Option<T> {
        let ev = if self.buf.len() == self.cap {
            self.buf.pop_front()
        } else {
            None
        };
        self.buf.push_back(item);
        ev
    }

    /// Iterate oldest → newest.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator {
        self.buf.iter()
    }

    /// Number of stored items.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Maximum number of items.
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Newest item.
    pub fn last(&self) -> Option<&T> {
        self.buf.back()
    }

    /// Remove all items.
    pub fn clear(&mut self) {
        self.buf.clear();
    }
}

impl<T> Extend<T> for RingBuffer<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for i in iter {
            self.push(i);
        }
    }
}

/// A rolling window of `f64` samples with monotonically increasing x positions.
///
/// Each pushed sample gets `x = x0 + n * dx` where `n` counts every sample ever
/// pushed, so the x values keep increasing as old samples scroll out.
#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    ring: RingBuffer<f64>,
    pushed: u64,
    x0: f64,
    dx: f64,
}

impl Stream {
    /// Create a stream keeping the last `capacity` samples (x step 1, start 0).
    pub fn new(capacity: usize) -> Self {
        Self {
            ring: RingBuffer::new(capacity),
            pushed: 0,
            x0: 0.0,
            dx: 1.0,
        }
    }

    /// X of the first sample ever pushed.
    pub fn x_start(mut self, x0: f64) -> Self {
        self.x0 = if x0.is_finite() { x0 } else { 0.0 };
        self
    }

    /// X distance between consecutive samples (non-positive / non-finite → 1).
    pub fn x_step(mut self, dx: f64) -> Self {
        self.dx = if dx.is_finite() && dx > 0.0 { dx } else { 1.0 };
        self
    }

    /// Push a sample (non-finite values are stored as NaN and skipped by min/max).
    pub fn push(&mut self, y: f64) {
        self.ring.push(if y.is_finite() { y } else { f64::NAN });
        self.pushed += 1;
    }

    /// Number of samples currently held.
    pub fn len(&self) -> usize {
        self.ring.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    /// Iterate stored y values, oldest first.
    pub fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.ring.iter().copied()
    }

    /// Iterate `(x, y)` points, oldest first; x is strictly increasing.
    pub fn points(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        let first = self.pushed - self.ring.len() as u64;
        self.ring
            .iter()
            .enumerate()
            .map(move |(i, &y)| (self.x0 + (first + i as u64) as f64 * self.dx, y))
    }

    /// `(min, max)` of finite y values, `None` if there are none.
    pub fn y_range(&self) -> Option<(f64, f64)> {
        self.iter().filter(|v| v.is_finite()).fold(None, |a, v| {
            Some(a.map_or((v, v), |(lo, hi)| (lo.min(v), hi.max(v))))
        })
    }

    /// `(first x, last x)` of the current window.
    pub fn x_range(&self) -> Option<(f64, f64)> {
        let mut p = self.points();
        let a = p.next()?;
        Some((a.0, p.last().map_or(a.0, |l| l.0)))
    }

    /// Build a [`Series`] from the current window.
    pub fn series(&self, name: impl Into<String>) -> Series {
        Series::new(name).data(self.points())
    }
}

impl From<&Stream> for Series {
    fn from(s: &Stream) -> Self {
        s.series("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_evicts_oldest() {
        let mut r = RingBuffer::new(3);
        assert!(r.is_empty());
        for i in 0..3 {
            assert_eq!(r.push(i), None);
        }
        assert_eq!(r.push(3), Some(0));
        assert_eq!(r.iter().copied().collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(r.last(), Some(&3));
        assert_eq!(r.capacity(), 3);
        r.extend([7, 8]);
        assert_eq!(r.len(), 3);
        r.clear();
        assert!(r.is_empty());
        assert_eq!(RingBuffer::<u8>::new(0).capacity(), 1);
    }

    #[test]
    fn x_monotonic_after_wrap() {
        let mut s = Stream::new(4).x_start(10.0).x_step(0.5);
        for i in 0..10 {
            s.push(i as f64);
        }
        let p: Vec<_> = s.points().collect();
        assert_eq!(p.len(), 4);
        assert_eq!(p[0], (13.0, 6.0));
        assert_eq!(p[3], (14.5, 9.0));
        assert!(p.windows(2).all(|w| w[1].0 > w[0].0));
        assert_eq!(s.x_range(), Some((13.0, 14.5)));
        assert_eq!(s.y_range(), Some((6.0, 9.0)));
    }

    #[test]
    fn nan_and_empty() {
        let mut s = Stream::new(3).x_step(f64::NAN);
        assert_eq!(s.y_range(), None);
        assert_eq!(s.x_range(), None);
        s.push(f64::INFINITY);
        s.push(2.0);
        assert_eq!(s.y_range(), Some((2.0, 2.0)));
    }

    #[test]
    fn into_series() {
        let mut s = Stream::new(5);
        s.push(1.0);
        s.push(2.0);
        let ser = s.series("a");
        assert_eq!(ser.len(), 2);
        assert_eq!(Series::from(&s).len(), 2);
    }
}
