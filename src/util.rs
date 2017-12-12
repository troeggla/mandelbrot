use std;

pub fn parse_list<T: std::str::FromStr>(dimensions: String, delimiter: &str) -> (T, T) {
    let mut result: Vec<T> = dimensions.split(delimiter).take(2).map(|s| {
        s.parse::<T>().ok().unwrap()
    }).collect();

    (result.remove(0), result.remove(0))
}
