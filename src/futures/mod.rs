use std::{
    cmp::Ordering,
    collections::{binary_heap::PeekMut, BinaryHeap},
    iter::FromIterator,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, stream::FuturesUnordered, Future, Stream};
use pin_project::pin_project;

#[pin_project]
#[must_use = "futures do nothing unless you `.await` or poll them"]
struct OrderWrapper<T> {
    #[pin]
    data: T, // A future or a future's output
    index: usize,
}

impl<T> PartialEq for OrderWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for OrderWrapper<T> {}

impl<T> PartialOrd for OrderWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Delegate to `Ord::cmp`
        Some(self.cmp(other))
    }
}

impl<T> Ord for OrderWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max heap, so compare backwards here.
        other.index.cmp(&self.index)
    }
}

impl<T: Future> Future for OrderWrapper<T> {
    type Output = OrderWrapper<T::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let index = self.index;
        self.project().data.poll(cx).map(|output| OrderWrapper {
            data: output,
            index,
        })
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct AllSome<F, T>
where
    F: Future<Output = Option<T>>,
{
    in_progress_queue: FuturesUnordered<OrderWrapper<F>>,
    queued_outputs: BinaryHeap<OrderWrapper<T>>,
    output: Vec<T>,
    next_output_index: usize,
}

impl<F, T> AllSome<F, T>
where
    F: Future<Output = Option<T>>,
{
    fn finish(mut self: Pin<&mut Self>) -> Vec<T> {
        let this = &mut *self;
        std::mem::take(&mut this.output)
    }

    fn push(&mut self, future: F) {
        let wrapped = OrderWrapper {
            data: future,
            index: self.in_progress_queue.len(),
        };
        self.in_progress_queue.push(wrapped);
    }
}

impl<F, T> Default for AllSome<F, T>
where
    F: Future<Output = Option<T>>,
{
    fn default() -> Self {
        Self {
            in_progress_queue: Default::default(),
            queued_outputs: Default::default(),
            output: Default::default(),
            next_output_index: 0,
        }
    }
}

impl<F, T> FromIterator<F> for AllSome<F, T>
where
    F: Future<Output = Option<T>>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = F>,
    {
        let acc = AllSome::default();
        iter.into_iter().fold(acc, |mut acc, item| {
            acc.push(item);
            acc
        })
    }
}

impl<F, T> Future for AllSome<F, T>
where
    F: Future<Output = Option<T>>,
{
    type Output = Option<Vec<T>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        loop {
            // Check to see if we've already received the next value
            while let Some(next_output) = this.queued_outputs.peek_mut() {
                if next_output.index != this.next_output_index {
                    break;
                }

                this.next_output_index += 1;
                this.output.extend(Some(PeekMut::pop(next_output).data));
            }

            match ready!(Pin::new(&mut this.in_progress_queue).poll_next(cx)) {
                Some(OrderWrapper { data, index }) => {
                    match data {
                        // The next future from `self.in_progress_queue` returned
                        // None. This short-circuits waiting for any more results
                        // and `AllSome` is done.
                        None => return None.into(),
                        Some(data) => {
                            if index == this.next_output_index {
                                this.next_output_index += 1;
                                this.output.extend(Some(data));
                            } else {
                                this.queued_outputs.push(OrderWrapper { data, index })
                            }
                        }
                    }
                }
                None => return Some(self.finish()).into(),
            }
        }
    }
}

impl<F, T> Unpin for AllSome<F, T> where F: Future<Output = Option<T>> {}
