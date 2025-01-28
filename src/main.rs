use std::{
    cmp::{max_by_key, min_by_key},
    collections::{BTreeSet, HashSet},
    fmt::Debug,
    iter::repeat,
    time::{Duration, SystemTime},
};

use crossterm::terminal::size;
use prettytable::{row, Table};
use rand::random;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

#[derive(Debug)]
struct Squared;
#[derive(Debug)]
struct SquaredBreak;
#[derive(Debug)]
struct BTree;
#[derive(Debug)]
struct Binary;
#[derive(Debug)]
struct Hash;

struct Product {
    name: String,
    time: Duration,
    result: Vec<usize>,
}

impl Product {
    fn new(name: String, time: Duration, result: Vec<usize>) -> Self {
        Product { name, time, result }
    }
}

trait Intersect: Debug + Send + Sync {
    fn intersect(&self, big: &[usize], small: &[usize]) -> Vec<usize>;
}

impl Intersect for Squared {
    fn intersect(&self, big: &[usize], small: &[usize]) -> Vec<usize> {
        big.par_iter()
            .flat_map_iter(|i| small.iter().zip(repeat(i)))
            .filter(|(i, j)| *i == *j)
            .map(|(i, _)| *i)
            .collect()
    }
}

impl Intersect for SquaredBreak {
    fn intersect(&self, big: &[usize], small: &[usize]) -> Vec<usize> {
        big.par_iter()
            .filter(|i| small.par_iter().find_any(|j| j == i).is_some())
            .copied()
            .collect()
    }
}

impl Intersect for BTree {
    fn intersect(&self, big: &[usize], small: &[usize]) -> Vec<usize> {
        let small = BTreeSet::from_iter(small);
        big.par_iter()
            .filter(|i| small.contains(i))
            .copied()
            .collect()
    }
}

impl Intersect for Binary {
    fn intersect(&self, big: &[usize], small: &[usize]) -> Vec<usize> {
        let mut small = small.to_vec();
        small.sort();
        big.par_iter()
            .filter(|i| small.binary_search(i).is_ok())
            .copied()
            .collect()
    }
}

impl Intersect for Hash {
    fn intersect(&self, big: &[usize], small: &[usize]) -> Vec<usize> {
        let small: HashSet<usize> = small.iter().copied().collect();
        big.par_iter()
            .filter(|i| small.contains(i))
            .copied()
            .collect()
    }
}

fn test_method(method: &dyn Intersect, a: &[usize], b: &[usize], appendage: &str) -> Product {
    let name = format!("{:?}{}", method, appendage);
    let start = SystemTime::now();
    let result = method.intersect(a, b);
    let time = SystemTime::now().duration_since(start).unwrap();
    Product::new(name, time, result)
}

fn print_table(products: &[Product]) {
    let mut table = Table::new();
    table.add_row(row![
        "Name",
        "Time taken",
        "times faster than previous",
        "Absolute time difference",
        "percent of previous time",
        "Compared to"
    ]);
    table.add_row(row![
        products[0].name,
        format!("{:?}", products[0].time),
        "-",
        "-",
        "-",
        "-"
    ]);

    products.windows(2).for_each(|values| {
        table.add_row(row![
            values[1].name,
            format!("{:?}", values[1].time),
            format!(
                "{:.2}x",
                values[0].time.as_nanos() as f64 / values[1].time.as_nanos() as f64
            ),
            format!("{:?}", values[0].time - values[1].time),
            format!(
                "{:.2}%",
                values[1].time.as_nanos() as f64 / values[0].time.as_nanos() as f64 * 100.0
            ),
            values[0].name
        ]);
    });
    let first = &products[0];
    let last = products.last().unwrap();
    table.add_row(row![
        "Total",
        format!(
            "{:?}",
            products
                .iter()
                .map(|x| x.time)
                .fold(Duration::ZERO, |a, b| a + b)
        ),
        format!(
            "{:.2}x",
            first.time.as_nanos() as f64 / last.time.as_nanos() as f64
        ),
        format!("{:?}", first.time - last.time),
        format!(
            "{:.2}%",
            last.time.as_nanos() as f64 / first.time.as_nanos() as f64 * 100.0
        ),
        "-"
    ]);
    table.printstd();
}

fn print_graph(products: &[Product]) {
    let max_name_len = products.iter().map(|p| p.name.len()).max().unwrap();
    let width = (size().unwrap().0 as usize - max_name_len - 2) as f64;
    let min = (products.last().unwrap().time.as_nanos() as f64).ln();
    let base = width / ((products[0].time.as_nanos() as f64).ln() - min);

    println!("\ntimes as a log graph: ");
    products.iter().for_each(|product| {
        println!(
            "{:<x$}: {}",
            product.name,
            "*".repeat((((product.time.as_nanos() as f64).ln() - min) * base).round() as usize),
            x = max_name_len
        )
    });
}

fn main() {
    let methods: [Box<dyn Intersect>; 5] = [
        Box::new(Squared {}),
        Box::new(SquaredBreak {}),
        Box::new(BTree {}),
        Box::new(Binary {}),
        Box::new(Hash {}),
    ];
    let start = SystemTime::now();
    let a: Vec<usize> = (0..random::<u16>())
        .into_par_iter()
        .map(|_| random())
        .collect();
    let b: Vec<usize> = (0..random::<u16>())
        .into_par_iter()
        .map(|_| random())
        .collect();
    println!(
        "generating test data took {:?}",
        SystemTime::now().duration_since(start).unwrap()
    );
    println!("the arrays have the sizes {} and {}\n", a.len(), b.len());

    let big = max_by_key(&a, &b, |x| x.len());
    let small = min_by_key(&a, &b, |x| x.len());

    let mut products: Vec<_> = methods
        .par_iter()
        .flat_map(|method| {
            [
                (method, big, small, ""),
                (method, small, big, " switched order"),
            ]
        })
        .map(|(method, a, b, appendage)| test_method(&**method, a, b, appendage))
        .collect();

    products.sort_by(|a, b| b.time.cmp(&a.time));
    print_table(&products);
    print_graph(&products);

    let equal = products
        .windows(2)
        .all(|values| values[0].result == values[1].result);
    println!("\nall values are equal: {}", equal);
}
