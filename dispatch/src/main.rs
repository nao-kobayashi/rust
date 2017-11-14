trait Foo {
    fn method(&self) -> String;
}

impl Foo for u8 {
    fn method(&self) -> String {
        format!("u8 {}", *self)
    }
}

impl Foo for String {
    fn method(&self) -> String {
        format!("string {}", *self)
    }
}

fn do_something<T: Foo>(x: T){
    println!("{}", x.method());
}

fn do_something2(x: &Foo){
    println!("{}", x.method());
}

fn do_(x: &String) {
    println!("{}", x)
}


fn main() {
    let x = 5u8;
    let y = "Hello".to_string();
    let z = "Hello".to_string();

    do_something(x);
    do_something(y);

    do_something2(&x as &Foo);
    do_something2(&z);

    let test = "bollow".to_string();
    do_(&test);
    do_(&test);
    
}
