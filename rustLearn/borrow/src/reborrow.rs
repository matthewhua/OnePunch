fn main() {
    let mut data = vec![1, 2, 3, 4];
    let b = &mut data;

}

fn sum(v: &mut Vec<i32>) {
    println!("addr of the ref v:{:p}", %v);
    v.iter().sum()
}