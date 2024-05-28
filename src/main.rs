use std::{
    collections::{BTreeSet, HashSet},
    fmt::Debug,
    time::{Duration, SystemTime},
};

use anyhow::Error;
use rand::random;

#[derive(Debug)]
struct Squared;
#[derive(Debug)]
struct Binary;
#[derive(Debug)]
struct BinarySwitchedOrder;
#[derive(Debug)]
struct Hash;
#[derive(Debug)]
struct HashSwitchedOrder;

struct Product {
    time: Duration,
    result: Vec<usize>,
    method: Box<dyn Intersect>,
}

impl Product {
    fn new(method: Box<dyn Intersect>) -> Self {
        Product {
            time: Duration::ZERO,
            result: Vec::new(),
            method,
        }
    }
}

trait Intersect: Debug {
    fn intersect(&self, a: &[usize], b: &[usize]) -> Vec<usize>;
}

impl Intersect for Squared {
    fn intersect(&self, a: &[usize], b: &[usize]) -> Vec<usize> {
        let mut result = vec![];
        for i in a {
            for j in b {
                if i == j {
                    result.push(*i);
                }
            }
        }
        result
    }
}

impl Intersect for Binary {
    fn intersect(&self, a: &[usize], b: &[usize]) -> Vec<usize> {
        let mut result = vec![];
        let (big, small) = if a.len() > b.len() { (a, b) } else { (b, a) };
        let small = BTreeSet::from_iter(small);
        for i in big {
            if small.contains(i) {
                result.push(*i);
            }
        }
        result
    }
}

impl Intersect for BinarySwitchedOrder {
    fn intersect(&self, a: &[usize], b: &[usize]) -> Vec<usize> {
        let mut result = vec![];
        let (big, small) = if a.len() > b.len() { (b, a) } else { (a, b) };
        let small = BTreeSet::from_iter(small);
        for i in big {
            if small.contains(i) {
                result.push(*i);
            }
        }
        result
    }
}

impl Intersect for Hash {
    fn intersect(&self, a: &[usize], b: &[usize]) -> Vec<usize> {
        let mut result = vec![];
        let (big, small) = if a.len() > b.len() { (a, b) } else { (b, a) };
        let small: HashSet<usize> = small.into_iter().cloned().collect();
        for i in big {
            if small.contains(i) {
                result.push(*i);
            }
        }
        result
    }
}

impl Intersect for HashSwitchedOrder {
    fn intersect(&self, a: &[usize], b: &[usize]) -> Vec<usize> {
        let mut result = vec![];
        let (big, small) = if a.len() > b.len() { (b, a) } else { (a, b) };
        let small: HashSet<usize> = small.into_iter().cloned().collect();
        for i in big {
            if small.contains(i) {
                result.push(*i);
            }
        }
        result
    }
}

fn rand_array() -> Vec<usize> {
    let size: u16 = random();
    let mut values = Vec::with_capacity(size as usize);
    for _ in 0..size {
        values.push(random());
    }
    values
}

fn main() -> Result<(), Error> {
    let mut products: Vec<Product> = vec![];
    products.push(Product::new(Box::new(Squared {})));
    products.push(Product::new(Box::new(Binary {})));
    products.push(Product::new(Box::new(BinarySwitchedOrder {})));
    products.push(Product::new(Box::new(Hash {})));
    products.push(Product::new(Box::new(HashSwitchedOrder {})));
    let start = SystemTime::now();
    let a = rand_array();
    let b = rand_array();
    println!(
        "generated test data took {:?}",
        SystemTime::now().duration_since(start)?
    );
    println!("the arrays have the sizes {} and {}\n", a.len(), b.len());

    for product in products.iter_mut() {
        println!("testing function {:?}", product.method);
        let start = SystemTime::now();
        product.result = product.method.intersect(&a, &b);
        product.time = SystemTime::now().duration_since(start)?;
        println!(
            "finished testing function {:?} it took {:?}\n",
            product.method, product.time
        );
    }

    products.sort_by(|a, b| b.time.cmp(&a.time));
    products.windows(2).for_each(|values| {
        println!(
            "{:?} is {:.2} times faster than {:?}",
            values[1].method,
            values[0].time.as_nanos() as f64 / values[1].time.as_nanos() as f64,
            values[0].method
        )
    });

    let equal = products
        .windows(2)
        .all(|values| values[0].result == values[1].result);
    println!("\nall values are equal: {}", equal);
    Ok(())
}
