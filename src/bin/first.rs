fn calibration(input: &str) -> u32 {
    input
        .lines()
        .filter_map(|line| {
            let first = line.chars().find_map(|c| c.to_digit(10))?;
            let last = line.chars().rev().find_map(|c| c.to_digit(10))?;
            Some((first, last))
        })
        .map(|(first, last)| first * 10 + last)
        .sum()
}

fn main() {
    let input = std::fs::read_to_string("input/first.txt").expect("input/first.txt");
    println!("Solution A: {}", calibration(&input));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample() {
        let sample = include_str!("../../sample/first.txt");
        assert_eq!(142, calibration(sample))
    }
}
