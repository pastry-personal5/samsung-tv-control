use std::net::SocketAddr;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{lookup_host, TcpStream};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_native_tls::native_tls;
use tokio_native_tls::{TlsConnector, TlsStream};
use tokio_tungstenite::tungstenite::protocol::{Message, WebSocketConfig};
use tokio_tungstenite::tungstenite::Error as WebSocketError;
use tokio_tungstenite::{client_async_with_config, WebSocketStream};
use url::Url;

use crate::application::dispatcher::RequestId;
use crate::application::secret::PairingToken;
use crate::application::target::{TargetError, TvHost};
use crate::application::trust::CertificatePin;
use crate::domain::RemoteAction;

use super::codec::{encode_click, parse_channel_event, ChannelEvent, MAX_EVENT_BYTES};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);
const AUTH_TIMEOUT: Duration = Duration::from_secs(45);
const WRITE_TIMEOUT: Duration = Duration::from_secs(4);
const CLIENT_NAME: &str = "Samsung TV Remote for macOS";
const MAX_METADATA_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct ProbeObservation {
    pub pin: CertificatePin,
    pub name: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
    Target(TargetError),
    Offline,
    Timeout,
    Tls,
    CertificateChanged,
    WebSocket,
    PairingDenied,
    TokenRejected,
    PairingTokenMissing,
    Protocol,
    UncertainWrite,
    UnsupportedAction,
}

pub struct Session {
    socket: WebSocketStream<TlsStream<TcpStream>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionCommand {
    Click { id: RequestId, action: RemoteAction },
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEvent {
    Written(RequestId),
    NotSent(RequestId),
    Uncertain(RequestId),
    Disconnected,
    TokenRejected,
}

pub struct ActiveSession {
    pub commands: mpsc::Sender<SessionCommand>,
    pub events: mpsc::Receiver<SessionEvent>,
    pub task: tokio::task::JoinHandle<()>,
}

/// Reads the certificate without sending a pairing request or credential.
/// The returned fingerprint is an observation requiring physical-TV confirmation.
pub async fn probe_tv(host: &TvHost) -> Result<ProbeObservation, SessionError> {
    let (mut tls, pin) = timeout(CONNECT_TIMEOUT, tls_connect(host))
        .await
        .map_err(|_| SessionError::Timeout)??;
    let metadata = timeout(CONNECT_TIMEOUT, read_metadata(&mut tls, host))
        .await
        .ok()
        .and_then(Result::ok);
    let (name, model) = metadata.map(parse_metadata).unwrap_or((None, None));
    Ok(ProbeObservation { pin, name, model })
}

/// Checks the saved device pin before any token-bearing WebSocket upgrade.
pub async fn connect(
    host: &TvHost,
    trusted_pin: CertificatePin,
    token: Option<&PairingToken>,
) -> Result<Session, SessionError> {
    let (tls, observed_pin) = timeout(CONNECT_TIMEOUT, tls_connect(host))
        .await
        .map_err(|_| SessionError::Timeout)??;
    let socket = upgrade_with_pin(tls, host, observed_pin, trusted_pin, token).await?;
    Ok(Session { socket })
}

async fn upgrade_with_pin<S>(
    stream: S,
    host: &TvHost,
    observed_pin: CertificatePin,
    trusted_pin: CertificatePin,
    token: Option<&PairingToken>,
) -> Result<WebSocketStream<S>, SessionError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    if observed_pin != trusted_pin {
        return Err(SessionError::CertificateChanged);
    }

    let url = remote_url(host, token)?;
    let mut config = WebSocketConfig::default();
    config.read_buffer_size = 4096;
    config.write_buffer_size = 0;
    config.max_write_buffer_size = 64 * 1024;
    config.max_message_size = Some(MAX_EVENT_BYTES);
    config.max_frame_size = Some(MAX_EVENT_BYTES);
    let (socket, _) = timeout(
        CONNECT_TIMEOUT,
        client_async_with_config(url.as_str(), stream, Some(config)),
    )
    .await
    .map_err(|_| SessionError::Timeout)?
    .map_err(|_| SessionError::WebSocket)?;
    Ok(socket)
}

impl Session {
    pub async fn await_authorized(
        &mut self,
        reconnecting: bool,
    ) -> Result<Option<PairingToken>, SessionError> {
        timeout(AUTH_TIMEOUT, self.read_authorization(reconnecting))
            .await
            .map_err(|_| SessionError::Timeout)?
    }

    async fn read_authorization(
        &mut self,
        reconnecting: bool,
    ) -> Result<Option<PairingToken>, SessionError> {
        read_authorization_events(&mut self.socket, reconnecting).await
    }

    pub async fn close(mut self) {
        let _ = timeout(WRITE_TIMEOUT, self.socket.close(None)).await;
    }

    /// Runs the sole socket owner. Its one-slot command channel is a transport
    /// handoff; application admission and ordering stay with Dispatcher.
    pub fn start_actor(self) -> ActiveSession {
        let (command_sender, commands) = mpsc::channel(1);
        let (event_sender, event_receiver) = mpsc::channel(16);
        let task = tokio::spawn(run_actor(self.socket, commands, event_sender));
        ActiveSession {
            commands: command_sender,
            events: event_receiver,
            task,
        }
    }
}

async fn read_authorization_events<S>(
    stream: &mut S,
    reconnecting: bool,
) -> Result<Option<PairingToken>, SessionError>
where
    S: Stream<Item = Result<Message, WebSocketError>> + Unpin,
{
    loop {
        let message = stream
            .next()
            .await
            .ok_or(SessionError::Offline)?
            .map_err(|_| SessionError::Protocol)?;
        match message {
            Message::Text(text) => match parse_channel_event(text.as_bytes())
                .map_err(|_| SessionError::Protocol)?
            {
                ChannelEvent::Connected { token } if reconnecting => return Ok(token),
                ChannelEvent::Connected { token: Some(token) } => return Ok(Some(token)),
                ChannelEvent::Connected { token: None } => {
                    return Err(SessionError::PairingTokenMissing)
                }
                ChannelEvent::Denied if reconnecting => return Err(SessionError::TokenRejected),
                ChannelEvent::Denied => return Err(SessionError::PairingDenied),
                ChannelEvent::Ignored => {}
            },
            Message::Close(_) => return Err(SessionError::Offline),
            _ => {}
        }
    }
}

async fn run_actor(
    socket: WebSocketStream<TlsStream<TcpStream>>,
    mut commands: mpsc::Receiver<SessionCommand>,
    events: mpsc::Sender<SessionEvent>,
) {
    let (mut writer, mut reader) = socket.split();
    let mut heartbeat = tokio::time::interval_at(
        tokio::time::Instant::now() + Duration::from_secs(20),
        Duration::from_secs(20),
    );
    let mut awaiting_pong = false;
    loop {
        tokio::select! {
            command = commands.recv() => match command {
                Some(SessionCommand::Click { id, action }) => {
                    let outcome = write_click(&mut writer, id, action, WRITE_TIMEOUT).await;
                    let failed = matches!(outcome, SessionEvent::Uncertain(_));
                    if events.send(outcome).await.is_err() || failed {
                        break;
                    }
                }
                Some(SessionCommand::Close) | None => break,
            },
            incoming = reader.next() => match incoming {
                Some(Ok(Message::Pong(_))) => awaiting_pong = false,
                Some(Ok(Message::Text(text))) => {
                    if matches!(parse_channel_event(text.as_bytes()), Ok(ChannelEvent::Denied)) {
                        let _ = events.send(SessionEvent::TokenRejected).await;
                        break;
                    }
                }
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                _ => {},
            },
            _ = heartbeat.tick() => {
                if awaiting_pong || !matches!(
                    timeout(WRITE_TIMEOUT, writer.send(Message::Ping(Vec::new().into()))).await,
                    Ok(Ok(()))
                ) {
                    break;
                }
                awaiting_pong = true;
            }
        }
    }
    let _ = events.send(SessionEvent::Disconnected).await;
    let _ = timeout(WRITE_TIMEOUT, writer.close()).await;
}

async fn write_click<S>(
    writer: &mut S,
    id: RequestId,
    action: RemoteAction,
    deadline: Duration,
) -> SessionEvent
where
    S: Sink<Message, Error = WebSocketError> + Unpin,
{
    let Ok(frame) = encode_click(action) else {
        return SessionEvent::NotSent(id);
    };
    match timeout(deadline, writer.send(Message::Text(frame.into()))).await {
        Ok(Ok(())) => SessionEvent::Written(id),
        _ => SessionEvent::Uncertain(id),
    }
}

async fn tls_connect(
    host: &TvHost,
) -> Result<(TlsStream<TcpStream>, CertificatePin), SessionError> {
    let addresses = lookup_host((host.as_str(), 8002))
        .await
        .map_err(|_| SessionError::Offline)?
        .collect::<Vec<SocketAddr>>();
    let ips = addresses.iter().map(SocketAddr::ip).collect::<Vec<_>>();
    host.validate_resolved(&ips).map_err(SessionError::Target)?;
    let target = addresses
        .first()
        .ok_or(SessionError::Target(TargetError::NoResolvedAddress))?;
    let tcp = TcpStream::connect(target)
        .await
        .map_err(|_| SessionError::Offline)?;
    tcp.set_nodelay(true).map_err(|_| SessionError::Offline)?;

    let mut builder = native_tls::TlsConnector::builder();
    // This relaxation exists only for one private endpoint. The certificate
    // pin is checked before any URL containing a pairing token is transmitted.
    builder.danger_accept_invalid_certs(true);
    builder.danger_accept_invalid_hostnames(true);
    let connector = TlsConnector::from(builder.build().map_err(|_| SessionError::Tls)?);
    let tls = connector
        .connect(host.as_str(), tcp)
        .await
        .map_err(|_| SessionError::Tls)?;
    let certificate = tls
        .get_ref()
        .peer_certificate()
        .map_err(|_| SessionError::Tls)?
        .ok_or(SessionError::Tls)?;
    let der = certificate.to_der().map_err(|_| SessionError::Tls)?;
    let pin = CertificatePin::from_der(&der);
    Ok((tls, pin))
}

async fn read_metadata(
    tls: &mut TlsStream<TcpStream>,
    host: &TvHost,
) -> Result<Vec<u8>, SessionError> {
    let request = format!(
        "GET /api/v2/ HTTP/1.1\r\nHost: {}:8002\r\nConnection: close\r\n\r\n",
        host.as_str()
    );
    tls.write_all(request.as_bytes())
        .await
        .map_err(|_| SessionError::Offline)?;
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        let size = tls
            .read(&mut buffer)
            .await
            .map_err(|_| SessionError::Offline)?;
        if size == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..size]);
        if bytes.len() > MAX_METADATA_BYTES {
            return Err(SessionError::Protocol);
        }
        if let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            let header_end = header_end + 4;
            if let Ok(header) = std::str::from_utf8(&bytes[..header_end]) {
                let length = header.lines().find_map(|line| {
                    let (key, value) = line.split_once(':')?;
                    key.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                });
                if length.is_some_and(|length| bytes.len() >= header_end + length) {
                    break;
                }
            }
        }
    }
    Ok(bytes)
}

fn parse_metadata(response: Vec<u8>) -> (Option<String>, Option<String>) {
    let Some(header_end) = response.windows(4).position(|window| window == b"\r\n\r\n") else {
        return (None, None);
    };
    let header = &response[..header_end];
    if !header.starts_with(b"HTTP/1.1 200") && !header.starts_with(b"HTTP/1.0 200") {
        return (None, None);
    }
    let body = &response[header_end + 4..];
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return (None, None);
    };
    let device = value.get("device").unwrap_or(&value);
    let name = device
        .get("name")
        .or_else(|| value.get("name"))
        .and_then(serde_json::Value::as_str)
        .and_then(safe_metadata_text);
    let model = device
        .get("modelName")
        .or_else(|| device.get("model"))
        .and_then(serde_json::Value::as_str)
        .and_then(safe_metadata_text);
    (name, model)
}

fn safe_metadata_text(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 80 || value.chars().any(char::is_control) {
        None
    } else {
        Some(value.to_owned())
    }
}

fn remote_url(host: &TvHost, token: Option<&PairingToken>) -> Result<Url, SessionError> {
    let authority = if host.as_str().contains(':') {
        format!("[{}]", host.as_str())
    } else {
        host.as_str().to_owned()
    };
    let mut url = Url::parse(&format!(
        "wss://{authority}:8002/api/v2/channels/samsung.remote.control"
    ))
    .map_err(|_| SessionError::Target(TargetError::InvalidHost))?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("name", &STANDARD.encode(CLIENT_NAME));
        if let Some(token) = token {
            query.append_pair("token", token.as_str());
        }
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    struct FakeWriter {
        frames: Vec<Message>,
        flush_pending: bool,
    }

    impl Sink<Message> for FakeWriter {
        type Error = WebSocketError;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(mut self: Pin<&mut Self>, frame: Message) -> Result<(), Self::Error> {
            self.frames.push(frame);
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            if self.flush_pending {
                Poll::Pending
            } else {
                Poll::Ready(Ok(()))
            }
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    async fn authorization(
        frames: impl IntoIterator<Item = Message>,
        reconnecting: bool,
    ) -> Result<Option<PairingToken>, SessionError> {
        let mut frames = stream::iter(frames.into_iter().map(Ok));
        read_authorization_events(&mut frames, reconnecting).await
    }

    #[tokio::test]
    async fn fake_events_distinguish_consent_denial_and_token_revocation() {
        let connected =
            Message::Text(r#"{"event":"ms.channel.connect","data":{"token":"test-token"}}"#.into());
        let denied = Message::Text(r#"{"event":"ms.channel.unauthorized"}"#.into());
        assert!(authorization([connected], false).await.unwrap().is_some());
        assert_eq!(
            authorization([denied.clone()], false).await.err(),
            Some(SessionError::PairingDenied)
        );
        assert_eq!(
            authorization([denied], true).await.err(),
            Some(SessionError::TokenRejected)
        );
        assert!(authorization(
            [Message::Text(r#"{"event":"ms.channel.connect"}"#.into())],
            true
        )
        .await
        .unwrap()
        .is_none());
    }

    #[tokio::test]
    async fn fake_events_reject_missing_token_and_malformed_or_oversized_frames() {
        assert_eq!(
            authorization(
                [Message::Text(r#"{"event":"ms.channel.connect"}"#.into())],
                false
            )
            .await
            .err(),
            Some(SessionError::PairingTokenMissing)
        );
        assert_eq!(
            authorization([Message::Text("{".into())], false)
                .await
                .err(),
            Some(SessionError::Protocol)
        );
        assert_eq!(
            authorization(
                [Message::Text("x".repeat(MAX_EVENT_BYTES + 1).into())],
                false
            )
            .await
            .err(),
            Some(SessionError::Protocol)
        );
    }

    #[tokio::test]
    async fn silent_fake_tv_times_out_without_accepting_pairing() {
        let mut frames = stream::pending::<Result<Message, WebSocketError>>();
        assert!(timeout(
            Duration::from_millis(1),
            read_authorization_events(&mut frames, false)
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn fake_writer_distinguishes_flushed_ambiguous_and_unsent_clicks() {
        let mut writer = FakeWriter {
            frames: Vec::new(),
            flush_pending: false,
        };
        let id = RequestId::from_test_value(1);
        assert_eq!(
            write_click(&mut writer, id, RemoteAction::Up, Duration::from_millis(1)).await,
            SessionEvent::Written(id)
        );
        assert_eq!(writer.frames.len(), 1);

        writer.flush_pending = true;
        assert_eq!(
            write_click(
                &mut writer,
                id,
                RemoteAction::Down,
                Duration::from_millis(1),
            )
            .await,
            SessionEvent::Uncertain(id)
        );
        assert_eq!(writer.frames.len(), 2);

        assert_eq!(
            write_click(
                &mut writer,
                id,
                RemoteAction::PowerToggle,
                Duration::from_millis(1),
            )
            .await,
            SessionEvent::NotSent(id)
        );
        assert_eq!(writer.frames.len(), 2);
    }

    #[tokio::test]
    async fn changed_certificate_stops_upgrade_before_token_is_sent() {
        let host = TvHost::parse("tv.local").unwrap();
        let token = PairingToken::from_stored("synthetic-token".to_owned()).unwrap();
        let (stream, mut peer) = tokio::io::duplex(1024);
        let result = upgrade_with_pin(
            stream,
            &host,
            CertificatePin::from_der(b"changed certificate"),
            CertificatePin::from_der(b"trusted certificate"),
            Some(&token),
        )
        .await;

        assert!(matches!(result, Err(SessionError::CertificateChanged)));
        let mut bytes = [0u8; 64];
        assert_eq!(peer.read(&mut bytes).await.unwrap(), 0);
    }

    #[test]
    fn token_url_is_secure_and_escapes_untrusted_token_characters() {
        let host = TvHost::parse("tv.local").unwrap();
        let token = PairingToken::from_stored("abc&def".to_owned()).unwrap();
        let url = remote_url(&host, Some(&token)).unwrap();
        assert_eq!(url.scheme(), "wss");
        assert_eq!(url.port(), Some(8002));
        assert_eq!(url.path(), "/api/v2/channels/samsung.remote.control");
        assert!(url.as_str().contains("token=abc%26def"));
        assert_eq!(
            url.query_pairs().find(|(key, _)| key == "name").unwrap().1,
            STANDARD.encode(CLIENT_NAME)
        );
    }

    #[test]
    fn pin_roundtrip_and_debug_redaction() {
        let value = "0f".repeat(32);
        let pin = CertificatePin::from_hex(&value).unwrap();
        assert_eq!(pin.to_hex(), value);
        assert!(!format!("{pin:?}").contains(&value));
        assert!(CertificatePin::from_hex("invalid").is_none());
    }

    #[test]
    fn metadata_projection_keeps_only_safe_name_and_model() {
        let body = br#"{"device":{"name":"Living Room TV","modelName":"Example Model","duid":"private-id","ip":"10.0.0.1"}}"#;
        let response = [
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n".as_slice(),
            body.as_slice(),
        ]
        .concat();
        assert_eq!(
            parse_metadata(response),
            (
                Some("Living Room TV".to_owned()),
                Some("Example Model".to_owned())
            )
        );
        assert_eq!(
            parse_metadata(b"HTTP/1.1 302 Found\r\n\r\n{}".to_vec()),
            (None, None)
        );
    }
}
