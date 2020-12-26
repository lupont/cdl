use std::fmt::Display;
use std::io::{self, Write};

pub fn print_indexed_list<T, D, F>(headers: &[&str], data: &[T], f: F)
where
    D: Display,
    F: Fn(&T) -> D,
{
    print!("  INDEX  ");
    for header in headers {
        print!("{}  ", header);
    }
    println!();

    for (i, item) in data.iter().enumerate() {
        let index = i + 1;
        println!(
            "> {index}{space}     {item}",
            index = index,
            space = if index < 10 { " " } else { "" },
            item = f(item),
        );
    }
}

pub fn print_indexed_list2<T, D1, D2, F, G>(headers: &[&str], data: &[T], f: F, g: G)
where
    D1: Display,
    D2: Display,
    F: Fn(&T) -> D1,
    G: Fn(&T) -> D2,
{
    let mut max_len = 0usize;
    for val in data {
        let curr = format!("{}", f(val)).len();
        if curr > max_len {
            max_len = curr;
        }
    }

    print!("  INDEX  ");
    for header in headers {
        print!("{}  ", &header);
        print!(
            "{}",
            " ".repeat(if header.len() <= max_len {
                max_len - header.len()
            } else {
                0
            })
        );
    }
    print!("\n");

    for i in 0..data.len() {
        let index = i + 1;
        let c1 = f(&data[i]);
        let c2 = g(&data[i]);

        println!(
            "> {index}{space1}     {i}{space2}  {j}",
            index = index,
            space1 = if index < 10 { " " } else { "" },
            i = c1,
            space2 = " ".repeat(
                max_len - format!("{}", c1).len()
                    + if data.len() == 1 && format!("{}", c1).len() < 4 {
                        1
                    } else {
                        0
                    }
            ),
            j = c2,
        );
    }
}

pub fn parse_input(input: &str) -> Option<Vec<usize>> {
    let foo = input
        .trim()
        .split(' ')
        .filter_map(|s| {
            if s.contains("-") {
                let mut parts = s.split("-");
                let start = parts.next()?.parse::<usize>().ok()?;
                let end = parts.next()?.parse::<usize>().ok()?;

                Some((start..=end).collect::<Vec<_>>())
            } else {
                Some(vec![s.parse::<usize>().ok()?])
            }
        })
        .flatten()
        .fold(Vec::new(), |mut a, c| {
            if !a.contains(&c) {
                a.push(c);
            }
            a
        });

    match foo.len() {
        0 => None,
        _ => Some(foo),
    }
}

pub fn read_input() -> io::Result<String> {
    print!("==> ");
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_parse() {
        assert_eq!(parse_input("1"), Some(vec![1]));
        assert_eq!(parse_input("1 2 3"), Some(vec![1, 2, 3]));
        assert_eq!(parse_input("1 1 2 1 3 4 10"), Some(vec![1, 2, 3, 4, 10]));
        assert_eq!(parse_input("1-9"), Some((1..=9).collect()));
        assert_eq!(parse_input("1-3 5 7"), Some(vec![1, 2, 3, 5, 7]));
        assert_eq!(parse_input("1 3 5-6 7"), Some(vec![1, 3, 5, 6, 7]));
        assert_eq!(parse_input("1-3 1 2 3"), Some((1..=3).collect()));
    }
}
