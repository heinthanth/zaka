use std::collections::HashSet;

use zakac_diagnostics::Code;
use zakac_diagnostics::Level;

#[test]
fn ids_are_unique_and_well_formed()
{
    let mut seen = HashSet::new();
    for code in Code::ALL
    {
        let id = code.id();
        assert!(seen.insert(id), "duplicate id {id}");
        assert_eq!(id.len(), 6, "{id}");
        let is_well_formed =
            id.starts_with("ZK") && id[2..].chars().all(|c| c.is_ascii_digit());
        assert!(is_well_formed, "{id}");
    }
}

#[test]
fn ids_are_in_order()
{
    let ids: Vec<&str> = Code::ALL.iter().map(|c| c.id()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted);
}

#[test]
fn parse_round_trips()
{
    for code in Code::ALL
    {
        assert_eq!(Code::parse(code.id()), Some(*code));
    }
    assert_eq!(Code::parse("ZK9999"), None);
    assert_eq!(Code::parse("ZK0418"), None);
    assert_eq!(Code::parse("invalid-character"), None);
}

#[test]
fn lexer_codes_are_errors()
{
    assert_eq!(Code::InvalidCharacter.level(), Level::Error);
    assert_eq!(Code::InvalidCharacter.id(), "ZK0101");
    assert_eq!(Code::BlankRawIdentifier.id(), "ZK0109");
    assert_eq!(Code::InvalidCharacter.to_string(), "ZK0101");
}

#[test]
fn serde_uses_the_id()
{
    let text = serde_saphyr::to_string(&Code::MixedDigitScripts).unwrap();
    assert_eq!(text.trim(), "ZK0103");
    let back: Code = serde_saphyr::from_str("ZK0103").unwrap();
    assert_eq!(back, Code::MixedDigitScripts);
    let bad: Result<Code, _> = serde_saphyr::from_str("ZK9999");
    assert!(bad.is_err());
}
