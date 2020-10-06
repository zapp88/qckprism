mod cli;
mod qck;

fn main() -> rusb::Result<()> {
    let args = cli::fetch_args();
    qck::send_to_device(qck::Command{
        light_level : args.light_level,
        first_color : args.first_color as qck::Color,
        second_color : args.second_color as qck::Color,
    })
}