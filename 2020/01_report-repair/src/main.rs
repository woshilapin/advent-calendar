use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

#[cfg(not(feature = "three"))]
fn fix_expense_report(entries: &[u32]) -> (u32, u32) {
    for (i, &expense1) in entries.iter().enumerate() {
        for &expense2 in &entries[i..] {
            if expense1 + expense2 == 2020 {
                return (expense1, expense2);
            }
        }
    }
    panic!("failed to find 2 expense reports suming up to 2020");
}

#[cfg(feature = "three")]
fn fix_expense_report(entries: &[u32]) -> (u32, u32, u32) {
    for i in 0..entries.len() {
        let expense1 = entries[i];
        for j in 0..entries.len() {
            if j == i {
                continue;
            }
            let expense2 = entries[j];
            for k in 0..entries.len() {
                if k == i || k == j {
                    continue;
                }
                let expense3 = entries[k];
                if expense1 + expense2 + expense3 == 2020 {
                    return (expense1, expense2, expense3);
                }
            }
        }
    }
    panic!("failed to find 3 expense reports suming up to 2020");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1]).expect(format!("expect file '{}' to exist", args[1]).as_str());
    let reader = BufReader::new(file);
    let entries: Vec<u32> = reader
        .lines()
        .map(|line| {
            line.expect("expect line to be parseable as a String")
                .parse()
                .expect("expect string to be parseable as a u32")
        })
        .collect();
    #[cfg(not(feature = "three"))]
    {
        let (expense1, expense2) = fix_expense_report(&entries);
        println!("Total is {}", expense1 * expense2);
    }
    #[cfg(feature = "three")]
    {
        let (expense1, expense2, expense3) = fix_expense_report(&entries);
        println!("Total is {}", expense1 * expense2 * expense3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "three"))]
    fn expense_report() {
        let entries = vec![1721, 979, 366, 299, 675, 1456];
        assert_eq!((1721, 299), fix_expense_report(&entries));
    }

    #[test]
    #[cfg(feature = "three")]
    fn expense_report() {
        let entries = vec![1721, 979, 366, 299, 675, 1456];
        assert_eq!((979, 366, 675), fix_expense_report(&entries));
    }
}
