mod ownership1;
mod check_copy;

use std::iter::Iterator;

fn main() {
    let data = vec![1,2,3,4];
    let data1 = data;
    println!("sum of data1: {}", sum(data1));

    // 下面两句无法编译通过
    // println!("data1: {:?}", data1);
    // 不能使用已经使用过的变量
    // println!("sum of data: {}", sum(data));
}


fn sum(data: Vec<u32>) -> u32 {
    data.iter().fold(0, |acc, &x| acc + x)
}