#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;

mod passwords;


fn main() {
    let hash = passwords::hash_password("thing").unwrap();
    let hash_2 = passwords::hash_password("thing").unwrap();
    passwords::verify_password("thing", &hash).unwrap();
    println!("{}", hash.to_string());
    println!("{}", hash_2.to_string());
    rocket::ignite().mount("/", routes![index]).launch();
}