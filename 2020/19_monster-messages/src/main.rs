use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Rule {
    Char(char),
    Sequences(HashSet<Vec<usize>>),
}

impl std::str::FromStr for Rule {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rule = if s.chars().count() == 3 && s.starts_with('"') && s.ends_with('"') {
            Rule::Char(
                s.chars()
                    .skip(1)
                    .next()
                    .expect("expect at least 1 char surrounded by quotes so this should not fail"),
            )
        } else {
            Rule::Sequences(
                s.split('|')
                    .map(|sequence| {
                        sequence
                            .split_whitespace()
                            .map(|index| {
                                index.parse().expect("expect a rule index to be an integer")
                            })
                            .collect()
                    })
                    .collect(),
            )
        };
        Ok(rule)
    }
}

#[derive(Debug)]
struct Rules {
    constraints: HashSet<Vec<char>>,
}

macro_rules! hash_set {
    ($($e:expr),*) => {{
        let mut hash_set = HashSet::new();
        $(hash_set.insert($e);)*
        hash_set
    }}
}

fn add_sub_to_main(mut main_sequence: Vec<char>, sub_sequence: Vec<char>) -> Vec<char> {
    main_sequence.extend(sub_sequence);
    main_sequence
}
#[test]
fn test_add_sub_to_main() {
    let main = vec!['a', 'b'];
    let sub = vec!['c'];
    assert_eq!(vec!['a', 'b', 'c'], add_sub_to_main(main, sub));
}

fn add_subs_to_main(
    main_sequence: Vec<char>,
    sub_sequences: HashSet<Vec<char>>,
) -> HashSet<Vec<char>> {
    sub_sequences
        .into_iter()
        .map(|sub_sequence| add_sub_to_main(main_sequence.clone(), sub_sequence))
        .collect()
}
#[test]
fn test_add_subs_to_main() {
    let main = vec!['a', 'b'];
    let subs = hash_set![vec!['c'], vec!['d']];
    let sequences = add_subs_to_main(main, subs);
    assert_eq!(sequences.len(), 2);
    assert!(sequences.contains(&vec!['a', 'b', 'c']));
    assert!(sequences.contains(&vec!['a', 'b', 'd']));
}
fn add_subs_to_mains(
    main_sequences: HashSet<Vec<char>>,
    sub_sequences: HashSet<Vec<char>>,
) -> HashSet<Vec<char>> {
    main_sequences
        .into_iter()
        .flat_map(|main_sequence| add_subs_to_main(main_sequence, sub_sequences.clone()))
        .collect()
}
#[test]
fn test_add_subs_to_mains() {
    let main = hash_set![vec!['a', 'b'], vec!['z']];
    let subs = hash_set![vec!['c'], vec!['d']];
    let sequences = add_subs_to_mains(main, subs);
    assert_eq!(sequences.len(), 4);
    assert!(sequences.contains(&vec!['a', 'b', 'c']));
    assert!(sequences.contains(&vec!['a', 'b', 'd']));
    assert!(sequences.contains(&vec!['z', 'c']));
    assert!(sequences.contains(&vec!['z', 'd']));
}
type RulesMap = std::collections::HashMap<usize, Rule>;
fn expand_sequence(sequence: &Vec<usize>, rules_map: &RulesMap) -> HashSet<Vec<char>> {
    let sequence = sequence
        .iter()
        .map(|rule_index| {
            let rule = rules_map.get(rule_index).expect("expect the rule to exist");
            let chars = get_chars(rules_map, rule);
            chars
        })
        .fold(HashSet::new(), |main_sequences, sub_sequences| {
            if main_sequences.is_empty() {
                sub_sequences
            } else {
                add_subs_to_mains(main_sequences, sub_sequences)
            }
        });
    sequence
}
#[test]
fn test_expand_sequence() {
    macro_rules! hash_map {
        ($(($k:expr, $v:expr)),*) => {{
            let mut hash_map = std::collections::HashMap::new();
            $(hash_map.insert($k, $v);)*
            hash_map
        }}
    }
    let sequence = vec![2, 0, 1, 3];
    let rules_map = hash_map![
        (0, Rule::Sequences(hash_set![vec![1]])),
        (1, Rule::Sequences(hash_set![vec![2, 3], vec![2]])),
        (2, Rule::Char('a')),
        (3, Rule::Char('b'))
    ];
    let sequences = expand_sequence(&sequence, &rules_map);
    assert_eq!(sequences.len(), 4);
    assert!(sequences.contains(&vec!['a', 'a', 'b', 'a', 'b', 'b']));
    assert!(sequences.contains(&vec!['a', 'a', 'a', 'b', 'b']));
    assert!(sequences.contains(&vec!['a', 'a', 'b', 'a', 'b']));
    assert!(sequences.contains(&vec!['a', 'a', 'a', 'b']));
}

fn get_chars(rules_map: &RulesMap, rule: &Rule) -> HashSet<Vec<char>> {
    match rule {
        Rule::Char(c) => hash_set![vec![*c]],
        Rule::Sequences(sequences) => sequences
            .iter()
            .flat_map(|sequence| expand_sequence(sequence, rules_map))
            .collect::<HashSet<Vec<char>>>(),
    }
}

impl<I> std::convert::From<I> for Rules
where
    I: Iterator<Item = &'static str>,
{
    fn from(iter: I) -> Self {
        let mut rules_map: RulesMap = iter
            .map(|rule| {
                let mut split = rule.split(':');
                let index = split
                    .next()
                    .expect("expect at least one element for the index of the rule")
                    .parse()
                    .expect("expect the index of the rule to be an integer");
                let rule = Rule::from(
                    split
                        .next()
                        .expect("expect at least a second element to describe the rule")
                        .trim()
                        .parse()
                        .expect("expect each part of the rule to be an integer"),
                );
                (index, rule)
            })
            .collect();
        #[cfg(feature = "looping")]
        rules_map.insert(8, Rule::Sequences(hash_set![vec![42], vec![42, 8]]));
        #[cfg(feature = "looping")]
        rules_map.insert(
            11,
            Rule::Sequences(hash_set![vec![42, 31], vec![42, 11, 31]]),
        );
        let constraints = get_chars(&rules_map, &rules_map[&0]);
        Self { constraints }
    }
}

impl Rules {
    fn is_valid(&self, message: &Message) -> bool {
        if message.len() != self.constraints.iter().next().map(Vec::len).unwrap_or(0) {
            false
        } else {
            self.constraints
                .iter()
                .any(|sequence| sequence.iter().zip(message.chars()).all(|(&c, m)| c == m))
        }
    }
}

#[derive(Debug)]
struct Message {
    inner: &'static str,
}

impl std::ops::Deref for Message {
    type Target = &'static str;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::convert::From<&'static str> for Message {
    fn from(inner: &'static str) -> Self {
        Self { inner }
    }
}

#[derive(Debug)]
struct Messages<I>
where
    I: Iterator<Item = &'static str>,
{
    iter: I,
}
impl<I> std::convert::From<I> for Messages<I>
where
    I: Iterator<Item = &'static str>,
{
    fn from(iter: I) -> Self {
        Self { iter }
    }
}

impl<I> std::iter::Iterator for Messages<I>
where
    I: Iterator<Item = &'static str>,
{
    type Item = Message;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Message::from)
    }
}

fn main() {
    let mut lines = include_str!("../messages.txt").lines();
    let rules = Rules::from(lines.by_ref().take_while(|line| !line.trim().is_empty()));
    let messages = Messages::from(lines);
    let valid_messages = messages.filter(|message| rules.is_valid(message));
    println!("There is {} valid messages", valid_messages.count());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monster_messages1() {
        let rules = r#"0: 1 2
1: "a"
2: 1 3 | 3 1
3: "b""#;
        let rules = Rules::from(rules.lines());
        assert!(rules.is_valid(&Message::from("aab")));
        assert!(rules.is_valid(&Message::from("aba")));
        assert!(!rules.is_valid(&Message::from("aaa")));
        assert!(!rules.is_valid(&Message::from("abb")));
        assert!(!rules.is_valid(&Message::from("bab")));
        assert!(!rules.is_valid(&Message::from("bba")));
        assert!(!rules.is_valid(&Message::from("baa")));
        assert!(!rules.is_valid(&Message::from("bbb")));
    }

    #[test]
    fn monster_messages2() {
        let rules = r#"0: 4 1 5
1: 2 3 | 3 2
2: 4 4 | 5 5
3: 4 5 | 5 4
4: "a"
5: "b""#;
        let rules = Rules::from(rules.lines());
        assert!(rules.is_valid(&Message::from("aaaabb")));
        assert!(rules.is_valid(&Message::from("aaabab")));
        assert!(rules.is_valid(&Message::from("abbabb")));
        assert!(rules.is_valid(&Message::from("abbbab")));
        assert!(rules.is_valid(&Message::from("aabaab")));
        assert!(rules.is_valid(&Message::from("aabbbb")));
        assert!(rules.is_valid(&Message::from("abaaab")));
        assert!(rules.is_valid(&Message::from("ababbb")));
        assert!(!rules.is_valid(&Message::from("bababa")));
        assert!(!rules.is_valid(&Message::from("aaabbb")));
        assert!(!rules.is_valid(&Message::from("aaaabbb")));
    }
}
