static mut N: &'static u32 = &{
    let mut test:u32 = 1;
    test
};

fn main() {
    println!("{}", N)
}