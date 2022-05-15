use std::{ thread, time, io };
use kira::manager::{ AudioManager, AudioManagerSettings };
use kira::manager::backend::cpal::CpalBackend;
use kira::sound::static_sound::{ StaticSoundData, StaticSoundSettings };
use kira::{ tween, StartTime, Volume, LoopBehavior };
use casual_logger::{ Log, Opt };

fn main() {
    Log::set_opt(Opt::Release);
    Log::remove_old_logs();

    let result = AudioManager::<CpalBackend>::new(
            AudioManagerSettings::default()
    );
    let mut manager = match result {
        Ok(m) => m,
        Err(e) => {
            panic!("{}", Log::fatal(&e.to_string()));
        }
    };
    let result = StaticSoundData::from_file(
        "noise.wav",
        StaticSoundSettings::new().loop_behavior(
            LoopBehavior {
                start_position: 0.0
            }
        )
    );
    let sound_data = match result {
        Ok(d) => d,
        Err(e) => {
            panic!(
                "{}\nYou need to put noise.wav in the same directory.",
                Log::fatal(&e.to_string())
            );
        } 
    };
    let result = manager.play(sound_data);
    let mut handle = match result {
        Ok(h) => h,
        Err(e) => {
            panic!("{}", Log::fatal(&e.to_string()));
        }
    };

    let stdin = io::stdin();

    println!("Welcome!!\nThis is WhiteNoisePlayer.\n");
    println!("Please type...");
    println!("[q] for quitting.");
    println!("[w] for volume up.");
    println!("[s] for volume down.");

    let tick_delta = time::Duration::from_millis(500);
    let mut current_volume = 0.0;
    let volume_delta = 2.0;
    let volume_tween = tween::Tween {
        start_time: StartTime::Immediate,
        duration: time::Duration::from_millis(250),
        easing: tween::Easing::Linear
    };
    
    loop
    {
        let mut buffer = String::new();
        let result = stdin.read_line(&mut buffer);
        match result {
            Ok(_n) => {},
            Err(e) => {
                Log::error(&e.to_string());
                Log::flush();
                continue;
            }
        };
        let cleaned = buffer.trim();

        match cleaned {
            "q" => {
                break;
            },
            "w" => {
                current_volume += volume_delta;
                let result = handle.set_volume(
                    Volume::Decibels(current_volume),
                    volume_tween
                );
                match result {
                    Ok(()) => {},
                    Err(e) => {
                        Log::error(&e.to_string());
                        Log::flush();
                        continue;
                    }
                };
            },
            "s" => {
                current_volume -= volume_delta;
                let result = handle.set_volume(
                    Volume::Decibels(current_volume),
                    volume_tween
                );
                match result {
                    Ok(()) => {},
                    Err(e) => {
                        Log::error(&e.to_string());
                        Log::flush();
                        continue;
                    }
                };
            },
            &_ => { }
        }

        thread::sleep(tick_delta);
    };

    let result = handle.stop(tween::Tween {
        start_time: StartTime::Immediate,
        duration: time::Duration::from_millis(500),
        easing: tween::Easing::Linear
    });
    match result {
        Ok(()) => {},
        Err(e) => {
            panic!("{}", Log::fatal(&e.to_string()));
        }
    };

    Log::flush();
    thread::sleep(time::Duration::from_millis(1000));
    println!("Thank you byebye!!");
}
