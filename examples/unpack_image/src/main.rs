fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = unity_rs::Env::new();
    let data = std::fs::read("char_1016_agoat2.ab")?;
    env.load_from_slice(&data)?;
    for obj in env.objects() {
        if obj.class() != unity_rs::ClassID::Sprite {
            continue;
        }
        let s: unity_rs::Sprite = obj.read()?;
        s.decode_image()?.save(format!("{}.png", s.name))?;
    }
    Ok(())
}
