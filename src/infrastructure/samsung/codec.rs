use crate::application::credential_store::PairingToken;
use crate::domain::RemoteAction;
use serde_json::Value;

pub const MAX_EVENT_BYTES: usize = 16 * 1024;
#[derive(Debug)]
pub enum ChannelEvent {
    Connected { token: Option<PairingToken> },
    Denied,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    DeferredAction,
    OversizedEvent,
    MalformedEvent,
    InvalidToken,
}

/// Encodes one Samsung remote Click frame; UI and application code never own key strings.
pub fn encode_click(action: RemoteAction) -> Result<String, CodecError> {
    let key = match action {
        RemoteAction::PowerToggle => return Err(CodecError::DeferredAction),
        RemoteAction::Up => "KEY_UP",
        RemoteAction::Down => "KEY_DOWN",
        RemoteAction::Left => "KEY_LEFT",
        RemoteAction::Right => "KEY_RIGHT",
        RemoteAction::Enter => "KEY_ENTER",
        RemoteAction::Back => "KEY_RETURN",
        RemoteAction::Home => "KEY_HOME",
        RemoteAction::Mute => "KEY_MUTE",
        RemoteAction::VolumeUp => "KEY_VOLUP",
        RemoteAction::VolumeDown => "KEY_VOLDOWN",
    };
    Ok(format!(
        "{{\"method\":\"ms.remote.control\",\"params\":{{\"Cmd\":\"Click\",\"DataOfCmd\":\"{key}\",\"Option\":\"false\",\"TypeOfRemote\":\"SendRemoteKey\"}}}}"
    ))
}

/// Parses only the authorization facts needed to establish a remote session.
/// Unknown bounded events are ignored; malformed or oversized input is never accepted.
pub fn parse_channel_event(frame: &[u8]) -> Result<ChannelEvent, CodecError> {
    if frame.len() > MAX_EVENT_BYTES {
        return Err(CodecError::OversizedEvent);
    }
    let value: Value = serde_json::from_slice(frame).map_err(|_| CodecError::MalformedEvent)?;
    let object = value.as_object().ok_or(CodecError::MalformedEvent)?;
    let event = object
        .get("event")
        .and_then(Value::as_str)
        .ok_or(CodecError::MalformedEvent)?;
    match event {
        "ms.channel.connect" => {
            let token = object
                .get("data")
                .and_then(Value::as_object)
                .and_then(|data| data.get("token"));
            let token = match token {
                None | Some(Value::Null) => None,
                Some(Value::String(token)) => PairingToken::from_stored(token.to_owned())
                    .ok_or(CodecError::InvalidToken)
                    .map(Some)?,
                _ => return Err(CodecError::InvalidToken),
            };
            Ok(ChannelEvent::Connected { token })
        }
        "ms.channel.unauthorized" | "ms.channel.error" => Ok(ChannelEvent::Denied),
        _ => Ok(ChannelEvent::Ignored),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_frames_have_the_exact_samsung_key_map() {
        let expected = [
            "KEY_UP",
            "KEY_DOWN",
            "KEY_LEFT",
            "KEY_RIGHT",
            "KEY_ENTER",
            "KEY_RETURN",
            "KEY_HOME",
            "KEY_MUTE",
            "KEY_VOLUP",
            "KEY_VOLDOWN",
        ];
        for (action, key) in RemoteAction::LIVE_ACTIONS.into_iter().zip(expected) {
            let frame = encode_click(action).unwrap();
            let json: Value = serde_json::from_str(&frame).unwrap();
            assert_eq!(json["method"], "ms.remote.control");
            assert_eq!(json["params"]["Cmd"], "Click");
            assert_eq!(json["params"]["DataOfCmd"], key);
            assert_eq!(json["params"]["Option"], "false");
            assert_eq!(json["params"]["TypeOfRemote"], "SendRemoteKey");
        }
        assert_eq!(
            encode_click(RemoteAction::PowerToggle),
            Err(CodecError::DeferredAction)
        );
    }

    #[test]
    fn event_parser_bounds_and_redacts_pairing_token() {
        let event =
            parse_channel_event(br#"{"event":"ms.channel.connect","data":{"token":"secret123"}}"#)
                .unwrap();
        let ChannelEvent::Connected { token: Some(token) } = event else {
            panic!("expected token")
        };
        assert_eq!(token.as_str(), "secret123");
        assert!(!format!("{token:?}").contains("secret123"));
        assert_eq!(
            parse_channel_event(&vec![b'x'; MAX_EVENT_BYTES + 1]).unwrap_err(),
            CodecError::OversizedEvent
        );
        assert_eq!(
            parse_channel_event(b"broken").unwrap_err(),
            CodecError::MalformedEvent
        );
        assert_eq!(
            parse_channel_event(br#"{"event":"ms.channel.connect","data":{"token":""}}"#)
                .unwrap_err(),
            CodecError::InvalidToken
        );
    }

    #[test]
    fn no_token_connect_is_distinct_from_denial() {
        assert!(matches!(
            parse_channel_event(br#"{"event":"ms.channel.connect","data":{}}"#).unwrap(),
            ChannelEvent::Connected { token: None }
        ));
        assert!(matches!(
            parse_channel_event(br#"{"event":"ms.channel.unauthorized"}"#).unwrap(),
            ChannelEvent::Denied
        ));
    }
}
