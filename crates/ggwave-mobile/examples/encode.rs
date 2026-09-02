use ggwave_mobile::{MobileGgWave, MobileProtocol, MobileTuning};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let codec = MobileGgWave::new(MobileTuning::default())?;
    let wave = codec.encode(b"BINGO", MobileProtocol::UltrasonicFast, 85)?;
    println!("{} f32 samples", wave.len());
    Ok(())
}
