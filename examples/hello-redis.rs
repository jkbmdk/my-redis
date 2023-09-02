use redis::Commands;

fn main() {
    let client = redis::Client::open("redis://127.0.0.1").unwrap();
    let mut con = client.get_connection().unwrap();

    let _ : () = con.set("hello", 42).unwrap();
    let result: i32 = con.get("hello").unwrap();

    println!("got value from the server: result={:?}", result);
}
