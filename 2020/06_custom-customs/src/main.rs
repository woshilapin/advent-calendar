use std::{
    collections::BTreeSet,
    env,
    fs::File,
    io::{BufRead, BufReader},
};

type Answers = BTreeSet<char>;
type PersonAnswers = Answers;
type GroupAnswers = Answers;

struct PersonsAnswers<I>
where
    I: Iterator<Item = String>,
{
    stream: I,
}

impl<I> PersonsAnswers<I>
where
    I: Iterator<Item = String>,
{
    fn new(stream: I) -> Self {
        Self { stream }
    }

    fn groups_answers(self) -> GroupsAnswers<Self> {
        GroupsAnswers {
            persons_answers: self,
        }
    }
}

impl<I> Iterator for PersonsAnswers<I>
where
    I: Iterator<Item = String>,
{
    type Item = Option<PersonAnswers>;
    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next().map(|answers| {
            if answers.is_empty() {
                None
            } else {
                Some(answers.chars().collect())
            }
        })
    }
}

struct GroupsAnswers<I>
where
    I: Iterator<Item = Option<PersonAnswers>>,
{
    persons_answers: I,
}

impl<I> GroupsAnswers<I>
where
    I: Iterator<Item = Option<PersonAnswers>>,
{
    fn sum_answers(self) -> usize {
        self.map(|group_answers| group_answers.len()).sum()
    }
}

impl<I> Iterator for GroupsAnswers<I>
where
    I: Iterator<Item = Option<PersonAnswers>>,
{
    type Item = GroupAnswers;
    fn next(&mut self) -> Option<Self::Item> {
        let mut group_answers = None::<GroupAnswers>;
        while let Some(Some(person_answers)) = self.persons_answers.next() {
            if let Some(ga) = group_answers {
                #[cfg(not(feature = "everyone"))]
                {
                    group_answers = Some(ga.union(&person_answers).copied().collect());
                }
                #[cfg(feature = "everyone")]
                {
                    group_answers = Some(ga.intersection(&person_answers).copied().collect());
                }
            } else {
                group_answers = Some(person_answers);
            }
        }
        group_answers
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1]).expect(format!("expect file '{}' to exist", args[1]).as_str());
    let reader = BufReader::new(file);
    let persons_answers = PersonsAnswers::new(
        reader
            .lines()
            .map(|line| line.expect("expect line to be parseable as a String")),
    );
    let groups_answers = persons_answers.groups_answers();
    let count = groups_answers.sum_answers();
    println!("Total groups answers is {}", count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boarding_passes() {
        let boarding_passes = r#"abc

a
b
c

ab
ac

a
a
a
a

b"#;
        let persons_answers = PersonsAnswers::new(
            boarding_passes
                .split('\n')
                .map(std::borrow::ToOwned::to_owned),
        );
        let groups_answers = persons_answers.groups_answers();
        #[cfg(not(feature = "everyone"))]
        {
            assert_eq!(11, groups_answers.sum_answers());
        }
        #[cfg(feature = "everyone")]
        {
            assert_eq!(6, groups_answers.sum_answers());
        }
    }
}
