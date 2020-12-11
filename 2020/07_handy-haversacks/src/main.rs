use std::{
    collections::{HashMap, HashSet},
    env,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Bag {
    tint: String,
    color: String,
}

impl std::fmt::Display for Bag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} bag", self.tint, self.color)
    }
}

#[derive(Debug)]
struct Rule {
    bag: Bag,
    contains: HashMap<Bag, usize>,
}

struct Rules<I>
where
    I: Iterator<Item = String>,
{
    stream: I,
}

type RulesMap = HashMap<Bag, HashMap<Bag, usize>>;
impl<I> Rules<I>
where
    I: Iterator<Item = String>,
{
    fn new(stream: I) -> Self {
        Self { stream }
    }

    fn as_map(self) -> RulesMap {
        self.map(|rule| (rule.bag, rule.contains)).collect()
    }
}

fn contains_bag(rules_map: &RulesMap, bag: &Bag) -> HashSet<Bag> {
    let bag_contains: HashMap<Bag, Vec<Bag>> = rules_map
        .into_iter()
        .map(|(bag, contains)| (bag.clone(), contains.keys().cloned().collect()))
        .collect();
    fn find_wrappers(bag_contains: &HashMap<Bag, Vec<Bag>>, bag: &Bag) -> HashSet<Bag> {
        let mut set = HashSet::new();
        for (b, c) in bag_contains {
            if c.contains(&bag) {
                set.insert(b.clone());
                set.extend(find_wrappers(bag_contains, b));
            }
        }
        set
    }
    find_wrappers(&bag_contains, bag)
}

fn bags_in(rules_map: &RulesMap, bag: &Bag) -> Vec<Bag> {
    let mut bags = Vec::new();
    rules_map
        .get(bag)
        .expect("expect this bag to have a rule")
        .iter()
        .for_each(|(b, quantity)| {
            for _ in 0..*quantity {
                bags.push(b.clone());
                bags.extend(bags_in(rules_map, b));
            }
        });
    bags
}

impl<I> Iterator for Rules<I>
where
    I: Iterator<Item = String>,
{
    type Item = Rule;
    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next().map(|rule| {
            let mut split = rule.split(" bags contain ");
            let mut bag_properties = split
                .next()
                .expect("expect at least a bag definition")
                .split_whitespace();
            let tint = bag_properties.next().expect("expect a tint").to_owned();
            let color = bag_properties.next().expect("expect a color").to_owned();
            let contained_list = split
                .next()
                .expect("expect a list of contained bags")
                .split(", ");
            let mut contains = HashMap::default();
            for contained in contained_list {
                if contained.starts_with("no ") {
                    break;
                }
                let mut bag_properties = contained.split_whitespace();
                let quantity: usize = bag_properties
                    .next()
                    .expect("expect to have a quantity for the contained bag")
                    .parse()
                    .expect("expect the quantity to be parsearble as a usize");
                let tint = bag_properties.next().expect("expect a tint").to_owned();
                let color = bag_properties.next().expect("expect a color").to_owned();
                contains.insert(Bag { color, tint }, quantity);
            }
            Rule {
                bag: Bag { tint, color },
                contains,
            }
        })
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1]).expect(format!("expect file '{}' to exist", args[1]).as_str());
    let reader = BufReader::new(file);
    let rules = Rules::new(
        reader
            .lines()
            .map(|line| line.expect("expect line to be parseable as a String")),
    );
    let bag = Bag {
        tint: "shiny".to_string(),
        color: "gold".to_string(),
    };
    let rules_map = rules.as_map();
    let count = contains_bag(&rules_map, &bag).len();
    println!("There is {} different bags containing a {}", count, bag);
    let count = bags_in(&rules_map, &bag).len();
    println!("There is {} bags in {}", count, bag);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bags() {
        let rules = r#"light red bags contain 1 bright white bag, 2 muted yellow bags.
dark orange bags contain 3 bright white bags, 4 muted yellow bags.
bright white bags contain 1 shiny gold bag.
muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
dark olive bags contain 3 faded blue bags, 4 dotted black bags.
vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
faded blue bags contain no other bags.
dotted black bags contain no other bags."#;
        let rules = Rules::new(rules.split('\n').map(std::borrow::ToOwned::to_owned));
        let bag = Bag {
            tint: "shiny".to_string(),
            color: "gold".to_string(),
        };
        let rules_map = rules.as_map();
        assert_eq!(4, contains_bag(&rules_map, &bag).len());
        assert_eq!(32, bags_in(&rules_map, &bag).len());
    }

    #[test]
    fn nested_bags() {
        let rules = r#"shiny gold bags contain 2 dark red bags.
dark red bags contain 2 dark orange bags.
dark orange bags contain 2 dark yellow bags.
dark yellow bags contain 2 dark green bags.
dark green bags contain 2 dark blue bags.
dark blue bags contain 2 dark violet bags.
dark violet bags contain no other bags."#;
        let rules = Rules::new(rules.split('\n').map(std::borrow::ToOwned::to_owned));
        let bag = Bag {
            tint: "shiny".to_string(),
            color: "gold".to_string(),
        };
        let rules_map = rules.as_map();
        assert_eq!(126, bags_in(&rules_map, &bag).len());
    }
}
