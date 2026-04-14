const PI: f64 = std::f64::consts::PI;

static E: f32 = std::f32::consts::E;

// 常量的作用域 - const 在整个程序运行期间都有效
const GLOBAL_CONSTANT: &str = "I am global";

// static 变量的生命周期是整个程序
static mut MUTABLE_STATIC: i32 = 0;

fn main() {
    const V: u32 = 10;

    static V1: &str = "hello";
    println!("PI: {}, E: {}, V {}, V1: {}", PI, E, V, V1);
    
    // 演示常量的不可变性
    const LOCAL_CONST: i32 = 100;
    println!("Local constant: {}", LOCAL_CONST);
    
    // 演示 static 变量的使用
    unsafe {
        println!("Mutable static before: {}", MUTABLE_STATIC);
        MUTABLE_STATIC = 42;
        println!("Mutable static after: {}", MUTABLE_STATIC);
    }
    
    // 常量的内存位置
    println!("Global constant address: {:p}", &GLOBAL_CONSTANT);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constants() {
        assert_eq!(PI, std::f64::consts::PI);
        assert_eq!(E, std::f32::consts::E);
        assert_eq!(V, 10);
        assert_eq!(V1, "hello");
        assert_eq!(GLOBAL_CONSTANT, "I am global");
    }
}