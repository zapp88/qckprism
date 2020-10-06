mod cli;
mod qck;

fn main() -> rusb::Result<()> {
    let args = cli::fetch_cli_args();
    qck::send_to_device(qck::Command{
        light_level : args.light_level,
        first_color : args.first_color,
        second_color : args.second_color,
    })
}