type RuleName = &'static str;
type RuleConstraint = std::ops::RangeInclusive<usize>;
type RulesInner = std::collections::BTreeMap<RuleName, Vec<RuleConstraint>>;
#[derive(Debug)]
struct Rules {
    inner: RulesInner,
}
impl std::ops::Deref for Rules {
    type Target = RulesInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<I> std::convert::From<I> for Rules
where
    I: Iterator<Item = &'static str>,
{
    fn from(iter: I) -> Self {
        let inner = iter.fold(RulesInner::new(), |mut acc, line| {
            let mut split = line.split(": ");
            let name = split.next().expect("expect a name for the rule");
            let ranges = split
                .next()
                .expect("expect ranges for the rule")
                .split(" or ")
                .map(|range| {
                    let mut split = range.split('-');
                    let min = split
                        .next()
                        .expect("expect a minimum in the range")
                        .parse()
                        .expect("expect minimum bound to be an integer");
                    let max = split
                        .next()
                        .expect("expect a maximum in the range")
                        .parse()
                        .expect("expect maximum bound to be an integer");
                    std::ops::RangeInclusive::new(min, max)
                })
                .collect();
            acc.insert(name, ranges);
            acc
        });
        Self { inner }
    }
}

impl Rules {
    fn is_valid_field(&self, field: &usize) -> bool {
        for ranges in self.inner.values() {
            for range in ranges {
                if range.contains(&field) {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug)]
struct Ticket {
    fields: Vec<usize>,
}

impl<I> std::convert::From<I> for Ticket
where
    I: Iterator<Item = &'static str>,
{
    fn from(fields: I) -> Self {
        let fields = fields
            .map(|field| field.parse().expect("expect field to be an integer"))
            .collect();
        Self { fields }
    }
}

impl Ticket {
    fn is_valid(&self, rules: &Rules) -> bool {
        self.fields.iter().all(|field| rules.is_valid_field(field))
    }
    fn invalid_fields<'ticket>(
        &'ticket self,
        rules: &'ticket Rules,
    ) -> impl Iterator<Item = &'ticket usize> + 'ticket {
        self.fields
            .iter()
            .filter(move |field| !rules.is_valid_field(field))
    }
}

#[derive(Debug)]
struct Notes {
    rules: Rules,
    ticket: Ticket,
    tickets: Vec<Ticket>,
}

impl std::convert::From<&'static str> for Notes {
    fn from(notes: &'static str) -> Self {
        let mut notes = notes.split("\n\n");
        let rules = Rules::from(
            notes
                .next()
                .expect("expect at least rules in the notes")
                .trim()
                .lines(),
        );
        let ticket = Ticket::from(
            notes
                .next()
                .expect("expect at least my ticket information in the notes")
                .trim()
                .lines()
                .skip(1)
                .next()
                .expect("expect at least one line for my ticket information")
                .split(','),
        );
        let tickets = notes
            .next()
            .expect("expect at least other tickets information in the notes")
            .trim()
            .lines()
            .skip(1)
            .map(|line| Ticket::from(line.split(',')))
            .collect();
        Self {
            rules,
            ticket,
            tickets,
        }
    }
}

impl Notes {
    fn invalid_fields<'notes>(&'notes self) -> impl Iterator<Item = usize> + 'notes {
        self.tickets
            .iter()
            .filter(move |ticket| !ticket.is_valid(&self.rules))
            .flat_map(move |ticket| ticket.invalid_fields(&self.rules))
            .copied()
    }
    fn identify_fields(&self) -> Vec<RuleName> {
        let fields_len = self.rules.len();
        let mut fields_order = std::collections::HashMap::new();
        let valid_tickets: Vec<&Ticket> = self
            .tickets
            .iter()
            .filter(|ticket| ticket.is_valid(&self.rules))
            .collect();
        for (&rule_name, rule_ranges) in self.rules.inner.iter() {
            for field_index in 0..fields_len {
                if valid_tickets
                    .iter()
                    .map(|ticket| ticket.fields[field_index])
                    .all(|field| {
                        let contains = rule_ranges.iter().any(|range| range.contains(&field));
                        contains
                    })
                {
                    fields_order
                        .entry(rule_name)
                        .or_insert_with(Vec::new)
                        .push(field_index);
                }
            }
        }
        let mut final_order = Vec::new();
        while fields_order.len() > 0 {
            // Extract rules with only one solution
            for (&rule_name, indexes) in &fields_order {
                if indexes.len() == 1 {
                    final_order.push((indexes[0], rule_name));
                }
            }
            for (index, rule_name) in &final_order {
                fields_order.remove(rule_name);
                for (_, indexes) in &mut fields_order {
                    indexes.retain(|i| i != index);
                }
            }
        }
        final_order.sort_by_key(|&(index, _)| index);
        final_order
            .into_iter()
            .map(|(_, rule_name)| rule_name)
            .collect()
    }
    fn my_departure_fields<'notes>(&'notes self) -> impl Iterator<Item = usize> + 'notes {
        self.identify_fields()
            .into_iter()
            .zip(&self.ticket.fields)
            .filter(|(rule_name, _)| rule_name.starts_with("departure"))
            .map(|(_, field)| field)
            .copied()
    }
}

fn main() {
    let notes = Notes::from(include_str!("../notes.txt"));
    println!(
        "Sum of all invalid fields is {}",
        notes.invalid_fields().sum::<usize>()
    );
    println!(
        "Product of my 'departure' fields is {}",
        notes.my_departure_fields().product::<usize>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_fields() {
        let notes = r#"class: 1-3 or 5-7
row: 6-11 or 33-44
seat: 13-40 or 45-50

your ticket:
7,1,14

nearby tickets:
7,3,47
40,4,50
55,2,20
38,6,12"#;
        let notes = Notes::from(notes);
        assert_eq!(71, notes.invalid_fields().sum::<usize>());
    }

    #[test]
    fn identify_fields() {
        let notes = r#"class: 0-1 or 4-19
departure_row: 0-5 or 8-19
departure_seat: 0-13 or 16-19

your ticket:
11,12,13

nearby tickets:
3,9,18
15,1,5
5,14,9"#;
        let notes = Notes::from(notes);
        let identified_fields = notes.identify_fields();
        assert_eq!("departure_row", identified_fields[0]);
        assert_eq!("class", identified_fields[1]);
        assert_eq!("departure_seat", identified_fields[2]);
        assert_eq!(11 * 13, notes.my_departure_fields().product::<usize>());
    }
}
