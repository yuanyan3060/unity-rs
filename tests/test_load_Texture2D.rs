use unity_rs::classes::Texture2D;
use unity_rs::Env;

#[test]
fn test_load_texture2D(){
    let bundle = include_bytes!("../examples/unpack_image/char_1016_agoat2.ab");
    let mut env = Env::new();
    env.load_from_slice(bundle).expect("Load failure");

    for obj in env.objects() {
        println!("{:?}", obj.class());
        if obj.class() != unity_rs::ClassID::Texture2D {
            continue;
        }
        let s: Texture2D = obj.read().expect("Read Failure");
        s.decode_image().expect("Decode Failure").save(format!("Texture2D {}.png", s.name)).expect("Save Failure");
    }

}