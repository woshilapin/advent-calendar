use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, Result},
};

type OrbitalObjects = HashMap<String, Option<String>>;

fn build_orbital_objects<I>(relations: I) -> OrbitalObjects
where
    I: IntoIterator<Item = String>,
{
    let mut orbital_objects = HashMap::new();
    orbital_objects.insert("COM".to_string(), None);
    for relation in relations {
        let objects: Vec<_> = relation.split(')').collect();
        let center = objects[0].to_string();
        let orbital_object = objects[1].to_string();
        orbital_objects.insert(orbital_object, Some(center));
    }
    orbital_objects
}

fn orbits<'a>(orbital_objects: &'a OrbitalObjects, object: &'a str) -> Vec<&'a String> {
    let mut hops = Vec::new();
    let mut current_object = &orbital_objects[object];
    while let Some(c) = current_object {
        hops.push(c);
        current_object = &orbital_objects[c];
    }
    hops
}

#[cfg(not(feature = "santa"))]
fn count_orbits(orbital_objects: &OrbitalObjects) -> usize {
    let mut counter = 0;
    for orbital_object in orbital_objects.keys() {
        let orbits = orbits(orbital_objects, orbital_object);
        counter += orbits.len();
    }
    counter
}

#[cfg(feature = "santa")]
fn transfers_to_santa(orbital_objects: &OrbitalObjects) -> usize {
    let mut santa_orbits = orbits(orbital_objects, "SAN");
    santa_orbits.reverse();
    let mut my_orbits = orbits(orbital_objects, "YOU");
    my_orbits.reverse();
    let mut index = 0;
    while index < santa_orbits.len() && index < my_orbits.len() {
        if santa_orbits[index] != my_orbits[index] {
            break;
        }
        index += 1;
    }
    (santa_orbits.len() - index) + (my_orbits.len() - index)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);
    #[cfg(not(feature = "santa"))]
    {
        let orbital_objects = build_orbital_objects(reader.lines().map(Result::unwrap));
        let orbits_count = count_orbits(&orbital_objects);
        println!("Count of Orbits: {}", orbits_count);
    }
    #[cfg(feature = "santa")]
    {
        let orbital_objects = build_orbital_objects(reader.lines().map(Result::unwrap));
        let transfers_count = transfers_to_santa(&orbital_objects);
        println!("Count of Transfers to Santa: {}", transfers_count);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "santa"))]
    #[test]
    fn total_number_of_orbits() {
        let relations = vec![
            "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L",
        ]
        .into_iter()
        .map(String::from);
        let orbital_objects = build_orbital_objects(relations);
        assert_eq!(42, count_orbits(&orbital_objects));
    }

    #[cfg(feature = "santa")]
    #[test]
    fn to_santa() {
        let relations = vec![
            "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L", "K)YOU",
            "I)SAN",
        ]
        .into_iter()
        .map(String::from);
        let orbital_objects = build_orbital_objects(relations);
        assert_eq!(4, transfers_to_santa(&orbital_objects));
    }
}
