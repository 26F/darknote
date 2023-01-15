
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use std::f64::consts::PI;
use std::f64::consts::E;
use hound;


fn logistic(x: f64) -> f64 {

    let result = 1.0 / (1.0 + f64::powf(E, -x));

    return result;

}


fn linear_interpolation(initial: f64, target: f64, t: f64) -> f64 {

    let t = logistic(t);

    return (1.0 - t) * initial + t * target

}


// frequency as a function of numbered piano key
fn keyboard(n: usize) -> f64 {

    return 440_f64 * (2_f64.powf((n as f64 - 49_f64) / 12_f64));

}



// step-wise square wave
fn square_wave(x: f64, freq: f64) -> f64 {

    let period = 1.0 / freq;

    let t = x % period;

    if t < period / 2_f64 {

        return 1_f64;

    }


    return -1_f64;

}



// sawtooth
fn sawtooth(x: f64, freq: f64) -> f64 {

    return 2.0 * ((freq * x) % 1.0 / freq) * freq - 1.0;

}



fn main() {

    // project sample rate
    let p_sample_rate = 44100.0;

    // seed random number generator
    let mut generator = StdRng::seed_from_u64(0);

    // positioning of the chord in pitch space
    let pitch_shift: i32 = -1;

    let chord = vec![9 + pitch_shift, 16 + pitch_shift, 21 + pitch_shift, 28 + pitch_shift, 
                     33 + pitch_shift, 40 + pitch_shift, 45 + pitch_shift, 52 + pitch_shift, 
                     57 + pitch_shift, 64 + pitch_shift, 69 + pitch_shift, 74 + pitch_shift,
                     79 + pitch_shift, 84 + pitch_shift, 89 + pitch_shift, 94 + pitch_shift,
                     99 + pitch_shift, 104 + pitch_shift, 109 + pitch_shift];


    let mut tones = Vec::new();
    let mut init_tones = Vec::new();

    let mut sin_phases = Vec::new();

    let num_harmonics = 6;

    let num_seconds = 40;       // num seconds of sound file 
    let cut_off = p_sample_rate * num_seconds as f64;

    let target_seconds = p_sample_rate * 16.0; 
    

    let num_tones = chord.len();

    let t_scale = 7.999; // times the notes reaching harmony

    let sin_peroid = 0.001;

    let t_correction_scale = 4.0; // time for sin stereo detune pitch to become relatively stable

    let initial_volume = 0.05;

    let fade_in_stepness = 30.0;

    let init_vol_fade_in = 25.0;

    for note in &chord {
    
        let harmonic = keyboard(*note as usize);

        let init_freq = (generator.gen_range(0..100) + 300) as f64;

        let sin_phase = generator.gen_range(0..200) as f64;

        init_tones.push(init_freq);

        tones.push(harmonic);

        sin_phases.push(sin_phase);

    }


    let spec = hound::WavSpec {

        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Int,

    };

    let mut writer = hound::WavWriter::create("rustnote.wav", spec).unwrap();

    for t in (0..p_sample_rate as i32 * num_seconds).map(|x| x as f64 / (p_sample_rate * num_seconds as f64)) {

        let mut waveform = 0_f64;

        let mut leftchan = 0.0;
        let mut rightchan = 0.0;

        for index in 0..num_tones {

            let mut pitch = tones[index];

            if (t * p_sample_rate * num_seconds as f64) < target_seconds {

                pitch = linear_interpolation(init_tones[index], tones[index], (t * t_scale) - 4.0);

            }


            pitch += f64::sin(t * sin_peroid + sin_phases[index]) * (1.0 - logistic((t * t_correction_scale) - 4.0));

            let right_chan_fadein = logistic(t * 31.0 - 4.0);

            for harmonic in 1..=num_harmonics {

                let note = pitch * harmonic as f64;

                let sin_part = 0.80;
                let square_part = 0.05;
                let saw_part = 0.15;

                let sine = (t * note * 2.0 * PI * num_seconds as f64).sin();

                let square = square_wave(t, note * num_seconds as f64); 

                let saw = sawtooth(t, note * num_seconds as f64);

                waveform += sine * sin_part + square * square_part + saw * saw_part;

                if index % 2 == 0 {

                    leftchan += waveform * (1.0 - ((num_tones as f64) / ((index + 1) as f64)));
                    rightchan += waveform * ((num_tones as f64) / ((index + 1) as f64));

                } else {

                    leftchan += waveform * ((num_tones as f64) / ((index + 1) as f64));
                    rightchan += waveform * (1.0 - ((num_tones as f64) / ((index + 1) as f64)));

                }

                rightchan *= right_chan_fadein;

            }

        }

        leftchan /= (chord.len() * num_harmonics * (num_tones / 2)) as f64;
        rightchan /= (chord.len() * num_harmonics * (num_tones / 2)) as f64;

        let mut amplitude = (i32::MAX as f64 ) * (logistic((t * fade_in_stepness) - 10.0) + initial_volume * logistic((t * init_vol_fade_in) - 2.0));

        amplitude *= logistic((-t * 24.0) + 15.0);

        // left channel
        writer.write_sample((leftchan * amplitude) as i32).unwrap();
            
        // right channel
        writer.write_sample((rightchan * amplitude) as i32).unwrap();


        if (t * p_sample_rate * num_seconds as f64) >= cut_off {

            break;

        }


    }

    // write the wav file
    writer.finalize().unwrap();

}