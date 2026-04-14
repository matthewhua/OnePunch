// 泛型函数 - 适用于任何类型
fn id<T>(x: T) -> T {
    x
}

// 泛型结构体
struct Container<T> {
    value: T,
}

impl<T> Container<T> {
    // 泛型关联类型
    fn new(value: T) -> Self {
        Container { value }
    }
    
    fn get(&self) -> &T {
        &self.value
    }
    
    fn set(&mut self, value: T) {
        self.value = value;
    }
}

// trait bounds 约束
fn add<T: std::ops::Add<Output = T> + Copy>(a: T, b: T) -> T {
    a + b
}

// where 语法糖
fn multiply<T>(a: T, b: T) -> T 
where T: std::ops::Mul<Output = T> + Copy
{
    a * b
}

// 泛型枚举
enum Result<T, E> {
    Ok(T),
    Err(E),
}

fn main() {
    // 泛型函数使用
    let id1 = id(52);
    let id2 = id("Matthew");
    let id3 = id(3.14);
    println!("ID: {}, {}, {}", id1, id2, id3);
    
    // 泛型结构体使用
    let mut int_container = Container::new(42);
    let str_container = Container::new("Hello Rust");
    
    println!("Int container: {}", int_container.get());
    println!("Str container: {}", str_container.get());
    
    int_container.set(100);
    println!("Updated int container: {}", int_container.get());
    
    // trait bounds 使用
    let sum = add(5, 10);
    let product = multiply(5, 10);
    println!("5 + 10 = {}", sum);
    println!("5 * 10 = {}", product);
    
    // 泛型枚举使用
    let success = Result::Ok("Operation successful");
    let failure = Result::Err("Something went wrong");
    
    match success {
        Result::Ok(value) => println!("Success: {}", value),
        Result::Err(error) => println!("Error: {}", error),
    }
    
    match failure {
        Result::Ok(value) => println!("Success: {}", value),
        Result::Err(error) => println!("Error: {}", error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_id_function() {
        assert_eq!(id(42), 42);
        assert_eq!(id("test"), "test");
    }
    
    #[test]
    fn test_container() {
        let mut container = Container::new(123);
        assert_eq!(container.get(), &123);
        container.set(456);
        assert_eq!(container.get(), &456);
    }
    
    #[test]
    fn test_arithmetic() {
        assert_eq!(add(10, 20), 30);
        assert_eq!(multiply(10, 20), 200);
    }
}