//! Writer trait and it's implements.
use async_trait::async_trait;
use serde::Serialize;

use crate::http::header::{HeaderValue, CONTENT_TYPE};
use crate::http::{Request, Response, StatusError};
use crate::Depot;

/// Writer is used to write data to response.
#[async_trait]
pub trait Writer {
    /// Write data to [`Response`].
    #[must_use = "write future must be used"]
    async fn write(mut self, req: &mut Request, depot: &mut Depot, res: &mut Response);
}

#[async_trait]
impl<T, E> Writer for Result<T, E>
where
    T: Writer + Send,
    E: Writer + Send,
{
    #[inline]
    async fn write(mut self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        match self {
            Ok(v) => {
                v.write(req, depot, res).await;
            }
            Err(e) => {
                e.write(req, depot, res).await;
            }
        }
    }
}

/// `Piece` is used to write data to [`Response`].
///
/// `Piece` is simpler than [`Writer`] ant it implements [`Writer`].
pub trait Piece {
    /// Render data to [`Response`].
    fn render(self, res: &mut Response);
}
#[async_trait]
impl<P> Writer for P
where
    P: Piece + Sized + Send,
{
    #[inline]
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        self.render(res)
    }
}

#[allow(clippy::unit_arg)]
impl Piece for () {
    #[inline]
    fn render(self, _res: &mut Response) {}
}
impl Piece for &'static str {
    #[inline]
    fn render(self, res: &mut Response) {
        res.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"));
        res.write_body(self).ok();
    }
}
impl<'a> Piece for &'a String {
    #[inline]
    fn render(self, res: &mut Response) {
        res.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"));
        res.write_body(self.as_bytes().to_vec()).ok();
    }
}
impl Piece for String {
    #[inline]
    fn render(self, res: &mut Response) {
        res.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"));
        res.write_body(self).ok();
    }
}

/// Write text content to response as text content.
pub enum Text<C> {
    /// It will set ```content-type``` to ```text/plain; charset=utf-8```.
    Plain(C),
    /// It will set ```content-type``` to ```application/json; charset=utf-8```.
    Json(C),
    /// It will set ```content-type``` to ```application/xml; charset=utf-8```.
    Xml(C),
    /// It will set ```content-type``` to ```text/html; charset=utf-8```.
    Html(C),
    /// It will set ```content-type``` to ```text/javascript; charset=utf-8```.
    Js(C),
    /// It will set ```content-type``` to ```text/css; charset=utf-8```.
    Css(C),
}

impl<C> Text<C>
where
    C: AsRef<str>,
{
    fn set_header(self, res: &mut Response) -> C {
        let (ctype, content) = match self {
            Self::Plain(content) => (HeaderValue::from_static("text/plain; charset=utf-8"), content),
            Self::Json(content) => (HeaderValue::from_static("application/json; charset=utf-8"), content),
            Self::Xml(content) => (HeaderValue::from_static("application/xml; charset=utf-8"), content),
            Self::Html(content) => (HeaderValue::from_static("text/html; charset=utf-8"), content),
            Self::Js(content) => (HeaderValue::from_static("text/javascript; charset=utf-8"), content),
            Self::Css(content) => (HeaderValue::from_static("text/css; charset=utf-8"), content),
        };
        res.headers_mut().insert(CONTENT_TYPE, ctype);
        content
    }
}
impl Piece for Text<&'static str> {
    #[inline]
    fn render(self, res: &mut Response) {
        let content = self.set_header(res);
        res.write_body(content).ok();
    }
}
impl Piece for Text<String> {
    #[inline]
    fn render(self, res: &mut Response) {
        let content = self.set_header(res);
        res.write_body(content).ok();
    }
}
impl<'a> Piece for Text<&'a String> {
    #[inline]
    fn render(self, res: &mut Response) {
        let content = self.set_header(res);
        res.write_body(content.as_bytes().to_vec()).ok();
    }
}

/// Write serializable content to response as json content. It will set ```content-type``` to ```application/json; charset=utf-8```.
pub struct Json<T>(pub T);
#[async_trait]
impl<T> Piece for Json<T>
where
    T: Serialize + Send,
{
    #[inline]
    fn render(self, res: &mut Response) {
        match serde_json::to_vec(&self.0) {
            Ok(bytes) => {
                res.headers_mut().insert(
                    CONTENT_TYPE,
                    HeaderValue::from_static("application/json; charset=utf-8"),
                );
                res.write_body(bytes).ok();
            }
            Err(e) => {
                tracing::error!(error = ?e, "JsonContent write error");
                res.set_status_error(StatusError::internal_server_error());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use super::*;
    use crate::test::{ResponseExt, TestClient};

    #[tokio::test]
    async fn test_write_str() {
        #[fn_handler(internal)]
        async fn test() -> &'static str {
            "hello"
        }

        let router = Router::new().push(Router::with_path("test").get(test));

        let mut res = TestClient::get("http://127.0.0.1:7878/test").send(router).await;
        assert_eq!(res.take_string().await.unwrap(), "hello");
        assert_eq!(res.headers().get("content-type").unwrap(), "text/plain; charset=utf-8");
    }

    #[tokio::test]
    async fn test_write_string() {
        #[fn_handler(internal)]
        async fn test() -> String {
            "hello".to_owned()
        }

        let router = Router::new().push(Router::with_path("test").get(test));
        let mut res = TestClient::get("http://127.0.0.1:7878/test").send(router).await;
        assert_eq!(res.take_string().await.unwrap(), "hello");
        assert_eq!(res.headers().get("content-type").unwrap(), "text/plain; charset=utf-8");
    }

    #[tokio::test]
    async fn test_write_plain_text() {
        #[fn_handler(internal)]
        async fn test() -> Text<&'static str> {
            Text::Plain("hello")
        }

        let router = Router::new().push(Router::with_path("test").get(test));

        let mut res = TestClient::get("http://127.0.0.1:7878/test").send(router).await;
        assert_eq!(res.take_string().await.unwrap(), "hello");
        assert_eq!(res.headers().get("content-type").unwrap(), "text/plain; charset=utf-8");
    }

    #[tokio::test]
    async fn test_write_json_text() {
        #[fn_handler(internal)]
        async fn test() -> Text<&'static str> {
            Text::Json(r#"{"hello": "world"}"#)
        }

        let router = Router::new().push(Router::with_path("test").get(test));
        let mut res = TestClient::get("http://127.0.0.1:7878/test").send(router).await;
        assert_eq!(res.take_string().await.unwrap(), r#"{"hello": "world"}"#);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "application/json; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn test_write_json_content() {
        #[derive(Serialize, Debug)]
        struct User {
            name: String,
        }
        #[fn_handler(internal)]
        async fn test() -> Json<User> {
            Json(User { name: "jobs".into() })
        }

        let router = Router::new().push(Router::with_path("test").get(test));
        let mut res = TestClient::get("http://127.0.0.1:7878/test").send(router).await;
        assert_eq!(res.take_string().await.unwrap(), r#"{"name":"jobs"}"#);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "application/json; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn test_write_html_text() {
        #[fn_handler(internal)]
        async fn test() -> Text<&'static str> {
            Text::Html("<html><body>hello</body></html>")
        }

        let router = Router::new().push(Router::with_path("test").get(test));
        let mut res = TestClient::get("http://127.0.0.1:7878/test").send(router).await;
        assert_eq!(res.take_string().await.unwrap(), "<html><body>hello</body></html>");
        assert_eq!(res.headers().get("content-type").unwrap(), "text/html; charset=utf-8");
    }
}
