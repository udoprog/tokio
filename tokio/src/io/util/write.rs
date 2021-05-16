use crate::io::AsyncWrite;

use pin_project::{pin_project, pinned_drop};
use std::future::Future;
use std::io;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A future to write some of the buffer to an `AsyncWrite`.
#[pin_project(PinnedDrop)]
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Write<'a, W> where W: AsyncWrite + Unpin + ?Sized {
    writer: &'a mut W,
    buf: &'a [u8],
    // Make this future `!Unpin` for compatibility with async trait methods.
    #[pin]
    _pin: PhantomPinned,
}

/// Tries to write some bytes from the given `buf` to the writer in an
/// asynchronous manner, returning a future.
pub(crate) fn write<'a, W>(writer: &'a mut W, buf: &'a [u8]) -> Write<'a, W>
where
    W: AsyncWrite + Unpin + ?Sized,
{
    Write {
        writer,
        buf,
        _pin: PhantomPinned,
    }
}

impl<W> Future for Write<'_, W>
where
    W: AsyncWrite + Unpin + ?Sized,
{
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<usize>> {
        let me = self.project();
        Pin::new(&mut *me.writer).poll_write(cx, me.buf)
    }
}

#[pinned_drop]
impl<W> PinnedDrop for Write<'_, W> where W: AsyncWrite + Unpin + ?Sized {
    fn drop(self: Pin<&mut Self>) {
        let me = self.project();
        Pin::new(&mut **me.writer).cancel_pending_writes();
    }
}
