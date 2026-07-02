//@ run-pass
//@ edition: 2021

#![feature(async_for_loop, async_iterator)]

use std::async_iter::{AsyncIterator, PollNext};
use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

struct Iter {
    yielded: bool,
    progress_calls: Rc<Cell<usize>>,
}

impl AsyncIterator for Iter {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> PollNext<Self::Item> {
        if self.yielded {
            PollNext::Done
        } else {
            self.yielded = true;
            PollNext::Item(())
        }
    }

    fn poll_progress(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        let calls = self.progress_calls.get();
        self.progress_calls.set(calls + 1);
        Poll::Ready(())
    }
}

struct PendingOnce {
    pending: bool,
}

impl Future for PendingOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.pending {
            self.pending = false;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

struct PendingOnceIter {
    pending: bool,
    yielded: bool,
}

impl AsyncIterator for PendingOnceIter {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> PollNext<Self::Item> {
        if self.pending {
            self.pending = false;
            cx.waker().wake_by_ref();
            PollNext::Pending
        } else if self.yielded {
            PollNext::Done
        } else {
            self.yielded = true;
            PollNext::Item(())
        }
    }

    fn poll_progress(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Ready(())
    }
}

async fn await_in_for_await_body(progress_calls: Rc<Cell<usize>>) {
    for await () in (Iter { yielded: false, progress_calls }) {
        PendingOnce { pending: true }.await;
    }
}

async fn nested_for_await_body(progress_calls: Rc<Cell<usize>>) {
    for await () in (Iter { yielded: false, progress_calls }) {
        for await () in (PendingOnceIter { pending: true, yielded: false }) {}
    }
}

async fn five_nested_for_await_bodies(progress_calls: [Rc<Cell<usize>>; 5]) {
    for await () in (Iter {
        yielded: false,
        progress_calls: progress_calls[0].clone(),
    }) {
        for await () in (Iter {
            yielded: false,
            progress_calls: progress_calls[1].clone(),
        }) {
            for await () in (Iter {
                yielded: false,
                progress_calls: progress_calls[2].clone(),
            }) {
                for await () in (Iter {
                    yielded: false,
                    progress_calls: progress_calls[3].clone(),
                }) {
                    for await () in (Iter {
                        yielded: false,
                        progress_calls: progress_calls[4].clone(),
                    }) {
                        PendingOnce { pending: true }.await;
                    }
                }
            }
        }
    }
}

fn assert_progress_before_suspend(
    future: impl Future<Output = ()>,
    progress_calls: &[Rc<Cell<usize>>],
    expected_calls: &[usize],
) {
    let mut future = std::pin::pin!(future);
    let mut cx = Context::from_waker(std::task::Waker::noop());

    assert!(future.as_mut().poll(&mut cx).is_pending());
    assert_eq!(progress_calls.len(), expected_calls.len());
    for (progress_calls, expected_calls) in progress_calls.iter().zip(expected_calls) {
        assert_eq!(progress_calls.get(), *expected_calls);
    }

    assert!(future.as_mut().poll(&mut cx).is_ready());
    for (progress_calls, expected_calls) in progress_calls.iter().zip(expected_calls) {
        assert_eq!(progress_calls.get(), *expected_calls);
    }
}

fn main() {
    let progress_calls = Rc::new(Cell::new(0));
    assert_progress_before_suspend(
        await_in_for_await_body(progress_calls.clone()),
        std::slice::from_ref(&progress_calls),
        &[1],
    );

    let progress_calls = Rc::new(Cell::new(0));
    assert_progress_before_suspend(
        nested_for_await_body(progress_calls.clone()),
        std::slice::from_ref(&progress_calls),
        &[1],
    );

    let progress_calls = std::array::from_fn(|_| Rc::new(Cell::new(0)));
    assert_progress_before_suspend(
        five_nested_for_await_bodies(progress_calls.clone()),
        &progress_calls,
        &[1, 1, 1, 1, 1],
    );
}
