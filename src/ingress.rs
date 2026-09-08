//! Bounded HTTP/1 transport for the private local service.

use axum::Router;
use hyper::server::conn::http1;
use hyper_util::{
    rt::{TokioIo, TokioTimer},
    service::TowerToHyperService,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
use std::{future::Future, io, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct IngressLimits {
    pub max_connections: usize,
    pub header_timeout: Duration,
    pub connection_lifetime: Duration,
    pub shutdown_grace: Duration,
}

impl Default for IngressLimits {
    fn default() -> Self {
        Self {
            max_connections: 64,
            header_timeout: Duration::from_secs(5),
            connection_lifetime: Duration::from_secs(300),
            shutdown_grace: Duration::from_secs(5),
        }
    }
}

#[derive(Default)]
pub struct IngressStats {
    pub accepted: AtomicUsize,
    pub rejected: AtomicUsize,
    pub active: AtomicUsize,
    pub peak: AtomicUsize,
}

struct Active(Arc<IngressStats>);
impl Drop for Active {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::Relaxed);
    }
}

/// The controller and writer have separate lifetimes. Disconnecting a transport
/// never aborts a run already transferred to controller ownership.
pub async fn serve(
    listener: TcpListener,
    router: Router,
    shutdown: CancellationToken,
    limits: IngressLimits,
    stats: Arc<IngressStats>,
) -> Result<(), String> {
    if !listener
        .local_addr()
        .map_err(|_| "listener_address")?
        .ip()
        .is_loopback()
        || limits.max_connections == 0
        || limits.max_connections > 1024
        || limits.header_timeout.is_zero()
        || limits.connection_lifetime.is_zero()
        || limits.shutdown_grace.is_zero()
    {
        return Err("invalid_private_ingress_limits".into());
    }
    serve_connections(|| listener.accept(), router, shutdown, limits, stats).await
}

async fn serve_connections<A, F>(
    mut accept: A,
    router: Router,
    shutdown: CancellationToken,
    limits: IngressLimits,
    stats: Arc<IngressStats>,
) -> Result<(), String>
where
    A: FnMut() -> F,
    F: Future<Output = io::Result<(TcpStream, SocketAddr)>>,
{
    let permits = Arc::new(Semaphore::new(limits.max_connections));
    let mut connections = JoinSet::new();
    let result = loop {
        tokio::select! {
            biased;
            _=shutdown.cancelled()=>break Ok(()),
            _=connections.join_next(),if !connections.is_empty()=>{},
            accepted=accept()=>{
                let (socket,_)=match accepted {
                    Ok(accepted)=>accepted,
                    Err(_)=>break Err("listener_accept".into()),
                };
                let Ok(permit)=permits.clone().try_acquire_owned() else {
                    stats.rejected.fetch_add(1,Ordering::Relaxed);
                    drop(socket);
                    continue;
                };
                stats.accepted.fetch_add(1,Ordering::Relaxed);
                let active=stats.active.fetch_add(1,Ordering::Relaxed)+1;
                stats.peak.fetch_max(active,Ordering::Relaxed);
                let tracked=Active(stats.clone());
                let router=router.clone();
                let shutdown=shutdown.clone();
                let limits=limits.clone();
                connections.spawn(async move {
                    let _permit=permit;
                    let _tracked=tracked;
                    let mut builder=http1::Builder::new();
                    builder.timer(TokioTimer::new()).header_read_timeout(limits.header_timeout)
                        .max_buf_size(16*1024).max_headers(32).keep_alive(true);
                    let connection=builder.serve_connection(TokioIo::new(socket),TowerToHyperService::new(router));
                    tokio::pin!(connection);
                    tokio::select! {
                        _=&mut connection=>return,
                        _=shutdown.cancelled()=>{},
                        _=tokio::time::sleep(limits.connection_lifetime)=>{},
                    }
                    connection.as_mut().graceful_shutdown();
                    let _=timeout(limits.shutdown_grace,&mut connection).await;
                    // Only cancellable connection I/O is dropped here. The
                    // controller still owns admissions, runners and commits.
                });
            }
        }
    };
    // An accept failure stops new traffic through the same path as requested
    // shutdown. Retain every connection until its graceful cleanup completes.
    shutdown.cancel();
    while connections.join_next().await.is_some() {}
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    #[tokio::test]
    async fn slow_headers_consume_bounded_connections_and_shutdown_joins_them() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let stopped = CancellationToken::new();
        let stats = Arc::new(IngressStats::default());
        let server = tokio::spawn(serve(
            listener,
            Router::new().route("/", axum::routing::get(|| async { "ok" })),
            stopped.clone(),
            IngressLimits {
                max_connections: 2,
                header_timeout: Duration::from_millis(100),
                connection_lifetime: Duration::from_secs(2),
                shutdown_grace: Duration::from_millis(100),
            },
            stats.clone(),
        ));
        let mut first = TcpStream::connect(address).await.unwrap();
        first.write_all(b"GET / HTTP/1.1\r\nHost:").await.unwrap();
        let mut second = TcpStream::connect(address).await.unwrap();
        second.write_all(b"GET / HTTP/1.1\r\nHost:").await.unwrap();
        timeout(Duration::from_secs(1), async {
            while stats.active.load(Ordering::Relaxed) != 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let mut excess = TcpStream::connect(address).await.unwrap();
        let mut buffer = [0; 512];
        let closed = timeout(Duration::from_secs(1), excess.read(&mut buffer))
            .await
            .unwrap();
        assert!(closed.is_err() || closed.unwrap() == 0);
        let finished = timeout(Duration::from_secs(1), first.read(&mut buffer))
            .await
            .unwrap();
        assert!(
            finished.is_err()
                || !String::from_utf8_lossy(&buffer[..finished.unwrap()]).contains("\r\n\r\nok")
        );
        stopped.cancel();
        server.await.unwrap().unwrap();
        assert_eq!(stats.peak.load(Ordering::Relaxed), 2);
        assert_eq!(stats.rejected.load(Ordering::Relaxed), 1);
        assert_eq!(stats.active.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn accept_failure_preserves_inflight_response_and_joins_connections() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let stopped = CancellationToken::new();
        let stats = Arc::new(IngressStats::default());
        let entered = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Notify::new());
        let fail_accept = Arc::new(tokio::sync::Notify::new());
        let handler_entered = entered.clone();
        let handler_release = release.clone();
        let router = Router::new().route(
            "/",
            axum::routing::get(move || {
                let entered = handler_entered.clone();
                let release = handler_release.clone();
                async move {
                    entered.notify_one();
                    release.notified().await;
                    "settled response"
                }
            }),
        );
        let server = tokio::spawn({
            let stopped = stopped.clone();
            let stats = stats.clone();
            let fail_accept = fail_accept.clone();
            async move {
                let attempts = AtomicUsize::new(0);
                serve_connections(
                    || {
                        let listener = &listener;
                        let attempts = &attempts;
                        let fail_accept = &fail_accept;
                        async move {
                            if attempts.fetch_add(1, Ordering::Relaxed) == 0 {
                                listener.accept().await
                            } else {
                                fail_accept.notified().await;
                                Err(io::Error::other("injected accept failure"))
                            }
                        }
                    },
                    router,
                    stopped,
                    IngressLimits {
                        shutdown_grace: Duration::from_secs(1),
                        ..IngressLimits::default()
                    },
                    stats,
                )
                .await
            }
        });
        let mut client = TcpStream::connect(address).await.unwrap();
        client
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        timeout(Duration::from_secs(1), entered.notified())
            .await
            .unwrap();
        fail_accept.notify_one();
        timeout(Duration::from_secs(1), stopped.cancelled())
            .await
            .unwrap();
        assert!(!server.is_finished());
        assert_eq!(stats.active.load(Ordering::Relaxed), 1);
        release.notify_one();
        let mut response = Vec::new();
        timeout(Duration::from_secs(2), client.read_to_end(&mut response))
            .await
            .unwrap()
            .unwrap();
        assert!(String::from_utf8_lossy(&response).ends_with("settled response"));
        assert_eq!(server.await.unwrap(), Err("listener_accept".into()));
        assert_eq!(stats.active.load(Ordering::Relaxed), 0);
        assert_eq!(stats.accepted.load(Ordering::Relaxed), 1);
    }
}
