
struct Cat;

struct Dog;

trait Animal {
    fn name(&self) -> &'static str;
}

impl Animal for Cat {
    fn name(&self) -> &'static str {
        "Cat"
    }
}


impl Animal for Dog {
    fn name(&self) -> &'static str {
        "Dog"
    }
}

fn name(animal: impl Animal) -> &'static str {
    animal.name()
}

fn main() {
    let cat = Cat;
    let dog = Dog;

    println!("Animal name: {}", cat.name());  // 输出: Animal name: Cat
    println!("Animal name: {}", dog.name());  // 输出: Animal name: Dog
    println!("cat: {}", name(cat));
}