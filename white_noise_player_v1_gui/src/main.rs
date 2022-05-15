use std::{ time, thread };
use std::sync::mpsc;
use std::sync::mpsc::{ Sender, Receiver };
use iced::{
    button,
    Sandbox, Settings,
    Button, Element, Column, Text, Alignment, Length,
    alignment::{ Horizontal }
};
use kira::{
    tween, StartTime, Volume, LoopBehavior
};
use kira::manager::{
    AudioManager, AudioManagerSettings,
    backend::cpal::CpalBackend
};
use kira::sound::static_sound::{
    StaticSoundData, StaticSoundSettings
};
use casual_logger::{ Log, Opt };

enum ControlMessage {
    Control(f64),
    Quit
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerMessage {
    VolumeUpPressed,
    VolumeDownPressed,
    QuitPressed
}

struct Player {
    current_volume: f64,
    volume_delta: f64,
    control_channel: Sender<ControlMessage>,
    is_alive: bool,

    volume_up_button: button::State,
    volume_down_button: button::State,
    quit_button: button::State
}

impl Sandbox for Player {
    type Message = PlayerMessage;
    
    fn new() -> Self {
        let (
            control_tx,
            control_rx
        ): (Sender<ControlMessage>, Receiver<ControlMessage>) = mpsc::channel();
        
        thread::spawn(move || {
            // sound setting
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
            
            // start sound on sound thread
            let result = manager.play(sound_data);
            let mut handle = match result {
                Ok(h) => h,
                Err(e) => {
                    panic!("{}", Log::fatal(&e.to_string()));
                }
            };
            
            loop {
                let result = control_rx.recv();
                let recv = match result {
                    Ok(r) => r,
                    Err(e) => {
                        Log::error(&e.to_string());
                        Log::flush();
                        continue;
                    }
                };
                
                match recv {
                    ControlMessage::Control(n) => {
                        let result = handle.set_volume(
                            Volume::Decibels(n),
                            tween::Tween {
                                start_time: StartTime::Immediate,
                                duration: time::Duration::from_millis(250),
                                easing: tween::Easing::Linear
                            }
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
                    ControlMessage::Quit => {
                        let result = handle.stop(tween::Tween {
                            start_time: StartTime::Immediate,
                            duration: time::Duration::from_millis(500),
                            easing: tween::Easing::Linear
                        });
                        match result {
                            Ok(()) => {},
                            Err(e) => {
                                Log::error(&e.to_string());
                                Log::flush();
                                continue;
                            }
                        };
                    
                        thread::sleep(time::Duration::from_millis(500));
                        break;
                    },
                };
            };    
        });

        Self {
            current_volume: 0.0,
            volume_delta: 2.0,
            control_channel: control_tx,
            is_alive: true,

            volume_up_button: button::State::default(),
            volume_down_button: button::State::default(),
            quit_button: button::State::default()
        }
    }

    fn title(&self) -> String {
        String::from("WhiteNoisePLayer")
    }

    fn view(&mut self) -> Element<PlayerMessage> {
        Column::new()
            .padding(20)
            .spacing(20)
            .align_items(Alignment::Center)
            .push(
                Text::new("Welcome to WhiteNoisePlayer!!")
                .horizontal_alignment(Horizontal::Center)
            )
            .push(
                Text::new("Volume")
                .horizontal_alignment(Horizontal::Center)
            )
            .push(
                Button::new(
                    &mut self.volume_up_button,
                    Text::new("+")
                    .horizontal_alignment(Horizontal::Center)
                )
                .width(Length::Units(64))
                .on_press(PlayerMessage::VolumeUpPressed)
            )
            .push(
                Button::new(
                    &mut self.volume_down_button,
                    Text::new("-")
                    .horizontal_alignment(Horizontal::Center)
                )
                .width(Length::Units(64))
                .on_press(PlayerMessage::VolumeDownPressed)
            )
            .push(
                Text::new("Thank you !!")
                .horizontal_alignment(Horizontal::Center)
            )
            .push(
                Button::new(
                    &mut self.quit_button,
                    Text::new("quit")
                    .horizontal_alignment(Horizontal::Center)
                )
                .width(Length::Units(64))
                .on_press(PlayerMessage::QuitPressed)
            )
        .into()
    }

    fn update(&mut self, message: PlayerMessage) {
        match message {
            PlayerMessage::VolumeUpPressed => {
                self.current_volume += self.volume_delta;
                let result = self.control_channel.send(
                    ControlMessage::Control(self.current_volume)
                );
                match result {
                    Ok(()) => {},
                    Err(e) => {
                        Log::error(&e.to_string());
                        Log::flush();
                    }
                };
            },
            PlayerMessage::VolumeDownPressed => {
                self.current_volume -= self.volume_delta;
                let result = self.control_channel.send(
                    ControlMessage::Control(self.current_volume)
                );
                match result {
                    Ok(()) => {},
                    Err(e) => {
                        Log::error(&e.to_string());
                        Log::flush();
                    }
                };
            },
            PlayerMessage::QuitPressed => {
                let result = self.control_channel.send(ControlMessage::Quit);
                match result {
                    Ok(()) => {},
                    Err(e) => {
                        panic!("{}", Log::fatal(&e.to_string()));
                    }
                };
                
                Log::flush();
                thread::sleep(time::Duration::from_millis(500));
                self.is_alive = false;
            }
        };
    }

    fn should_exit(&self) -> bool {
        !self.is_alive
    }
}

pub fn main() -> iced::Result {
    Log::set_opt(Opt::Release);
    Log::remove_old_logs();

    // gui config
    let mut settings = Settings::default();
    settings.window.size = (300, 300);
    settings.window.resizable = false;

    // gui start
    Player::run(settings)
}
