use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// 演示不同的类型系统和特性
fn main() {
    println!("=== Rust 类型系统学习 ===");
    
    // 1. 基本类型
    basic_types();
    
    // 2. 复合类型
    composite_types();
    
    // 3. 推断类型
    type_inference();
    
    // 4. 类型转换
    type_conversion();
    
    // 5. 泛型约束
    generic_constraints();
}

// 基本类型演示
fn basic_types() {
    println!("\n--- 基本类型 ---");
    
    // 整数类型
    let integer_types = (
        8i8,  // 8位有符号
        16i16, // 16位有符号
        32i32, // 32位有符号（默认）
        64i64, // 64位有符号
        128i128, // 128位有符号
        8u8,   // 8位无符号
        32u32, // 32位无符号
    );
    
    println!("整数类型: {:?}", integer_types);
    
    // 浮点数类型
    let float_types = (1.0f32, 3.14159265359f64);
    println!("浮点数类型: {:?}", float_types);
    
    // 布尔类型
    let boolean = true;
    println!("布尔类型: {}", boolean);
    
    // 字符类型
    let char_value = 'A';
    println!("字符类型: {}", char_value);
    
    // 字符串类型
    let string_literal = "Hello";
    let string_object = String::from("World");
    println!("字符串字面量: {}", string_literal);
    println!("字符串对象: {}", string_object);
    
    // 元组类型
    let tuple = (1, "hello", 3.14, true);
    println!("元组: {:?}", tuple);
    println!("元组第一个元素: {}", tuple.0);
    
    // 数组类型
    let array = [1, 2, 3, 4, 5];
    let slice = &array[1..3];
    println!("数组: {:?}", array);
    println!("数组切片: {:?}", slice);
    
    // 指针类型
    let ptr_value = 42;
    let reference = &ptr_value;
    let raw_ptr = reference as *const i32;
    println!("引用值: {}", reference);
    println!("原始指针: {:p}", raw_ptr);
}

// 复合类型演示
fn composite_types() {
    println!("\n--- 复合类型 ---");
    
    // Vector 动态数组
    let mut vector = vec![1, 2, 3];
    vector.push(4);
    println!("Vector: {:?}", vector);
    
    // HashMap 键值对
    let mut map = HashMap::new();
    map.insert("name", "Alice");
    map.insert("age", 30);
    println!("HashMap: {:?}", map);
    
    // Option 类型
    let some_value: Option<i32> = Some(10);
    let none_value: Option<i32> = None;
    println!("Some 值: {:?}", some_value);
    println!("None 值: {:?}", none_value);
    
    // Result 类型
    let ok_result: Result<&str, &str> = Ok("成功");
    let err_result: Result<&str, &str> = Err("失败");
    println!("Ok 结果: {:?}", ok_result);
    println!("Err 结果: {:?}", err_result);
    
    // Box 智能指针
    let boxed_value = Box::new(100);
    println!("Box 值: {}", boxed_value);
    
    // Rc 引用计数
    use std::rc::Rc;
    let rc_value = Rc::new(200);
    println!("Rc 值: {}", rc_value);
}

// 类型推断演示
fn type_inference() {
    println!"\n--- 类型推断 ---");
    
    // 显式类型注解
    let explicit_int: i32 = 42;
    let explicit_str: &str = "Hello";
    println!("显式类型 - int: {}, str: {}", explicit_int, explicit_str);
    
    // 隐式类型推断
    let implicit_int = 42; // 自动推断为 i32
    let implicit_str = "Hello"; // 自动推断为 &str
    let implicit_float = 3.14; // 自动推断为 f64
    println!("隐式类型 - int: {}, str: {}, float: {}", 
             implicit_int, implicit_str, implicit_float);
    
    // 表达式类型推断
    let expression_result = if true { 42 } else { 3.14 }; // 自动推断为 f64
    println!("表达式类型: {}", expression_result);
}

// 类型转换演示
fn type_conversion() {
    println!("\n--- 类型转换 ---");
    
    // as 关键字 - 显式转换（可能丢失精度）
    let a = 100i32;
    let b = a as i64;
    let c = a as f64;
    let d = a as u8;
    println!("as 转换 - i32->i64: {}, i32->f64: {}, i32->u8: {}", b, c, d);
    
    // try_into 和 TryFrom - 更安全的转换
    let num: u8 = 100;
    match num.try_into() {
        Ok(n) => println!("try_into: {}", n),
        Err(e) => println!("转换失败: {}", e),
    }
    
    // String & str 转换
    let s1 = "Hello";
    let s2 = String::from(s1);
    let s3 = &s2[..];
    println!("str->String->str: {}", s3);
    
    // 数字到字符串
    let num = 42;
    let num_str = num.to_string();
    let num_from_str: i32 = num_str.parse().unwrap();
    println!("数字转字符串: {}, 再转回数字: {}", num_str, num_from_str);
}

// 泛型约束演示
fn generic_constraints() {
    println!("\n--- 泛型约束 ---");
    
    // 实现简单的排序函数
    fn sort<T: Ord + Clone>(arr: &mut [T]) {
        arr.sort();
    }
    
    let mut numbers = [3, 1, 4, 1, 5, 9, 2, 6];
    sort(&mut numbers);
    println!("排序后的数字: {:?}", numbers);
    
    let mut strings = ["banana", "apple", "cherry"];
    sort(&mut strings);
    println!("排序后的字符串: {:?}", strings);
    
    // 泛型 trait 约束
    fn display<T: std::fmt::Debug>(item: T) {
        println!("调试显示: {:?}", item);
    }
    
    display(42);
    display("Hello");
    display([1, 2, 3]);
    
    // 多个 trait 约束
    fn process<T>(item: T) -> String 
    where T: std::fmt::Display + std::fmt::Debug
    {
        format!("处理结果: {} ({:?})", item, item)
    }
    
    let result = process(42);
    println!("多约束处理: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_types() {
        assert_eq!(true, !false);
        assert_eq!('A' as u8, 65);
    }
    
    #[test]
    fn test_composite_types() {
        let mut vector = vec![1, 2, 3];
        vector.push(4);
        assert_eq!(vector, [1, 2, 3, 4]);
    }
    
    #[test]
    fn test_option_handling() {
        let some_value = Some(10);
        let none_value: Option<i32> = None;
        
        assert_eq!(some_value.unwrap(), 10);
        assert!(none_value.is_none());
    }
    
    #[test]
    fn test_type_conversion() {
        let num: i32 = 100;
        let converted: u8 = num as u8;
        assert_eq!(converted, 100);
    }
    
    #[test]
    fn test_sorting() {
        let mut numbers = [3, 1, 4, 1, 5];
        sort(&mut numbers);
        assert_eq!(numbers, [1, 1, 3, 4, 5]);
    }
}