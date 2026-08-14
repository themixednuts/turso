use std::borrow::Cow;

const FILE_PREFIX: &str = "turso-postgres-schema-";
const FILE_SUFFIX: &str = ".db";
const ENCODED_MARKER: char = '%';
const HEX: &[u8; 16] = b"0123456789abcdef";

pub fn postgres_schema_file_name(schema_name: &str) -> String {
    let use_plain_name = schema_name.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'$')
    });
    let encoded_len = if use_plain_name {
        schema_name.len()
    } else {
        1 + schema_name.len() * 2
    };
    let mut filename = String::with_capacity(FILE_PREFIX.len() + encoded_len + FILE_SUFFIX.len());
    filename.push_str(FILE_PREFIX);
    if use_plain_name {
        filename.push_str(schema_name);
    } else {
        filename.push(ENCODED_MARKER);
        for byte in schema_name.bytes() {
            filename.push(char::from(HEX[usize::from(byte >> 4)]));
            filename.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    filename.push_str(FILE_SUFFIX);
    filename
}

pub fn postgres_schema_name_from_file_name(file_name: &str) -> Option<Cow<'_, str>> {
    let name = file_name
        .strip_prefix(FILE_PREFIX)?
        .strip_suffix(FILE_SUFFIX)?;
    let Some(encoded) = name.strip_prefix(ENCODED_MARKER) else {
        return Some(Cow::Borrowed(name));
    };
    if encoded.len() % 2 != 0 {
        return None;
    }

    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = decode_hex_digit(pair[0])?;
        let low = decode_hex_digit(pair[1])?;
        bytes.push((high << 4) | low);
    }
    String::from_utf8(bytes).ok().map(Cow::Owned)
}

pub fn dropped_postgres_schema_file_name(sql: &str) -> Option<String> {
    let parsed = turso_pg_parser::parse(sql).ok()?;
    let stmt = turso_pg_parser::translator::try_extract_drop_schema(&parsed)?;
    let name = turso_parser::IdentKey::from_unquoted(&stmt.name);
    (name != "public").then(|| postgres_schema_file_name(name.as_str()))
}

fn decode_hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        dropped_postgres_schema_file_name, postgres_schema_file_name,
        postgres_schema_name_from_file_name,
    };

    #[test]
    fn plain_schema_file_names_remain_compatible() {
        assert_eq!(
            postgres_schema_file_name("my_schema"),
            "turso-postgres-schema-my_schema.db"
        );
    }

    #[test]
    fn encoded_schema_file_names_round_trip_without_case_collisions() {
        let upper = postgres_schema_file_name("Ä");
        let lower = postgres_schema_file_name("ä");

        assert!(!upper.eq_ignore_ascii_case(&lower));
        assert_eq!(
            postgres_schema_name_from_file_name(&upper).as_deref(),
            Some("Ä")
        );
        assert_eq!(
            postgres_schema_name_from_file_name(&lower).as_deref(),
            Some("ä")
        );
    }

    #[test]
    fn unsafe_file_name_characters_are_encoded() {
        let filename = postgres_schema_file_name("path/name");

        assert!(!filename.contains('/'));
        assert_eq!(
            postgres_schema_name_from_file_name(&filename).as_deref(),
            Some("path/name")
        );
    }

    #[test]
    fn drop_schema_file_name_uses_the_parsed_identifier() {
        assert_eq!(
            dropped_postgres_schema_file_name("DROP SCHEMA \"a b\" CASCADE"),
            Some(postgres_schema_file_name("a b"))
        );
        assert_eq!(
            dropped_postgres_schema_file_name("DROP SCHEMA public"),
            None
        );
    }
}
