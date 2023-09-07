use redis::Commands;

fn main() {
    let client = redis::Client::open("redis://127.0.0.1").unwrap();
    let mut con = client.get_connection().unwrap();

    let _: () = con.set("dad", "Joshua").unwrap();
    let _: () = con.set("mom", "Margaret").unwrap();
    let result: Vec<String> = redis::cmd("MGET")
        .arg(&["dad", "mom"])
        .query::<Vec<String>>(&mut con)
        .unwrap();

    println!("got value from the server: result={:?}", result);
}
