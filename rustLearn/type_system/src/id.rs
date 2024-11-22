fn id<T>(x: T) -> T{x}

fn main() {
    let id1 = id(52);
    let id2 = id("Matthew");
    println!("{}, {}", id1, id2);
}