type BusId = usize;
#[derive(Debug)]
struct ShuttleSearch<I>
where
    I: Iterator<Item = &'static str>,
{
    #[cfg(not(feature = "contest"))]
    arrival_time: usize,
    buses: I,
}

impl<I> std::convert::From<I> for ShuttleSearch<std::str::Split<'static, char>>
where
    I: Iterator<Item = &'static str>,
{
    fn from(mut iter: I) -> Self {
        #[cfg(not(feature = "contest"))]
        let arrival_time = iter
            .next()
            .expect("expect at least one line for the arrival time")
            .parse()
            .expect("expect arrival time to be an integer");
        #[cfg(feature = "contest")]
        let _ = iter.next();
        let buses = iter
            .next()
            .expect("expect at least a list of buses")
            .split::<'static, char>(',');
        Self {
            #[cfg(not(feature = "contest"))]
            arrival_time,
            buses,
        }
    }
}

impl<I> std::iter::Iterator for ShuttleSearch<I>
where
    I: Iterator<Item = &'static str>,
{
    type Item = BusId;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(bus_id) = self.buses.next() {
            #[cfg(not(feature = "contest"))]
            if let Ok(bus_id) = bus_id.parse() {
                return Some(bus_id);
            }
            #[cfg(feature = "contest")]
            return bus_id.parse().ok().or(Some(1));
        }
        None
    }
}

impl<I> ShuttleSearch<I>
where
    I: Iterator<Item = &'static str>,
{
    #[cfg(not(feature = "contest"))]
    fn next_bus(self) -> (usize, usize) {
        let arrival_time = self.arrival_time;
        self.map(|bus_id| {
            // Integer division will get the passage before arrival_time
            let previous_passage = (arrival_time / bus_id) * bus_id;
            let next_passage = previous_passage + bus_id - arrival_time;
            (bus_id, next_passage)
        })
        .min_by_key(|passage| passage.1)
        .expect("expect at least one bus to be the next")
    }
    /// This algorithm is inspired by the Chinese Remainder Theorem
    /// Important: The theorem assumes all pairs of 'bus_id` should be coprime.
    /// We never verifies this assumption which is pretty incorrect but works.
    /// The algorithm could probably be adapted to avoid this coprime
    /// assumption by modifying how the increment is modified.
    #[cfg(feature = "contest")]
    fn golden_timestamp(self) -> usize {
        let mut solution = 0;
        let mut increment = 1;
        for (index, bus_id) in self.enumerate() {
            for partial_solution in (0..).map(|v| solution + v * increment) {
                if (partial_solution + index) % bus_id == 0 {
                    solution = partial_solution;
                    // The increment should not be multiplied by `bus_id` but by
                    // the least common multiple between previous `increment`
                    // and `bus_id`... unless we know all `bus_id` are coprime.
                    increment *= bus_id;
                    break;
                }
            }
        }
        solution
    }
}

fn main() {
    let shuttle_search = ShuttleSearch::from(include_str!("../buses.txt").trim().split('\n'));
    #[cfg(not(feature = "contest"))]
    {
        let (bus_id, minutes) = shuttle_search.next_bus();
        println!(
            "The next bus is {} in {} minutes (bus_id * minutes = {})",
            bus_id,
            minutes,
            bus_id * minutes
        );
    }
    #[cfg(feature = "contest")]
    {
        let timestamp = shuttle_search.golden_timestamp();
        println!("The golden timestamp is {}", timestamp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "contest"))]
    #[test]
    fn shuttle_search() {
        let (bus_id, next_passage) =
            ShuttleSearch::from("939\n7,13,x,x,59,x,31,19".split('\n')).next_bus();
        assert_eq!(59, bus_id);
        assert_eq!(5, next_passage);
    }

    #[cfg(feature = "contest")]
    #[test]
    fn shuttle_search() {
        let timestamp = ShuttleSearch::from("0\n3,4,7".split('\n')).golden_timestamp();
        assert_eq!(75, timestamp);
        let timestamp =
            ShuttleSearch::from("0\n7,13,x,x,59,x,31,19".split('\n')).golden_timestamp();
        assert_eq!(1068781, timestamp);
        let timestamp = ShuttleSearch::from("0\n17,x,13,19".split('\n')).golden_timestamp();
        assert_eq!(3417, timestamp);
        let timestamp = ShuttleSearch::from("0\n67,7,59,61".split('\n')).golden_timestamp();
        assert_eq!(754018, timestamp);
        let timestamp = ShuttleSearch::from("0\n67,x,7,59,61".split('\n')).golden_timestamp();
        assert_eq!(779210, timestamp);
        let timestamp = ShuttleSearch::from("0\n67,7,x,59,61".split('\n')).golden_timestamp();
        assert_eq!(1261476, timestamp);
        let timestamp = ShuttleSearch::from("0\n1789,37,47,1889".split('\n')).golden_timestamp();
        assert_eq!(1202161486, timestamp);
    }
}
