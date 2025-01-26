use cmd_lib::run_fun as run;
use std::array::from_fn;

// brightness
pub fn brtns() -> String {
    format!(
        "brightness: {}",
        run! { brightnessctl i }
            .unwrap()
            .split(['(', ')'])
            .nth(1)
            .unwrap(),
    )
}

pub fn volume() -> String {
    format!(
        "volume: {}",
        run! { pactl get-sink-volume @DEFAULT_SINK@ }
            .unwrap()
            .split(' ')
            .nth(5)
            .unwrap()
    )
}

pub fn ram() -> String {
    let out = run! { free -h }.unwrap();
    let mem_line = out.lines().nth(1).unwrap();
    let mut split = mem_line.split(' ').filter(|i| !i.is_empty()).skip(1);
    let [total, used] = from_fn(|_| split.next().unwrap());

    format!(" {}/{}", used.replace("i", "B"), total.replace("i", "B"))
}

pub fn battery() -> String {
    let acpi_out = run! { acpi - b }.unwrap();
    let mut split = acpi_out.split([',', ':']);

    let status = split.nth(1).unwrap().trim();
    let percentage = split.next().unwrap().trim();
    let n = percentage.trim_end_matches('%').parse::<u8>().unwrap();
    let indicator = if status != "Not charging" {
        ""
    } else if n >= 75 {
        ""
    } else if n >= 50 {
        ""
    } else if n >= 25 {
        ""
    } else {
        ""
    };

    format!("{indicator}  {percentage}")
}

pub fn media() -> Option<String> {
    let status = run! { playerctl status }.ok()?;
    let player = run! { playerctl metadata --format "{{playerName}}" }.ok()?;
    let artist = run! { playerctl metadata --format "{{artist}}" }.ok()?;
    let title = run! { playerctl metadata --format "{{title}}" }.ok()?;
    let position = run! {
        playerctl metadata --format "{{duration(position)}} / {{duration(mpris:length)}}"
    }
    .ok()?;

    let indicator = if status == "Playing" { "" } else { "" };

    Some(format!(
        "{player}: {indicator} {title} - {artist} [{position}]"
    ))
}

pub fn storage() -> String {
    let x = run! { df -h --output=used,size,pcent / }.unwrap();
    let mut split = x
        .lines()
        .nth(1)
        .unwrap()
        .split(" ")
        .filter(|i| !i.is_empty());

    let [used, total, used_percent] = from_fn(|_| split.next().unwrap());

    format!("󰋊 {used} / {total} {used_percent}")
}

pub fn date() -> String {
    run! { date +"%a, %b %d %I:%M %P" }.unwrap()
}

pub fn cpu() -> String {
    format!(
        "  {}% 󰏈 {}C 󰈐 {} RPM",
        cpu_percentage(),
        cpu_temp(),
        cpu_fan()
    )
}

fn cpu_temp() -> u32 {
    let binding = run! { sensors | grep "Package id 0" }.unwrap();
    let line = binding.split("  ").nth(1).unwrap();
    line[1..line.len() - 4].parse().unwrap()
}

fn cpu_fan() -> u32 {
    run! { sensors | grep cpu_fan }
        .unwrap()
        .rsplit(' ')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap()
}

fn cpu_percentage() -> u32 {
    let f = || {
        let out = std::fs::read_to_string("/proc/stat").unwrap();
        let mut split = out[5..].split(' ');

        let total = split
            .clone()
            .filter(|i| !i.is_empty())
            .take(3)
            .map(|i| i.parse::<u32>().unwrap())
            .sum::<u32>();
        let idle = split.nth(3).unwrap().parse::<u32>().unwrap();

        (total + idle, idle)
    };

    let (previous_total, previous_idle) = f();
    std::thread::sleep(std::time::Duration::from_millis(500));
    let (total, idle) = f();

    100 * ((total - previous_total) - (idle - previous_idle)) / (total - previous_total)
}
