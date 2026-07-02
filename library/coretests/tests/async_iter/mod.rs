use core::async_iter::{self, AsyncIterator, IntoAsyncIterator, PollNext};
use core::pin::pin;
use core::task::Poll;

#[test]
fn into_async_iter() {
    let async_iter = async_iter::from_iter(0..3);
    let mut async_iter = pin!(async_iter.into_async_iter());

    let mut cx = &mut core::task::Context::from_waker(core::task::Waker::noop());

    assert_eq!(async_iter.as_mut().poll_progress(&mut cx), Poll::Ready(()));
    assert_eq!(async_iter.as_mut().poll_next(&mut cx), PollNext::Item(0));
    assert_eq!(async_iter.as_mut().poll_next(&mut cx), PollNext::Item(1));
    assert_eq!(async_iter.as_mut().poll_next(&mut cx), PollNext::Item(2));
    assert_eq!(async_iter.as_mut().poll_next(&mut cx), PollNext::Done);
}

#[test]
fn poll_next_map() {
    assert_eq!(PollNext::Item("hello").map(str::len), PollNext::Item(5));

    let pending: PollNext<&str> = PollNext::Pending;
    assert_eq!(pending.map(str::len), PollNext::Pending);

    let done: PollNext<&str> = PollNext::Done;
    assert_eq!(done.map(str::len), PollNext::Done);
}

#[test]
fn poll_next_is_item_is_pending_and_is_done() {
    assert!(PollNext::Item(2).is_item());
    assert!(!PollNext::<u32>::Pending.is_item());
    assert!(!PollNext::<u32>::Done.is_item());

    assert!(!PollNext::Item(2).is_pending());
    assert!(PollNext::<u32>::Pending.is_pending());
    assert!(!PollNext::<u32>::Done.is_pending());

    assert!(!PollNext::Item(2).is_done());
    assert!(!PollNext::<u32>::Pending.is_done());
    assert!(PollNext::<u32>::Done.is_done());
}

#[test]
fn poll_next_const() {
    const ITEM: PollNext<usize> = PollNext::Item(0);
    const PENDING: PollNext<usize> = PollNext::Pending;
    const DONE: PollNext<usize> = PollNext::Done;

    const IS_ITEM: bool = ITEM.is_item();
    assert!(IS_ITEM);

    const IS_PENDING: bool = PENDING.is_pending();
    assert!(IS_PENDING);

    const DONE_IS_ITEM: bool = DONE.is_item();
    assert!(!DONE_IS_ITEM);

    const DONE_IS_PENDING: bool = DONE.is_pending();
    assert!(!DONE_IS_PENDING);

    const DONE_IS_DONE: bool = DONE.is_done();
    assert!(DONE_IS_DONE);
}
