use crate::ops::DerefMut;
use crate::pin::Pin;
use crate::task::{Context, Poll};

/// Indicates whether an item is available, the current task has been scheduled to receive a wakeup,
/// or the async iterator is done.
///
/// This is returned by [`AsyncIterator::poll_next`].
#[unstable(feature = "async_iterator", issue = "79024")]
#[must_use = "this `PollNext` may be a `Pending` or `Done` variant, which should be handled"]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[lang = "PollNext"]
pub enum PollNext<T> {
    /// The async iterator produced an item.
    #[unstable(feature = "async_iterator", issue = "79024")]
    #[lang = "PollNextItem"]
    Item(#[unstable(feature = "async_iterator", issue = "79024")] T),

    /// An item wasn't immediately available, and a wakeup has been registered.
    #[unstable(feature = "async_iterator", issue = "79024")]
    #[lang = "PollNextPending"]
    Pending,

    /// The async iterator has no more items.
    #[unstable(feature = "async_iterator", issue = "79024")]
    #[lang = "PollNextDone"]
    Done,
}

/// A trait for dealing with asynchronous iterators.
///
/// This is the main async iterator trait. For more about the concept of async iterators
/// generally, please see the [module-level documentation]. In particular, you
/// may want to know how to [implement `AsyncIterator`][impl].
///
/// Like futures, async iterators should be polled continuously from the time they're created until
/// they're either finished or dropped. Unlike futures, there are two different ways to poll an
/// async iterator: [`poll_next`] for when the caller wants to receive another item, and
/// [`poll_progress`] for when they don't. These two poll methods correspond to the the two phases
/// of a `for await` loop:
///
/// ```rust,no_compile
/// for await item in my_iter {
///     do_something(item).await;
/// }
/// ```
///
/// While the loop is waiting to receive an item, it's calling `poll_next` on the iterator. And
/// while it's waiting on an `.await` in its body, it's calling `poll_progress` on the iterator.
/// Assuming it doesn't `break` or `return` early, the loop finishes when `poll_next` returns
/// [`PollNext::Done`]. Either way, once the loop is finished, it drops the iterator.
///
/// Note also that once a `for await` loop starts calling `poll_next`, it keeps calling `poll_next`
/// until it either yields an item or returns `Done`. In other words, it never calls `poll_progress`
/// when the last call to `poll_next` returned `Pending`. This is a rule that all other callers must
/// also follow. This rule mostly affects concurrent combinators that "merge" multiple async
/// iterators together: when one merged iterator yields an item, the combinator must keep driving
/// the others internally until they also yield. This ensures the smooth flow of control through
/// chains of combinators, and it means that many combinators don't need to allocate buffer space
/// for an item.
///
/// To summarize, the `AsyncIterator` contract is:
///
/// 1. An async iterator must be polled promptly after it's created and then continuously (whenever
///    its [`Waker`] is invoked) until it's dropped, using either `poll_next` or `poll_progress`.
/// 2. Once `poll_next` returns `Done`, neither `poll_next` nor `poll_progress` should be called
///    again, and the async iterator should be dropped promptly.
/// 3. `poll_progress` should not be called when the last call to `poll_next` returned `Pending`.
///
/// [module-level documentation]: index.html
/// [impl]: index.html#implementing-async-iterator
/// [`poll_next`]: AsyncIterator::poll_next
/// [`poll_progress`]: AsyncIterator::poll_progress
/// [`Waker`]: crate::task::Waker
#[unstable(feature = "async_iterator", issue = "79024")]
#[must_use = "async iterators do nothing unless polled"]
#[doc(alias = "Stream")]
#[lang = "async_iterator"]
pub trait AsyncIterator {
    /// The type of items yielded by the async iterator.
    type Item;

    /// Attempts to pull out the next item of this async iterator, registering the
    /// current task for wakeup if an item is not yet available, and returning
    /// `Done` if the async iterator is exhausted.
    ///
    /// # Return value
    ///
    /// There are several possible return values, each indicating a distinct
    /// async iterator state:
    ///
    /// - `PollNext::Item(item)` means that the async iterator has successfully produced an item,
    ///   `item`, and may produce further items on subsequent `poll_next` calls. In this case the
    ///   caller must arrange to call either `poll_next` or [`poll_progress`] again promptly.
    ///
    /// - `PollNext::Pending` means that this async iterator's next item is not ready yet.
    ///   Implementations will ensure that the current task will be notified when the next item may
    ///   be ready. The caller must arrange to call `poll_next` (not `poll_progress`) again at that
    ///   time.
    ///
    /// - `PollNext::Done` means that the async iterator has terminated, and the caller should drop
    ///   it promptly. Neither `poll_next` nor `poll_progress` should be invoked again.
    ///
    /// [`poll_progress`]: AsyncIterator::poll_progress
    ///
    /// # Panics
    ///
    /// Once an async iterator has finished (returned `Done` from `poll_next`), calling its
    /// `poll_next` method again may panic, block forever, or cause other kinds of
    /// problems; the `AsyncIterator` trait places no requirements on the effects of
    /// such a call. However, as the `poll_next` method is not marked `unsafe`,
    /// Rust's usual rules apply: calls must never cause undefined behavior
    /// (memory corruption, incorrect use of `unsafe` functions, or the like),
    /// regardless of the async iterator's state.
    #[lang = "async_iterator_poll_next"]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> PollNext<Self::Item>;

    /// Allows this async iterator to make progress internally, even though the next item is not
    /// yet needed.
    ///
    /// When an async iterator reaches a yield point, it often has nothing do until the caller
    /// requests another item via [`poll_next`]. In that case, `poll_progress` returns `Ready`
    /// immediately and has no effect. However, some async iterators have _concurrent_ work they
    /// need to do without waiting for the caller. This includes "buffered" iterators, which try to
    /// prepare some number of items in advance. It also includes async iterators that wrap multiple
    /// futures internally, because futures must be polled continuously. `poll_progress` is how the
    /// caller gives an async iterator a chance to do concurrent work and register wakeups, when the
    /// next item isn't yet needed.
    ///
    /// [`poll_next`]: AsyncIterator::poll_next
    ///
    /// Note that `poll_progress` may be called before any calls to `poll_next` have been made. An
    /// async iterator must be polled continuously once it's created, but callers don't necessarily
    /// need to request the first item immediately the way a `for await` loop does.
    ///
    /// # Return value
    ///
    /// - `Poll::Pending` means that more progress might be possible in the future. Implementations
    ///   will ensure that the current task will be notified when more progress can be made.
    ///
    /// - `Poll::Ready(())` means that the async iterator has made as much internal progress as it
    ///   can, and `poll_progress` does not need to be invoked again until the next time `poll_next`
    ///   returns an item. Continuing to call `poll_progress` is allowed but generally has no
    ///   effect.
    ///
    /// # Panics
    ///
    /// `poll_progress` must not be called after the most recent call to `poll_next` has returned
    /// `Pending` or after any call to `poll_next` has returned `Done`. Calling `poll_progress` in
    /// either of those cases may panic, block forever, or cause other kinds of problems; the
    /// `AsyncIterator` trait places no requirements on the effects of such a call. However, as the
    /// `poll_progress` method is not marked `unsafe`, Rust's usual rules apply: calls must never
    /// cause undefined behavior (memory corruption, incorrect use of `unsafe` functions, or the
    /// like), regardless of the async iterator's state.

    #[lang = "async_iterator_poll_progress"]
    fn poll_progress(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>;

    /// Returns the bounds on the remaining length of the async iterator.
    ///
    /// Specifically, `size_hint()` returns a tuple where the first element
    /// is the lower bound, and the second element is the upper bound.
    ///
    /// The second half of the tuple that is returned is an <code>[Option]<[usize]></code>.
    /// A [`None`] here means that either there is no known upper bound, or the
    /// upper bound is larger than [`usize`].
    ///
    /// # Implementation notes
    ///
    /// It is not enforced that an async iterator implementation yields the declared
    /// number of elements. A buggy async iterator may yield less than the lower bound
    /// or more than the upper bound of elements.
    ///
    /// `size_hint()` is primarily intended to be used for optimizations such as
    /// reserving space for the elements of the async iterator, but must not be
    /// trusted to e.g., omit bounds checks in unsafe code. An incorrect
    /// implementation of `size_hint()` should not lead to memory safety
    /// violations.
    ///
    /// That said, the implementation should provide a correct estimation,
    /// because otherwise it would be a violation of the trait's protocol.
    ///
    /// The default implementation returns <code>(0, [None])</code> which is correct for any
    /// async iterator.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

#[unstable(feature = "async_iterator", issue = "79024")]
impl<S: ?Sized + AsyncIterator + Unpin> AsyncIterator for &mut S {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> PollNext<Self::Item> {
        S::poll_next(Pin::new(&mut **self), cx)
    }

    fn poll_progress(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        S::poll_progress(Pin::new(&mut **self), cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

#[unstable(feature = "async_iterator", issue = "79024")]
impl<P> AsyncIterator for Pin<P>
where
    P: DerefMut,
    P::Target: AsyncIterator,
{
    type Item = <P::Target as AsyncIterator>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> PollNext<Self::Item> {
        <P::Target as AsyncIterator>::poll_next(self.as_deref_mut(), cx)
    }

    fn poll_progress(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        <P::Target as AsyncIterator>::poll_progress(self.as_deref_mut(), cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

#[doc(hidden)]
#[unstable(feature = "async_gen_internals", issue = "none")]
#[lang = "async_iterator_poll_progress_noop"]
pub(super) fn async_iterator_poll_progress_noop<T: ?Sized>(
    _self: Pin<&mut T>,
    _cx: &mut Context<'_>,
) -> Poll<()> {
    Poll::Ready(())
}

#[unstable(feature = "async_iterator", issue = "79024")]
impl<T> PollNext<T> {
    /// Maps a `PollNext<T>` to `PollNext<U>` by applying a function to a contained item.
    ///
    /// # Examples
    ///
    /// Converts a <code>PollNext<[String]></code> into a <code>PollNext<[usize]></code>,
    /// consuming the original:
    ///
    /// [String]: ../../std/string/struct.String.html "String"
    /// ```
    /// #![feature(async_iterator)]
    /// # use core::async_iter::PollNext;
    /// let poll_item_string = PollNext::Item(String::from("Hello, World!"));
    /// // `PollNext::map` takes self *by value*, consuming `poll_item_string`
    /// let poll_item_len = poll_item_string.map(|s| s.len());
    ///
    /// assert_eq!(poll_item_len, PollNext::Item(13));
    /// ```
    #[unstable(feature = "async_iterator", issue = "79024")]
    #[inline]
    pub fn map<U, F>(self, f: F) -> PollNext<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            PollNext::Item(t) => PollNext::Item(f(t)),
            PollNext::Pending => PollNext::Pending,
            PollNext::Done => PollNext::Done,
        }
    }

    /// Returns `true` if this is an [`Item`] value.
    ///
    /// [`Item`]: PollNext::Item
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(async_iterator)]
    /// # use core::async_iter::PollNext;
    /// let x: PollNext<u32> = PollNext::Item(2);
    /// assert_eq!(x.is_item(), true);
    ///
    /// let x: PollNext<u32> = PollNext::Pending;
    /// assert_eq!(x.is_item(), false);
    ///
    /// let x: PollNext<u32> = PollNext::Done;
    /// assert_eq!(x.is_item(), false);
    /// ```
    #[inline]
    #[unstable(feature = "async_iterator", issue = "79024")]
    pub const fn is_item(&self) -> bool {
        matches!(*self, PollNext::Item(_))
    }

    /// Returns `true` if this is a [`Pending`] value.
    ///
    /// [`Pending`]: PollNext::Pending
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(async_iterator)]
    /// # use core::async_iter::PollNext;
    /// let x: PollNext<u32> = PollNext::Item(2);
    /// assert_eq!(x.is_pending(), false);
    ///
    /// let x: PollNext<u32> = PollNext::Pending;
    /// assert_eq!(x.is_pending(), true);
    ///
    /// let x: PollNext<u32> = PollNext::Done;
    /// assert_eq!(x.is_pending(), false);
    /// ```
    #[inline]
    #[unstable(feature = "async_iterator", issue = "79024")]
    pub const fn is_pending(&self) -> bool {
        matches!(*self, PollNext::Pending)
    }

    /// Returns `true` if this is a [`Done`] value.
    ///
    /// [`Done`]: PollNext::Done
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(async_iterator)]
    /// # use core::async_iter::PollNext;
    /// let x: PollNext<u32> = PollNext::Item(2);
    /// assert_eq!(x.is_done(), false);
    ///
    /// let x: PollNext<u32> = PollNext::Pending;
    /// assert_eq!(x.is_done(), false);
    ///
    /// let x: PollNext<u32> = PollNext::Done;
    /// assert_eq!(x.is_done(), true);
    /// ```
    #[inline]
    #[unstable(feature = "async_iterator", issue = "79024")]
    pub const fn is_done(&self) -> bool {
        matches!(*self, PollNext::Done)
    }
}

#[unstable(feature = "async_gen_internals", issue = "none")]
impl<T> PollNext<T> {
    /// A helper function for internal desugaring -- produces `Item(t)`,
    /// which corresponds to the async iterator yielding a value.
    #[doc(hidden)]
    #[unstable(feature = "async_gen_internals", issue = "none")]
    #[lang = "AsyncGenReady"]
    pub fn async_gen_ready(t: T) -> Self {
        PollNext::Item(t)
    }

    /// A helper constant for internal desugaring -- produces `Pending`,
    /// which corresponds to the async iterator pending on an `.await`.
    #[doc(hidden)]
    #[unstable(feature = "async_gen_internals", issue = "none")]
    #[lang = "AsyncGenPending"]
    // FIXME(gen_blocks): This probably could be deduplicated.
    pub const PENDING: Self = PollNext::Pending;

    /// A helper constant for internal desugaring -- produces `Done`,
    /// which corresponds to the async iterator finishing its iteration.
    #[doc(hidden)]
    #[unstable(feature = "async_gen_internals", issue = "none")]
    #[lang = "AsyncGenFinished"]
    pub const FINISHED: Self = PollNext::Done;
}

/// Converts something into an async iterator
#[unstable(feature = "async_iterator", issue = "79024")]
pub trait IntoAsyncIterator {
    /// The type of the item yielded by the iterator
    type Item;
    /// The type of the resulting iterator
    type IntoAsyncIter: AsyncIterator<Item = Self::Item>;

    /// Converts `self` into an async iterator
    #[lang = "into_async_iter_into_iter"]
    fn into_async_iter(self) -> Self::IntoAsyncIter;
}

#[unstable(feature = "async_iterator", issue = "79024")]
impl<I: AsyncIterator> IntoAsyncIterator for I {
    type Item = I::Item;
    type IntoAsyncIter = I;

    fn into_async_iter(self) -> Self::IntoAsyncIter {
        self
    }
}
