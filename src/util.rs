use bluenoise::BlueNoise;
use rand_pcg::Pcg64Mcg;


pub fn blue_noise_image(w: u32, h: u32) -> Vec<u8> {
    let mut noise = BlueNoise::<Pcg64Mcg>::new(w as f32, h as f32, 5.0);
    let noise_black = noise.with_samples(w * (h / 3)).with_seed(10);
    
    let mut noise2 = BlueNoise::<Pcg64Mcg>::new(w as f32, h as f32, 5.0);
    let noise_gray = noise2.with_samples(w * (h / 3)).with_seed(20);

    let mut img: Vec<u8> = vec![0; (w * h) as usize];

    for p in noise_black {
        img[(p.y as u32 * w + p.x as u32) as usize] = 255;
    }
    let mut c = 0;
    for p in noise_gray {
        if p.y as u32 * w + p.x as u32 == 255 {
            break;
        }
        c += 1;
        img[(p.y as u32 * w + p.x as u32) as usize] = 127;
    }
    dbg!(c);

    img
}