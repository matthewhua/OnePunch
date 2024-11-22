pub trait Fly {
    fn fly(&self);
}

struct Goose;

#[allow(dead_code)]
struct Duck {
    height: u8,
}

impl Goose {
    pub fn new() -> Self {
        Self
    }
}

impl Duck {
    pub fn new(height: u8) -> Self {
        Self { height }
    }
}


impl Fly for Goose {
    fn fly(&self) {
        println!("Goose is flying");
    }
}

impl Fly for Duck {
    fn fly(&self) {
        println!("Duck is flying {} meters", self.height);
    }
}

fn fly(a: impl Fly) {
    a.fly();
}

// impl Fly 作为返回值，需要有某个确定的类型，这样才能编译通过
// 这段代码无法提供确定的类型，所以出错
// fn select(name: &str) -> impl Fly {
//     match name {
//         "goose" => Goose::new() as Fly,
//         "duck" => Duck::new(3) as Fly,
//     }
// }

fn main() {
    let goose = Goose::new();
    let duck = Duck::new(3);
    fly(goose);
    fly(duck);
}