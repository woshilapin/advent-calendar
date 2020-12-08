use std::{
    collections::{hash_map::Entry, HashMap},
    env,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    num::ParseIntError,
    str::FromStr,
};

use thiserror::Error;

#[derive(Debug, Error)]
enum MyError {
    #[error("Cannot convert into a quantity")]
    ParseQuantity(#[from] ParseIntError),
    #[error("Cannot convert {0} into a Chemical")]
    ParseChemical(String),
    #[error("Not enough input chemicals for a reaction")]
    NotEnoughInputChemical(String),
    #[error("Cannot convert {0} into a Reaction")]
    ParseReaction(String),
    #[error("Cannot open the file")]
    IOError(#[from] io::Error),
}

#[derive(Debug, Clone)]
struct Chemical {
    name: String,
    quantity: usize,
}

impl Display for Chemical {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} {}", self.quantity, self.name)
    }
}

impl FromStr for Chemical {
    type Err = MyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits: Vec<_> = s.split(" ").collect();
        if splits.len() != 2 {
            Err(MyError::ParseChemical(s.to_string()))
        } else {
            let chemical = Chemical {
                name: splits[1].to_string(),
                quantity: splits[0].parse()?,
            };
            Ok(chemical)
        }
    }
}

#[derive(Debug, Clone)]
struct Reaction {
    inputs: Vec<Chemical>,
    output: Chemical,
}

impl FromStr for Reaction {
    type Err = MyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits: Vec<_> = s.split("=>").collect();
        if splits.len() != 2 {
            Err(MyError::ParseReaction(s.to_string()))
        } else {
            let inputs: Vec<Chemical> = splits[0]
                .split(",")
                .map(str::trim)
                .map(str::parse)
                .collect::<Result<_, _>>()?;
            if inputs.len() < 1 {
                return Err(MyError::NotEnoughInputChemical(s.to_string()));
            }
            let output: Chemical = splits[1].trim().parse()?;
            let reaction = Reaction { inputs, output };
            Ok(reaction)
        }
    }
}

type Waste = HashMap<String, usize>;
struct Reactions {
    reactions: HashMap<String, Reaction>,
    waste: Waste,
}

impl From<Vec<Reaction>> for Reactions {
    fn from(mut reactions: Vec<Reaction>) -> Self {
        let reactions: HashMap<String, Reaction> = reactions
            .drain(..)
            .map(|reaction| (reaction.output.name.clone(), reaction))
            .collect();
        let waste = HashMap::new();
        Reactions { reactions, waste }
    }
}

impl Reactions {
    fn produce_from(&mut self, into: &Chemical, from: &str) -> Chemical {
        let mut quantity = 0;
        let reaction = self.reactions.get(&into.name).unwrap().clone();
        let count = if into.quantity % reaction.output.quantity == 0 {
            into.quantity / reaction.output.quantity
        } else {
            1 + into.quantity / reaction.output.quantity
        };
        for input in reaction.inputs {
            let mut input_needed = count * input.quantity;
            if let Entry::Occupied(mut entry) = self.waste.entry(input.name.clone()) {
                input_needed -= if *entry.get_mut() <= input_needed {
                    let (_, value) = entry.remove_entry();
                    value
                } else {
                    *entry.get_mut() -= input_needed;
                    input_needed
                };
            }
            if input.name == from {
                quantity += input_needed;
            } else {
                let chemical = Chemical {
                    name: input.name.clone(),
                    quantity: input_needed,
                };
                let from = self.produce_from(&chemical, from);
                quantity += from.quantity;
            }
        }
        if count * reaction.output.quantity > into.quantity {
            *self.waste.entry(into.name.clone()).or_insert(0) +=
                count * reaction.output.quantity - into.quantity;
        }
        Chemical {
            name: from.to_string(),
            quantity,
        }
    }

    fn produce_with(&mut self, into: &str, from: &Chemical) -> Chemical {
        let mut quantity = 1;
        let mut has_capped = false;
        let mut increment = 1;
        while increment != 0 {
            let chemical = Chemical {
                name: into.to_string(),
                quantity,
            };
            self.waste.clear();
            let p = self.produce_from(&chemical, &from.name);
            if !has_capped {
                if p.quantity <= from.quantity {
                    increment *= 2;
                    quantity = increment;
                } else {
                    has_capped = true;
                }
            } else {
                increment /= 2;
                if p.quantity <= from.quantity {
                    quantity += increment;
                } else {
                    quantity -= increment;
                }
            }
        }
        let chemical = Chemical {
            name: into.to_string(),
            quantity,
        };
        self.waste.clear();
        let p = self.produce_from(&chemical, &from.name);
        if p.quantity <= from.quantity {
            Chemical {
                name: into.to_string(),
                quantity,
            }
        } else {
            Chemical {
                name: into.to_string(),
                quantity: quantity - 1,
            }
        }
    }
}

fn main() -> Result<(), MyError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);
    let mut reactions = reader
        .lines()
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .map(|s| s.trim().parse())
        .collect::<Result<Vec<Reaction>, MyError>>()
        .map(Reactions::from)?;
    let into = Chemical {
        name: "FUEL".to_string(),
        quantity: 1,
    };
    let from = reactions.produce_from(&into, "ORE");
    println!("To produce {}, you need {}", into, from);
    reactions.waste.clear();
    let from = Chemical {
        name: "ORE".to_string(),
        quantity: 1000000000000,
    };
    let into = reactions.produce_with("FUEL", &from);
    println!("With {}, you can produce {}", from, into);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod chemicals {
        use super::*;

        #[test]
        fn build_chemical() {
            let chemical = Chemical::from_str("7 ORE").unwrap();
            assert_eq!("ORE", chemical.name);
            assert_eq!(7, chemical.quantity);
        }

        #[test]
        #[should_panic]
        fn incorrect_chemical() {
            Chemical::from_str("FOOBAR").unwrap();
        }

        #[test]
        #[should_panic]
        fn incorrect_quantity() {
            Chemical::from_str("NOT_A_QUANTITY ORE").unwrap();
        }
    }

    mod reactions {
        use super::*;

        #[test]
        fn build_reaction() {
            let reaction = Reaction::from_str("1 ORE => 1 FUEL").unwrap();
            assert_eq!(1, reaction.inputs.len());
        }

        #[test]
        fn build_reaction_multi_chemicals() {
            let reaction = Reaction::from_str("1 ORE, 1 MUSHROOM => 1 FUEL").unwrap();
            assert_eq!(2, reaction.inputs.len());
        }

        #[test]
        #[should_panic]
        fn incorrect_reaction() {
            Reaction::from_str("1 ORE, 1 MUSHROOM > 1 FUEL").unwrap();
        }
    }

    mod produce_from {
        use super::*;

        #[test]
        fn test1() {
            let mut reactions = vec![
                "10 ORE => 10 A",
                "1 ORE => 1 B",
                "7 A, 1 B => 1 C",
                "7 A, 1 C => 1 D",
                "7 A, 1 D => 1 E",
                "7 A, 1 E => 1 FUEL",
            ]
            .into_iter()
            .map(str::parse)
            .collect::<Result<Vec<Reaction>, MyError>>()
            .map(|reaction| Reactions::from(reaction))
            .unwrap();
            let into = Chemical {
                name: "FUEL".to_string(),
                quantity: 1,
            };
            let from = reactions.produce_from(&into, "ORE");
            assert_eq!("ORE", from.name);
            assert_eq!(31, from.quantity);
        }

        #[test]
        fn test2() {
            let mut reactions = vec![
                "9 ORE => 2 A",
                "8 ORE => 3 B",
                "7 ORE => 5 C",
                "3 A, 4 B => 1 AB",
                "5 B, 7 C => 1 BC",
                "4 C, 1 A => 1 CA",
                "2 AB, 3 BC, 4 CA => 1 FUEL",
            ]
            .into_iter()
            .map(str::parse)
            .collect::<Result<Vec<Reaction>, MyError>>()
            .map(|reaction| Reactions::from(reaction))
            .unwrap();
            let into = Chemical {
                name: "FUEL".to_string(),
                quantity: 1,
            };
            let from = reactions.produce_from(&into, "ORE");
            assert_eq!("ORE", from.name);
            assert_eq!(165, from.quantity);
        }

        #[test]
        fn test3() {
            let mut reactions = vec![
                "157 ORE => 5 NZVS",
                "165 ORE => 6 DCFZ",
                "44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL",
                "12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ",
                "179 ORE => 7 PSHF",
                "177 ORE => 5 HKGWZ",
                "7 DCFZ, 7 PSHF => 2 XJWVT",
                "165 ORE => 2 GPVTF",
                "3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT",
            ]
            .into_iter()
            .map(str::parse)
            .collect::<Result<Vec<Reaction>, MyError>>()
            .map(|reaction| Reactions::from(reaction))
            .unwrap();
            let into = Chemical {
                name: "FUEL".to_string(),
                quantity: 1,
            };
            let from = reactions.produce_from(&into, "ORE");
            assert_eq!("ORE", from.name);
            assert_eq!(13312, from.quantity);
        }

        #[test]
        fn test4() {
            let mut reactions = vec![
                "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG",
                "17 NVRVD, 3 JNWZP => 8 VPVL",
                "53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL",
                "22 VJHF, 37 MNCFX => 5 FWMGM",
                "139 ORE => 4 NVRVD",
                "144 ORE => 7 JNWZP",
                "5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC",
                "5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV",
                "145 ORE => 6 MNCFX",
                "1 NVRVD => 8 CXFTF",
                "1 VJHF, 6 MNCFX => 4 RFSQX",
                "176 ORE => 6 VJHF",
            ]
            .into_iter()
            .map(str::parse)
            .collect::<Result<Vec<Reaction>, MyError>>()
            .map(|reaction| Reactions::from(reaction))
            .unwrap();
            let into = Chemical {
                name: "FUEL".to_string(),
                quantity: 1,
            };
            let from = reactions.produce_from(&into, "ORE");
            assert_eq!("ORE", from.name);
            assert_eq!(180697, from.quantity);
        }

        #[test]
        fn test5() {
            let mut reactions = vec![
                "171 ORE => 8 CNZTR",
                "7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL",
                "114 ORE => 4 BHXH",
                "14 VRPVC => 6 BMBT",
                "6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL",
                "6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT",
                "15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW",
                "13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW",
                "5 BMBT => 4 WPTQ",
                "189 ORE => 9 KTJDG",
                "1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP",
                "12 VRPVC, 27 CNZTR => 2 XDBXC",
                "15 KTJDG, 12 BHXH => 5 XCVML",
                "3 BHXH, 2 VRPVC => 7 MZWV",
                "121 ORE => 7 VRPVC",
                "7 XCVML => 6 RJRHP",
                "5 BHXH, 4 VRPVC => 5 LTCX",
            ]
            .into_iter()
            .map(str::parse)
            .collect::<Result<Vec<Reaction>, MyError>>()
            .map(|reaction| Reactions::from(reaction))
            .unwrap();
            let into = Chemical {
                name: "FUEL".to_string(),
                quantity: 1,
            };
            let from = reactions.produce_from(&into, "ORE");
            assert_eq!("ORE", from.name);
            assert_eq!(2210736, from.quantity);
        }
    }

    mod produce_with {
        use super::*;

        #[test]
        fn test1() {
            let mut reactions = vec![
                "157 ORE => 5 NZVS",
                "165 ORE => 6 DCFZ",
                "44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL",
                "12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ",
                "179 ORE => 7 PSHF",
                "177 ORE => 5 HKGWZ",
                "7 DCFZ, 7 PSHF => 2 XJWVT",
                "165 ORE => 2 GPVTF",
                "3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT",
            ]
            .into_iter()
            .map(str::parse)
            .collect::<Result<Vec<Reaction>, MyError>>()
            .map(|reaction| Reactions::from(reaction))
            .unwrap();
            let from = Chemical {
                name: "ORE".to_string(),
                quantity: 1000000000000,
            };
            let into = reactions.produce_with("FUEL", &from);
            assert_eq!("FUEL", into.name);
            assert_eq!(82892753, into.quantity);
        }

        #[test]
        fn test2() {
            let mut reactions = vec![
                "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG",
                "17 NVRVD, 3 JNWZP => 8 VPVL",
                "53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL",
                "22 VJHF, 37 MNCFX => 5 FWMGM",
                "139 ORE => 4 NVRVD",
                "144 ORE => 7 JNWZP",
                "5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC",
                "5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV",
                "145 ORE => 6 MNCFX",
                "1 NVRVD => 8 CXFTF",
                "1 VJHF, 6 MNCFX => 4 RFSQX",
                "176 ORE => 6 VJHF",
            ]
            .into_iter()
            .map(str::parse)
            .collect::<Result<Vec<Reaction>, MyError>>()
            .map(|reaction| Reactions::from(reaction))
            .unwrap();
            let from = Chemical {
                name: "ORE".to_string(),
                quantity: 1000000000000,
            };
            let into = reactions.produce_with("FUEL", &from);
            assert_eq!("FUEL", into.name);
            assert_eq!(5586022, into.quantity);
        }

        #[test]
        fn test3() {
            let mut reactions = vec![
                "171 ORE => 8 CNZTR",
                "7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL",
                "114 ORE => 4 BHXH",
                "14 VRPVC => 6 BMBT",
                "6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL",
                "6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT",
                "15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW",
                "13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW",
                "5 BMBT => 4 WPTQ",
                "189 ORE => 9 KTJDG",
                "1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP",
                "12 VRPVC, 27 CNZTR => 2 XDBXC",
                "15 KTJDG, 12 BHXH => 5 XCVML",
                "3 BHXH, 2 VRPVC => 7 MZWV",
                "121 ORE => 7 VRPVC",
                "7 XCVML => 6 RJRHP",
                "5 BHXH, 4 VRPVC => 5 LTCX",
            ]
            .into_iter()
            .map(str::parse)
            .collect::<Result<Vec<Reaction>, MyError>>()
            .map(|reaction| Reactions::from(reaction))
            .unwrap();
            let from = Chemical {
                name: "ORE".to_string(),
                quantity: 1000000000000,
            };
            let into = reactions.produce_with("FUEL", &from);
            assert_eq!("FUEL", into.name);
            assert_eq!(460664, into.quantity);
        }
    }
}
