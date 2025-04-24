#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    filer_lib::run().await
}
