mod api;
mod git;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching data..");
    let res = api::fetch_data("https://api.github.com/users/lepton9").await?;
    // println!("{:?}", r);
    Ok(())
}
