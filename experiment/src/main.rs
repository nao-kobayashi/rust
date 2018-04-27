/*fn main() {
    /*
    let mut v = vec![1, 2, 3, 4, 5];

    for i in &v {
        println!("A reference to {}", i);
    }

    for i in &mut v {
        println!("A mutable reference to {}", i);
    }

    for i in v {
        println!("rTake ownership of the vector and its element {}", i);
    }

    let x = 5;

    match x {
        e @ 1 ... 5 => println!("got a range element {}", e),
        _ => println!("anything"),
    }*/
/*
    //&str
    let s = "aaabbbccc";
    let m = s;
    let n = s;
    println!("{}", s);
    println!("{}", m);
    println!("{}", n);


    //let s = vec!["udon".to_string(), "ramen".to_string(), "soba".to_string()];
    let s = vec!["udon", "ramen", "soba"];
    let t = s;
    let u = s;
*/
    let r = 1;
    {
        let x = &r;
        println!("{}", x);
    }

}
*/
/*
static mut STASH: &i32 = &128;
static WORTH_POINTING_AT: i32 = 1000;
fn f(p: &'static i32) {
    unsafe {
        STASH = p;
    }
}

fn g<'a>(p: &'a i32) { 
    println!("{}", p);
 }

fn smallest(v: &[i32]) -> &i32 {
    let mut s = &v[0];
    for r in &v[1..] {
        if *r < *s { s = r; }
    }
    s
}

struct S<'a> {
    r: &'a i32
}
*/
struct S2<'a, 'b> {
    x: &'a i32,
    y: &'b i32
}

fn main(){

    let mut v = (136, 139);
    let m = &mut v;
    let m0 = &mut m.0;      // ok: reborrowing mutable from mutable
    let r1 = &m.1;          // ok: reborrowing shared from mutable,
                            // and doesn't overlap with m0
    let xxx = &m.0;
    *m0 = 137;
    println!("{}", m0);
    println!("{}", r1);
    //v.1;                    // error: access through other paths still forbidden

/*
    let x = 10;
    let r; 
    {
        let y = 20;
        {
            let s = S2 { x: &x, y: &y};
            r = s.x;
        }
    }
*/
    /*let s;
    {
        let x = 10;
        s = S {r: &x};
    }
    println!("{}", s.r);

    let s;
    {
        let parabola = [9, 4, 1, 0, 1, 4, 9];
        s = smallest(&parabola);
    }
    assert_eq!(*s, 0); // bad: points to element of dropped array

    let x = 10;
    g(&x);
    println!("{}", x);

    //f(&x);

    f(&10); 
    unsafe {
        println!("{}", STASH);
    }

    f(&WORTH_POINTING_AT);
    unsafe {
        println!("{}", STASH);
    }*/
}