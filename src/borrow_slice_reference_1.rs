fn main() {
    let mut arr = [2, 4, 5, 6, 8, 9];
    let foa = before_first_odd(&arr[..]);
    for num in foa {
        println!("Num -> {num}")
    }
    arr[2] = 4;
}

fn before_first_odd(arr: &[i32]) -> &[i32] {
    for (i, &num) in arr.iter().enumerate() {
        if num % 2 == 1 {
            return &arr[..i];
        }
    }
    arr
}
