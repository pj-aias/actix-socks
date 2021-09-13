use std::borrow::Cow;
use std::convert::TryFrom;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_connect::{Connect, ConnectError, Connection};
use actix_http::http::Uri;
use actix_service::Service;
use tokio_socks::tcp::Socks5Stream;
use tokio_socks::{Error as SocksConnectError, TargetAddr, ToProxyAddrs};

fn to_connect_error(e: SocksConnectError) -> ConnectError {
    ConnectError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
}

async fn connect_socks<P: ToProxyAddrs + 'static>(
    proxy: P,
    req: Connect<Uri>,
) -> Result<Connection<Uri, Socks5Stream>, ConnectError> {
    let res = tokio_socks::tcp::Socks5Stream::connect(
        proxy,
        TargetAddr::Domain(Cow::Borrowed(req.host()), req.port()),
    )
    .await
    .map_err(to_connect_error)?;
    Ok(Connection::new(
        res,
        Uri::try_from(format!("{}:{}", req.host(), req.port())).unwrap(), // since req.req is private >:(
    ))
}

#[derive(Clone, Debug)]
pub struct SocksConnector<P: ToProxyAddrs>(P);

impl<P: ToProxyAddrs> SocksConnector<P> {
    pub fn new(proxy: P) -> Self {
        SocksConnector(proxy)
    }
}

impl<P> Service for SocksConnector<P>
where
    P: ToProxyAddrs + Copy + 'static,
{
    type Request = Connect<Uri>;
    type Response = Connection<Uri, Socks5Stream>;
    type Error = ConnectError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        Box::pin(connect_socks(self.0, req))
    }
}

#[cfg(test)]
mod test {
    #[actix_rt::test]
    async fn test() {
        let client = actix_web::client::ClientBuilder::new()
            .connector(
                actix_web::client::Connector::new()
                    .connector(crate::SocksConnector("127.0.0.1:9050"))
                    .timeout(std::time::Duration::from_secs(60))
                    .finish(),
            )
            .finish();
        let mut res = client
            .get("http://igxf3reqx3hfwfxdvczxkx474iukh6okpwaffro366vh7ijk22cwuzyd.onion")
            .send()
            .await
            .expect("request error");

        let body = res.body().await.expect("parse error");

        let body = String::from_utf8_lossy(&body);
        eprintln!("body: '{}'", body);
        assert_eq!(res.status(), 200);
    }
}
