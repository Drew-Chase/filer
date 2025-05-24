use clap::Parser;

#[derive(Parser)]
#[command(
    version,
    name = "filer",
    about = "A file server for the Filer application",
    author = "Drew Chase"
)]
pub struct FilerArguments {
    #[arg(
        long = "disable-indexing",
        help = "Disables the automatic indexing of files",
        default_value = "false"
    )]
    pub disable_indexing: bool,
    #[arg(
        long = "disable-file-watchers",
        help = "Disables automatic watching and updating of file system changes in real-time",
        default_value = "false"
    )]
    pub disable_filewatchers: bool,
    #[arg(
        short,
        long,
        help = "Port to listen on, this will set the value temporarily. To set this permanently use the app-config.json file"
    )]
    pub port: Option<u16>,
}
