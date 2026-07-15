use std::io;

use axum::{
    body::{Body, Bytes},
    response::{IntoResponse, Response},
};
use futures_util::{Stream, stream};
use kernel::excel::TemporaryXlsxFile;
use tokio::io::{AsyncRead, AsyncReadExt};

const XLSX_CONTENT_TYPE: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
const STREAM_CHUNK_BYTES: usize = 64 * 1024;

struct GuardedReader<R, G> {
    reader: R,
    _cleanup: G,
    finished: bool,
}

impl<R, G> GuardedReader<R, G> {
    fn new(reader: R, cleanup: G) -> Self {
        Self {
            reader,
            _cleanup: cleanup,
            finished: false,
        }
    }
}

pub fn xlsx_attachment(file_name: &str, bytes: Vec<u8>) -> Response {
    attachment_headers(file_name, bytes.len() as u64, bytes).into_response()
}

pub fn xlsx_file_attachment(file_name: &str, artifact: TemporaryXlsxFile) -> Response {
    let (file, content_length, cleanup) = artifact.into_stream_parts();
    let reader = tokio::fs::File::from_std(file);
    let stream = guarded_reader_stream(GuardedReader::new(reader, cleanup));
    attachment_headers(file_name, content_length, Body::from_stream(stream)).into_response()
}

fn attachment_headers<T>(file_name: &str, content_length: u64, body: T) -> ([(&'static str, String); 3], T) {
    let headers = [
        ("content-type", XLSX_CONTENT_TYPE.to_owned()),
        ("content-disposition", format!("attachment; filename=\"{file_name}\"")),
        ("content-length", content_length.to_string()),
    ];
    (headers, body)
}

fn guarded_reader_stream<R, G>(state: GuardedReader<R, G>) -> impl Stream<Item = Result<Bytes, io::Error>>
where
    R: AsyncRead + Unpin + Send + 'static,
    G: Send + 'static,
{
    stream::unfold(state, read_next_chunk)
}

async fn read_next_chunk<R, G>(mut state: GuardedReader<R, G>) -> Option<(Result<Bytes, io::Error>, GuardedReader<R, G>)>
where
    R: AsyncRead + Unpin,
{
    if state.finished {
        return None;
    }
    let mut bytes = vec![0; STREAM_CHUNK_BYTES];
    match state.reader.read(&mut bytes).await {
        Ok(0) => None,
        Ok(read) => {
            bytes.truncate(read);
            Some((Ok(Bytes::from(bytes)), state))
        }
        Err(error) => {
            state.finished = true;
            Some((Err(error), state))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    use http_body_util::BodyExt;
    use kernel::excel::{StreamingXlsxWriter, read_xlsx};
    use tokio::io::{AsyncRead, ReadBuf};

    use super::*;

    #[tokio::test]
    async fn streams_xlsx_with_headers_and_cleans_after_completion() {
        let artifact = xlsx_artifact();
        let path = artifact.path().to_owned();
        let expected_length = artifact.content_length();

        let response = xlsx_file_attachment("users.xlsx", artifact);
        assert_xlsx_headers(&response, "users.xlsx", expected_length);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();

        assert_eq!(u64::try_from(bytes.len()).unwrap(), expected_length);
        assert_eq!(read_xlsx(&bytes).unwrap()[1], vec!["alice"]);
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn dropping_response_cleans_an_unread_xlsx() {
        let artifact = xlsx_artifact();
        let path = artifact.path().to_owned();

        drop(xlsx_file_attachment("users.xlsx", artifact));

        assert!(!path.exists());
    }

    #[tokio::test]
    async fn read_error_drops_the_stream_cleanup_guard() {
        let artifact = xlsx_artifact();
        let path = artifact.path().to_owned();
        let (file, _, guard) = artifact.into_stream_parts();
        drop(file);
        let stream = guarded_reader_stream(GuardedReader::new(FailingReader, guard));

        assert!(Body::from_stream(stream).collect().await.is_err());
        assert!(!path.exists());
    }

    fn xlsx_artifact() -> TemporaryXlsxFile {
        let mut writer = StreamingXlsxWriter::new("users", &["name"]).unwrap();
        writer.append_rows(&[vec!["alice".into()]]).unwrap();
        writer.finish().unwrap()
    }

    fn assert_xlsx_headers(response: &Response, file_name: &str, content_length: u64) {
        assert_eq!(response.headers()["content-type"], XLSX_CONTENT_TYPE);
        assert_eq!(response.headers()["content-disposition"], format!("attachment; filename=\"{file_name}\""));
        assert_eq!(response.headers()["content-length"], content_length.to_string());
    }

    struct FailingReader;

    impl AsyncRead for FailingReader {
        fn poll_read(self: Pin<&mut Self>, _context: &mut Context<'_>, _buffer: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::other("injected read failure")))
        }
    }
}
