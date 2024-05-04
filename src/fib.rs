fn nth_fib(n: u32) -> u32 {
    if n <= 2 {
        return n - 1;
    }
    let mut i = 2;
    let mut a = 0;
    let mut b = 1;
    while i < n {
        let c = a + b;
        a = b;
        b = c;
        i += 1;
    }
    return b;
}
