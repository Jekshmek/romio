use super::clock;
use super::Delay;
use super::Error;

use futures::{Future, Stream, Poll, ready};
use futures::task::LocalWaker;

use std::time::{Instant, Duration};
use std::pin::Pin;

/// A stream representing notifications at fixed interval
#[derive(Debug)]
pub struct Interval {
    /// Future that completes the next time the `Interval` yields a value.
    delay: Delay,

    /// The duration between values yielded by `Interval`.
    duration: Duration,
}

impl Interval {
    /// Create a new `Interval` that starts at `at` and yields every `duration`
    /// interval after that.
    ///
    /// Note that when it starts, it produces item too.
    ///
    /// The `duration` argument must be a non-zero duration.
    ///
    /// # Panics
    ///
    /// This function panics if `duration` is zero.
    pub fn new(at: Instant, duration: Duration) -> Interval {
        assert!(duration > Duration::new(0, 0), "`duration` must be non-zero.");

        Interval::new_with_delay(Delay::new(at), duration)
    }

    /// Creates new `Interval` that yields with interval of `duration`.
    /// /// The function is shortcut for `Interval::new(Instant::now() + duration, duration)`.
    ///
    /// The `duration` argument must be a non-zero duration.
    ///
    /// # Panics
    ///
    /// This function panics if `duration` is zero.
    pub fn new_interval(duration: Duration) -> Interval {
        Interval::new(clock::now() + duration, duration)
    }

    pub(crate) fn new_with_delay(delay: Delay, duration: Duration) -> Interval {
        Interval {
            delay,
            duration,
        }
    }

    fn delay<'a>(self: Pin<&'a mut Self>) -> Pin<&'a mut Delay> {
        unsafe { Pin::map_unchecked_mut(self, |this| &mut this.delay) }
    }
}

impl Stream for Interval {
    type Item = Result<Instant, Error>;

    fn poll_next(mut self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Option<Self::Item>> {
        // Wait for the delay to be done
        let _ = ready!(self.as_mut().delay().poll(lw)?);

        // Get the `now` by looking at the `delay` deadline
        let now = self.delay.deadline();

        // The next interval value is `duration` after the one that just
        // yielded.
        let delay = now + self.duration;
        self.delay.reset(delay);

        // Return the current instant
        Poll::Ready(Some(Ok(now)))
    }
}
