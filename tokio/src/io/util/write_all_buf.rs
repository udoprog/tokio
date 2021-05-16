use crate::io::AsyncWrite;

use bytes::Buf;
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
pub struct WriteAllBuf<'a, W, B> where W: AsyncWrite + Unpin {
    writer: &'a mut W,
    buf: &'a mut B,
    #[pin]
    _pin: PhantomPinned,
}

/// Tries to write some bytes from the given `buf` to the writer in an
/// asynchronous manner, returning a future.
pub(crate) fn write_all_buf<'a, W, B>(writer: &'a mut W, buf: &'a mut B) -> WriteAllBuf<'a, W, B>
where
    W: AsyncWrite + Unpin,
    B: Buf,
{
    WriteAllBuf {
        writer,
        buf,
        _pin: PhantomPinned,
    }
}

impl<W, B> Future for WriteAllBuf<'_, W, B>
where
    W: AsyncWrite + Unpin,
    B: Buf,
{
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let me = self.project();
        while me.buf.has_remaining() {
            let n = ready!(Pin::new(&mut *me.writer).poll_write(cx, me.buf.chunk())?);
            me.buf.advance(n);
            if n == 0 {
                return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
            }
        }

        Poll::Ready(Ok(()))
    }
}

#[pinned_drop]
impl<W, B> PinnedDrop for WriteAllBuf<'_, W, B> where W: AsyncWrite + Unpin {
    fn drop(self: Pin<&mut Self>) {
        let me = self.project();
        Pin::new(&mut **me.writer).cancel_pending_writes();
    }
}
