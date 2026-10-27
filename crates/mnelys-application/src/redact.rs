//! Redaction of diagnosis text before it leaves the core (plan §19.3).
//!
//! A diagnosis bundle is written to be handed to another human, so it must not carry a
//! credential, a home directory, an address or an identity. The passes below are string
//! scanning rather than regex: they are pure, ordered, and every span they replace is
//! asserted in the tests.
//!
//! This errs towards over-redacting. A masked word that turned out to be harmless costs a
//! little diagnostic value; an unmasked token costs the user their account.

/// What a masked credential becomes.
const MASK: &str = "<redacted>";
/// A user home root.
const HOME: &str = "<HOME>";
/// An IPv4 address, with its port when one follows.
const ADDRESS: &str = "<IP>";
/// A UUID, which Minecraft prints as account and entity identifiers.
const UUID: &str = "<UUID>";

/// Key names whose following value is a credential.
const SECRET_KEYS: [&str; 19] = [
    "accesstoken",
    "access_token",
    "authtoken",
    "auth_token",
    "refreshtoken",
    "refresh_token",
    "token",
    "password",
    "passwd",
    "secret",
    "client_secret",
    "clientsecret",
    "api_key",
    "apikey",
    "xuid",
    "userproof",
    "authorization",
    "bearer",
    "basic",
];

/// Values that only introduce the credential behind them, so the next token goes too.
const AUTH_SCHEMES: [&str; 2] = ["bearer", "basic"];

/// Redacts one run of text.
///
/// `literals` are values this machine knows are secret without asking — an account name,
/// its UUID, a token string. They are matched case-insensitively, longest first.
#[must_use]
pub fn redact(text: &str, literals: &[&str]) -> String {
    let stepped = mask_secret_keys(text);
    let stepped = mask_server_argument(&stepped);
    let stepped = mask_home_roots(&stepped);
    let stepped = mask_ipv4(&stepped);
    let stepped = mask_uuids(&stepped);
    mask_literals(&stepped, literals)
}

/// Letters, digits and `_`: they mean a match landed inside a longer word.
const fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Every span where `locate` fires, in increasing order and never overlapping.
///
/// `locate` answers "does a match start exactly here, and where does it end?". Walking one
/// byte at a time keeps the scan cheap, and a match jumps the cursor past its own span.
fn find_spans(text: &str, locate: impl Fn(usize) -> Option<usize>) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut index = 0;
    while index < text.len() {
        match locate(index).filter(|end| *end > index) {
            Some(end) => {
                spans.push((index, end));
                index = end;
            }
            None => index += 1,
        }
    }
    spans
}

/// Rebuilds `text` with every span replaced by `placeholder`.
fn mask_spans(text: &str, spans: &[(usize, usize)], placeholder: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    for (start, end) in spans {
        copy_through(text, &mut out, cursor, *start);
        out.push_str(placeholder);
        cursor = *end;
    }
    copy_through(text, &mut out, cursor, text.len());
    out
}

/// Masks the value after every [`SECRET_KEYS`] occurrence.
fn mask_secret_keys(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut index = 0;
    while let Some(key_end) = find_secret_key(bytes, index) {
        let Some(mut value) = value_span(bytes, key_end) else {
            // A key with nothing after it has no value to mask.
            index = key_end;
            continue;
        };
        // `Authorization: Bearer <token>` masks the scheme and the credential behind it.
        if is_auth_scheme(text, value) {
            value = (
                value.0,
                value_span(bytes, value.1).map_or(value.1, |next| next.1),
            );
        }
        spans.push(value);
        index = value.1;
    }
    mask_spans(text, &spans, MASK)
}

/// The span of the value a key introduces, skipping the `:`/`=`/quote/space between.
///
/// A quote before the separator closes the key name; a quote after it opens the value and
/// stays out of the span, so masking leaves the pair of quotes around the placeholder.
/// Anything else that follows the key is masked too: a launch argument passes its token as
/// a bare word, and an unmasked credential costs more than a masked ordinary word.
fn value_span(bytes: &[u8], from: usize) -> Option<(usize, usize)> {
    let mut index = from;
    let mut saw_separator = false;
    while index < bytes.len() {
        match bytes[index] {
            b':' | b'=' => {
                saw_separator = true;
                index += 1;
            }
            b' ' | b'\t' => index += 1,
            b'"' | b'\'' if !saw_separator => index += 1,
            _ => break,
        }
    }
    let quoted = matches!(bytes.get(index), Some(b'"') | Some(b'\''));
    let start = if quoted { index + 1 } else { index };
    let mut end = start;
    while end < bytes.len() {
        let byte = bytes[end];
        if byte.is_ascii_whitespace()
            || matches!(byte, b',' | b'}' | b']' | b';')
            || (quoted && matches!(byte, b'"' | b'\''))
        {
            break;
        }
        end += 1;
    }
    if end == start {
        return None;
    }
    Some((start, end))
}

/// Whether the masked value is an `Authorization` scheme naming the credential after it.
fn is_auth_scheme(text: &str, value: (usize, usize)) -> bool {
    text.get(value.0..value.1)
        .is_some_and(|word| AUTH_SCHEMES.contains(&word.to_ascii_lowercase().as_str()))
}

/// Masks the argument of `--server <address>`: the only place an address is passed in.
fn mask_server_argument(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut index = 0;
    while let Some(found) = find_word(bytes, index, "--server") {
        let after = found + "--server".len();
        match value_span(bytes, after) {
            Some(value) => {
                spans.push(value);
                index = value.1;
            }
            None => index = after,
        }
    }
    mask_spans(text, &spans, MASK)
}

/// Masks `<drive>:\Users\<name>`, `/home/<name>` and `/Users/<name>` roots.
fn mask_home_roots(text: &str) -> String {
    let bytes = text.as_bytes();
    let spans = find_spans(text, |at| home_root_at(bytes, at));
    mask_spans(text, &spans, HOME)
}

/// The index just past a secret key at or after `from`.
fn find_secret_key(bytes: &[u8], from: usize) -> Option<usize> {
    let mut index = from;
    while index < bytes.len() {
        let starts_a_word = index == 0 || !is_word_byte(bytes[index - 1]);
        if starts_a_word {
            for key in SECRET_KEYS {
                let end = index + key.len();
                if end > bytes.len() || !bytes[index..end].eq_ignore_ascii_case(key.as_bytes()) {
                    continue;
                }
                let followed_by_word = bytes
                    .get(end)
                    .is_some_and(|byte| is_word_byte(*byte) || *byte == b'-');
                if !followed_by_word {
                    return Some(end);
                }
            }
        }
        index += 1;
    }
    None
}

/// Where the home root starting exactly at `at` ends, if one starts there.
fn home_root_at(bytes: &[u8], at: usize) -> Option<usize> {
    let after_root = |root_start: usize| -> Option<usize> {
        let separator = *bytes.get(root_start)?;
        if separator != b'\\' && separator != b'/' {
            return None;
        }
        let users_end = root_start + 1 + "users".len();
        if users_end >= bytes.len()
            || !bytes[root_start + 1..users_end].eq_ignore_ascii_case(b"users")
            || !matches!(bytes[users_end], b'\\' | b'/')
        {
            return None;
        }
        path_segment_end(bytes, users_end + 1)
    };
    if bytes.get(at).is_some_and(u8::is_ascii_alphabetic)
        && bytes.get(at + 1) == Some(&b':')
        && matches!(bytes.get(at + 2), Some(&b'\\') | Some(&b'/'))
    {
        return after_root(at + 2);
    }
    if bytes.get(at) == Some(&b'/') {
        for root in ["home", "Users"] {
            let root_end = at + 1 + root.len();
            if bytes.get(root_end) == Some(&b'/')
                && bytes[at + 1..root_end].eq_ignore_ascii_case(root.as_bytes())
            {
                return path_segment_end(bytes, root_end + 1);
            }
        }
    }
    None
}

/// The index just past the path segment that starts at `from`.
fn path_segment_end(bytes: &[u8], from: usize) -> Option<usize> {
    let mut end = from;
    while end < bytes.len() && !matches!(bytes[end], b'\\' | b'/' | b'"' | b'\'' | b' ') {
        end += 1;
    }
    if end == from {
        return None;
    }
    Some(end)
}

/// Masks dotted-quad IPv4 addresses, with their port when one is attached.
fn mask_ipv4(text: &str) -> String {
    let bytes = text.as_bytes();
    let spans = find_spans(text, |at| ipv4_end(bytes, at));
    mask_spans(text, &spans, ADDRESS)
}

/// The index just past an IPv4 address, plus port, starting exactly at `at`.
fn ipv4_end(bytes: &[u8], at: usize) -> Option<usize> {
    let preceded_by_number = at > 0
        && bytes
            .get(at - 1)
            .is_some_and(|byte| byte.is_ascii_digit() || matches!(byte, b'.' | b'_'));
    if !bytes.get(at).is_some_and(u8::is_ascii_digit) || preceded_by_number {
        return None;
    }
    let mut end = at;
    for group in 0..4 {
        let mut digits = 0;
        let mut value = 0u16;
        while digits < 3 && bytes.get(end).is_some_and(u8::is_ascii_digit) {
            value = value * 10 + u16::from(bytes[end] - b'0');
            end += 1;
            digits += 1;
        }
        if digits == 0 || value > 255 {
            return None;
        }
        if group < 3 {
            if bytes.get(end) == Some(&b'.') {
                end += 1;
            } else {
                return None;
            }
        }
    }
    if bytes
        .get(end)
        .is_some_and(|byte| byte.is_ascii_digit() || *byte == b'.')
    {
        return None;
    }
    if bytes.get(end) == Some(&b':') {
        let mut port = end + 1;
        while bytes.get(port).is_some_and(u8::is_ascii_digit) {
            port += 1;
        }
        if port > end + 1 {
            return Some(port);
        }
    }
    Some(end)
}

/// Masks UUIDs.
fn mask_uuids(text: &str) -> String {
    let bytes = text.as_bytes();
    let spans = find_spans(text, |at| uuid_end(bytes, at));
    mask_spans(text, &spans, UUID)
}

/// The index just past an `8-4-4-4-12` hex UUID starting exactly at `at`.
fn uuid_end(bytes: &[u8], at: usize) -> Option<usize> {
    let inside_a_word = at > 0
        && bytes
            .get(at - 1)
            .is_some_and(|byte| byte.is_ascii_hexdigit() || *byte == b'-');
    if inside_a_word {
        return None;
    }
    let mut end = at;
    for group in [8usize, 4, 4, 4, 12] {
        for _ in 0..group {
            if !bytes.get(end).is_some_and(u8::is_ascii_hexdigit) {
                return None;
            }
            end += 1;
        }
        if group != 12 && bytes.get(end) != Some(&b'-') {
            return None;
        }
        if group != 12 {
            end += 1;
        }
    }
    if bytes
        .get(end)
        .is_some_and(|byte| byte.is_ascii_hexdigit() || *byte == b'-')
    {
        return None;
    }
    Some(end)
}

/// Replaces every known literal, case-insensitively, longest first.
fn mask_literals(text: &str, literals: &[&str]) -> String {
    let mut values: Vec<&str> = literals
        .iter()
        .copied()
        .filter(|value| !value.is_empty())
        .collect();
    values.sort_by_key(|value| std::cmp::Reverse(value.len()));
    let mut replaced = text.to_owned();
    for value in values {
        // `to_ascii_lowercase` keeps byte offsets identical to the source, so a
        // multi-byte character cannot shift a span.
        let source = replaced;
        let haystack = source.to_ascii_lowercase();
        let needle = value.to_ascii_lowercase();
        let mut out = String::with_capacity(source.len());
        let mut index = 0;
        while let Some(found) = haystack[index..].find(&needle) {
            out.push_str(source.get(index..index + found).unwrap_or_default());
            out.push_str(MASK);
            index += found + needle.len();
        }
        out.push_str(source.get(index..).unwrap_or_default());
        replaced = out;
    }
    replaced
}

/// The first index of a lowercase `needle` at or after `from`, ending on a word boundary.
fn find_word(bytes: &[u8], from: usize, needle: &str) -> Option<usize> {
    let haystack = bytes
        .iter()
        .map(|byte| byte.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let target = needle.as_bytes();
    let mut index = from;
    while index + target.len() <= haystack.len() {
        if haystack[index..index + target.len()] == *target {
            let ends_cleanly = matches!(
                haystack.get(index + target.len()),
                None | Some(&b' ') | Some(&b'=')
            );
            if ends_cleanly {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

/// Appends `text[from..to]` to `out`, ignoring a span that is not on a character boundary.
fn copy_through(text: &str, out: &mut String, from: usize, to: usize) {
    out.push_str(text.get(from..to).unwrap_or_default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_launch_argument_loses_its_token_but_keeps_the_username() {
        let line = "--username Steve --accessToken abcdef.ghijkl --version 1.21.4";
        assert_eq!(
            redact(line, &[]),
            "--username Steve --accessToken <redacted> --version 1.21.4"
        );
    }

    #[test]
    fn a_json_credential_field_is_masked_inside_its_quotes() {
        let line = r#"{"access_token": "eyJhbGci.abc", "expires_in": 3600}"#;
        assert_eq!(
            redact(line, &[]),
            r#"{"access_token": "<redacted>", "expires_in": 3600}"#
        );
    }

    #[test]
    fn an_authorization_header_masks_the_scheme_and_the_credential() {
        let line = "header Authorization: Bearer eyJ0eXA.abcdef";
        assert_eq!(redact(line, &[]), "header Authorization: <redacted>");
    }

    #[test]
    fn a_password_assignment_is_masked() {
        assert_eq!(
            redact("password=hunter2 sending", &[]),
            "password=<redacted> sending"
        );
    }

    #[test]
    fn a_bare_key_mention_masks_the_word_after_it_too() {
        // Over-redacting on purpose: `--accessToken abc` has no separator either, and
        // missing a credential costs more than masking an ordinary word.
        assert_eq!(
            redact("the token was refreshed", &[]),
            "the token <redacted> refreshed"
        );
    }

    #[test]
    fn a_quoted_credential_keeps_its_quote_pair() {
        assert_eq!(
            redact(r#"token: "eyJhbGci.abc" expired"#, &[]),
            "token: \"<redacted>\" expired"
        );
    }

    #[test]
    fn words_that_contain_a_key_name_are_not_touched() {
        assert_eq!(
            redact("Tokenizer loaded 3 files", &[]),
            "Tokenizer loaded 3 files"
        );
        assert_eq!(redact("basicgraphics ok", &[]), "basicgraphics ok");
    }

    #[test]
    fn a_direct_join_address_is_masked() {
        assert_eq!(
            redact("--server mc.example.test:25565 --gameDir x", &[]),
            "--server <redacted> --gameDir x"
        );
    }

    #[test]
    fn windows_and_posix_home_roots_become_a_placeholder() {
        assert_eq!(
            redact(
                "open C:\\Users\\Herta\\AppData\\Roaming\\.minecraft\\logs\\latest.log",
                &[]
            ),
            "open <HOME>\\AppData\\Roaming\\.minecraft\\logs\\latest.log"
        );
        assert_eq!(
            redact("at /home/steve/.minecraft/versions", &[]),
            "at <HOME>/.minecraft/versions"
        );
        assert_eq!(
            redact("at /Users/steve/.minecraft", &[]),
            "at <HOME>/.minecraft"
        );
    }

    #[test]
    fn dotted_quad_addresses_and_ports_are_masked() {
        assert_eq!(
            redact("connect to 192.168.1.20:25565 now", &[]),
            "connect to <IP> now"
        );
        assert_eq!(
            redact("loopback 127.0.0.1 refused", &[]),
            "loopback <IP> refused"
        );
        assert_eq!(
            redact("bound on (10.0.0.5) locally", &[]),
            "bound on (<IP>) locally"
        );
    }

    #[test]
    fn version_numbers_are_not_mistaken_for_addresses() {
        assert_eq!(
            redact("Minecraft 1.21.4 needs 61.0", &[]),
            "Minecraft 1.21.4 needs 61.0"
        );
        assert_eq!(
            redact("java version 8.0.392_8", &[]),
            "java version 8.0.392_8"
        );
    }

    #[test]
    fn a_uuid_becomes_a_placeholder() {
        assert_eq!(
            redact("--uuid d4ee57e8-8b4c-4f30-a1f5-9b2e5c1a7d3b --name x", &[]),
            "--uuid <UUID> --name x"
        );
    }

    #[test]
    fn known_literals_are_masked_whatever_the_case() {
        let line = "player Steve joined for STEVE and Steve again";
        assert_eq!(
            redact(line, &["Steve"]),
            "player <redacted> joined for <redacted> and <redacted> again"
        );
    }

    #[test]
    fn redaction_is_idempotent() {
        let once = redact("token: abc123 at /home/steve/x 192.168.0.2", &[]);
        assert_eq!(redact(&once, &[]), once);
    }

    #[test]
    fn a_multibyte_neighbour_does_not_shift_a_span() {
        let line = "玩家 Steve 进入 · token: abc123";
        let masked = redact(line, &["Steve"]);
        assert_eq!(masked, "玩家 <redacted> 进入 · token: <redacted>");
    }
}
